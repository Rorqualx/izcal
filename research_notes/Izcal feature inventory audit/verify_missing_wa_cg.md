# Verification of Missing rows: web platform APIs (WA) and CSS, graphics and media (CG)

Checked 24 October 2026 against `reports/` in the `feature-audit` worktree. Inputs were the 77 **Missing** rows in `map_web_platform_apis.md` and the 24 **Missing** rows in `map_css_graphics_media.md`. A Python regex sweep over all 33 reports used different terms from the mapper's: interface and attribute names, header names, spec short names, decision IDs, and broad synonyms such as "file input", "pressure", "WEBGL_", "orientation", "speech synthesis" and "processing-instruction". Every hit was read in context. Engine status is the one in the inventory notes (`web_platform_apis.md`, `css_graphics_media.md`). It was not re-checked.

## The rule-based holds

The mapper treated the [1E] (single-engine) rows as Missing because "no per-feature decision is recorded". The CG mapper also wrote that "no admission rule covers non-CSS web APIs in general". The corpus does have such a rule:

- **DM 6.3**: "Izcal's admission rule for single-engine features is LY-D10's." The sentence sits in the paragraph that defines the whole IDL corpus Izcal ships. It is not limited to DM's own APIs.
- **DM 9.1**: "Single-engine features beyond those admitted in 6.12 | When LY-D10's admission rule is met | Admission rule, not schedule".
- **AH 6.15**: "The corpus's rule (ST-D18 as applied by LY-D10, and DV-D9 for APIs) admits a web-facing feature when…". The app host reads the rule as covering every web-facing feature.
- The WA mapper's own status key defines **Excluded** as "refused, held by a recorded rule, or deferred with an owner".

Under that key, a single-engine or no-engine row that no spec admits counts as **held** (Excluded), not Missing. DM 6.18's gap rule ("recorded in the ledger as a gap before it is admitted") applies only at admission. Such rows are marked **Held (rule)** below. These are policy changes, not new evidence. A stricter reading, where every holdover needs a per-feature ledger entry, would keep them Missing, and the summary gives both counts. Three of these rows (WA-12.3, WA-13.5, WA-13.13) are HTTP headers that the networking spec would own. NW has no admission statement of its own, so for those rows the hold rests on DM 6.3 and AH 6.15 alone.

Evidence-based changes are marked **Covered**, **Partial** or **Excluded** and cite the spec text.

## WA rows (77)

| # | Feature | Mapper said | Verdict | Evidence |
|---|---|---|---|---|
| WA-1.18 | Out-of-order HTML streaming | Missing [1E] | **Actually covered** | ST 9.1 (staging table): "Parser behaviour in the pinned WHATWG standard (… `<template for>` …) is implemented as specified, whatever the shipping status", reason column "No hold". The mapper searched "template for" and missed this row |
| WA-2.3 | Heading level adjustment | Missing [1E] | Held (rule) | No hits for `headingoffset`/`headingreset`; AX 6.x only checks "heading-level skips". Unmerged HTML PR, Firefox flag only: DM 6.3, DM 9.1 |
| WA-2.14 | Processing instructions in HTML | Missing [1E] | **Actually covered** | ST 9.1: "Parser behaviour in the pinned WHATWG standard (customizable `<select>`, processing instructions…)"; ST 6.5: serialization "and the processing-instruction form"; ST 11.2: the standard "now creates processing-instruction nodes" (commit `320c05f6`). The DOM attribute API falls under DM 6.3's DOM/HTML corpus |
| WA-2.16 | `<model>` element | Missing [1E] | Held (rule) | Only PDF's glTF annotations (PD-D23). WebKit-only: DM 6.3, DM 9.1 |
| WA-3.8 | Directory upload | Missing | Confirmed missing | "file input" hits only LY 6.x localized button text; no `webkitdirectory`, no directory picker. Three engines |
| WA-3.9 | HTML Media Capture | Missing | Confirmed missing | No `capture` attribute or `accept="image"` hits (same as CG-15.13) |
| WA-3.10 | `<input type=checkbox switch>` | Missing [1E] | Held (rule) | AX 6.9 "switches" is the ARIA role only. WebKit-only |
| WA-3.13 | OpaqueRange | Missing [1E] | Held (rule) | ED 6.x text-control selection APIs (`setSelectionRange`) only. Chromium-only |
| WA-4.11 | Focusgroup | Missing [1E] | Held (rule) | Roving tabindex appears only in shell chrome (SH 6.x tab strip). Chromium-only |
| WA-4.15 | Pointer Lock | Missing | Confirmed missing | DM 6.5 and the DM 6.3 corpus list UI Events and Pointer Events only; no lock, `movementX` or raw input. Three engines |
| WA-4.16 | Keyboard Lock and Keyboard Map | Missing [1E] | Held (rule) | No `navigator.keyboard` or `keyboardLock` hits. Chromium-only |
| WA-5.9 | No-Vary-Search | Missing | Confirmed missing | No "Vary" or query-key cache rule. Its main consumer, Speculation Rules and prerender, is Stage 2 (S1 Section 9 table), but NVS is not named. Two engines |
| WA-5.11 | Web bundles and signed exchanges | Missing [1E] | Held (rule) | "bundle" hits are list or TUF bundles only. Chromium-only, in decline |
| WA-5.12 | Integrity Policy | Missing | Confirmed missing | NW 6.7 has only "integrity metadata" on the request; no `Integrity-Policy` header. Baseline |
| WA-5.13 | Signature-based integrity | Missing [1E] | Held (rule) | No HTTP Message Signatures or `Unencoded-Digest` hits. Chromium-only |
| WA-6.23 | Observable API | Missing [1E] | Held (rule) | "observable" hits are DET-1 prose only. Chromium-only |
| WA-8.9 | WebSocketStream | Missing [1E] | Held (rule) | DM 6.18 names only `WebSocket` objects. Chromium-only |
| WA-8.16 | URL Pattern | Missing | Confirmed missing | DM 6.10 allocates static routes (`addRoutes`, "at most 1,024 conditions") to NW, but no spec names `URLPattern`, which those conditions need. Baseline |
| WA-8.20 | Network Information and Save-Data | Missing [1E] | Held (rule) | Only ST 6.x: "behaviour depend on network type" is refused for `document.write`. Chromium-only |
| WA-9.10 | File System Observer | Missing [1E] | Held (rule) | No hits. Chromium-only |
| WA-9.11 | File and Directory Entries API | Missing | Confirmed missing | ED 6.11 paste and drop give "files as broker-granted read handles"; no entries or directory reader. Three engines |
| WA-10.6 | Declarative Web Push | Missing [1E] | Held (rule) | NW 6.14 covers ordinary Web Push only (Stage 2). WebKit-only |
| WA-10.9 | Periodic Background Sync | Missing [1E] | Held (rule) | S1 13.2: "push and background sync are Stage 2", periodic not named. Chromium-only |
| WA-10.11 | Content Index | Missing [1E] | Held (rule) | No hits. Chromium Android only |
| WA-11.1 | Web App Manifest | Missing | Confirmed missing | AH 2 and 6.1 discuss PWAs as history and answer "why not a PWA?"; AH 6.8's "PWA-grade" window contract is for `app://` apps only. No web manifest decision. Multi-engine |
| WA-11.2 | Install prompts | Missing [1E] | Held (rule) | "install prompt" hits are AH's `.izapp` prompt only. Chromium-only |
| WA-11.3 | Launch Handler and File Handling | Missing [1E] | Held (rule) | EX 6.x `file_handlers` key, Stage 3 only. Chromium-only |
| WA-11.5 | Web Share Target | Missing [1E] | Held (rule) | No hits. Chromium Android/ChromeOS |
| WA-11.6 | Badging API | Missing | Confirmed missing | "badge" hits are trust and extension badges only. Two engines |
| WA-11.7 | Window Controls Overlay | Missing [1E] | Held (rule) | SH 6.x: "Custom title bars are a Stage 2 experiment" covers shell chrome, not the web API. Chromium-only |
| WA-11.8 | Window Management | Missing [1E] | Held (rule) | MD 6.5 refuses only the HDR-headroom extension; AH 2's refusal concerns app windows. Chromium-only |
| WA-11.11 | Get Installed Related Apps | Missing [1E] | Held (rule) | No hits. Chromium-only |
| WA-11.12 | PWA origin migration | Missing [1E] | Held (rule) | Only EX's MV2→MV3 migration tool. Chromium-only |
| WA-12.3 | Document Policy | Missing [1E] | Held (rule) | No hits. Chromium-only; header, NW has no admission rule of its own (see above) |
| WA-12.6 | Web Share | Missing | Confirmed missing | No `share()`, share sheet or "Web Share" hits. Meets the two-engine bar |
| WA-12.7 | Contact Picker | Missing [1E] | Held (rule) | Only ED's spell-dictionary "address book" quote. Chromium Android only |
| WA-12.8 | EyeDropper | Missing [1E] | Held (rule) | "colour picker" hits are `<input type=color>` pickers (LY 6.x, UI design). Chromium-only |
| WA-12.10 | Web Speech API | Missing | **Actually partial** | MD 6.20 out-of-scope row: "speech synthesis and recognition (a separate spec; recognition is an ML service feature under B10)". Owned in name only: the separate spec does not exist and has no API design. The CG mapper rated the same text Partial (CG-16.3, 16.4) |
| WA-12.12 | Selective permissions intervention | Missing [1E] | Confirmed missing | FP-D7 and CB 6.x cover heavy-ad resource interventions only. A browser policy, not a web API, so the admission rule does not fit |
| WA-13.5 | Document-Isolation-Policy | Missing [1E] | Held (rule) | No hits. Chromium-only; header (see above) |
| WA-13.13 | Connection Allowlists | Missing [1E] | Held (rule) | "allowlist" hits are seccomp and crate allowlists only. Chromium-only; header (see above) |
| WA-14.5 | Topics API | Missing [1E] | Held (rule) | SS 6.15: "the retired Privacy Sandbox APIs are a privacy-spec question, out of scope here"; NW 2 lesson: "partitioning, not new APIs, is the shipped privacy tool". Retired, Chromium-only |
| WA-14.6 | Protected Audience | Missing [1E] | Held (rule) | As WA-14.5. Deprecated, Chromium-only |
| WA-14.7 | Shared Storage and selectURL | Missing [1E] | Held (rule) | As WA-14.5 |
| WA-14.8 | Private Aggregation | Missing [1E] | Held (rule) | As WA-14.5. Removed from Chrome |
| WA-14.9 | Attribution Reporting | Missing [1E] | Held (rule) | As WA-14.5. The PATWG successor would be a new row when it ships |
| WA-14.10 | Private Click Measurement | Missing [1E] | Held (rule) | No hits ("PCM" hits are audio). WebKit-only |
| WA-14.11 | Private State Tokens | Missing [1E] | Held (rule) | CB 2 history row only, plus SS 6.15's deferral. Chromium-only |
| WA-14.12 | Private Verification Tokens | Missing [1E] | Held (rule) | No hits. Origin trial only |
| WA-15.1 | Credential Management | Missing | Confirmed missing | S1 13.2 commits to "WebAuthn through platform authenticators", which needs `navigator.credentials`, but no spec names the container, `PasswordCredential` or `FederatedCredential` |
| WA-15.3 | FedCM | Missing [1E] | Held (rule) | SS 6.15 and SS 11.4: "CHIPS and FedCM … a privacy-spec question, out of scope here". Chromium-only |
| WA-15.4 | Digital Credentials API | Missing | Confirmed missing | No mdoc, mDL, OpenID4VP or "identity document" hits. Two engines |
| WA-15.5 | WebOTP | Missing [1E] | Held (rule) | Only DT 6.x redaction of one-time-code fields. Chromium Android |
| WA-15.6 | Device Bound Session Credentials | Missing [1E] | Held (rule) | "device bound" hits are USB drivers only. Chromium-only |
| WA-15.7 | Email Verification Protocol | Missing [1E] | Held (rule) | No hits. Origin trial |
| WA-15.10 | Secure Payment Confirmation | Missing [1E] | Held (rule) | DV 6.x: "Payments stay in Payment Request" is about device routing, not SPC. Chromium-only |
| WA-15.11 | Apple Pay JS | Missing [1E] | Held (rule) | No hits. WebKit vendor API |
| WA-16.2 | Device orientation and motion events | Missing | Confirmed missing | CB 6.x lists "sensors" only as a fingerprinting surface to inventory. Three engines |
| WA-16.3 | Generic Sensor family | Missing [1E] | Held (rule) | As WA-16.2. Chromium-only |
| WA-16.4 | Battery Status | Missing [1E] | **Actually excluded** | FP 6.15 developer-signals table: "Device, battery or pressure state \| Never \| —" |
| WA-16.6 | Compute Pressure | Missing [1E] | **Actually excluded** (weak) | The same FP 6.15 row, "pressure state … Never". In FP's context it reads as memory pressure, so the exclusion is implied, not named. The rule holds it anyway |
| WA-16.7 | CPU Performance API | Missing [1E] | Held (rule) | DM 6.16 normalizes `hardwareConcurrency` through the environment record only. Chromium-only |
| WA-16.9 | Vibration | Missing | Confirmed missing | No `vibrate` or haptic hits. Two engines (mobile) |
| WA-16.10 | Screen Orientation | Missing | Confirmed missing | DM 6.18 and 6.16: `screen` comes from the normalized environment record, but `screen.orientation` and `lock()` are not designed. Three engines |
| WA-16.11 | Idle Detection | Missing [1E] | Held (rule) | AH 6.x permission-registry list only. Chromium-only |
| WA-16.19 | Device Posture | Missing [1E] | Held (rule) | No hits. Chromium; Safari flag |
| WA-16.20 | Viewport Segments | Missing [1E] | Held (rule) | No hits. Chromium; Safari flag |
| WA-16.21 | Shape Detection | Missing [1E] | Held (rule) | Only DV 3's "barcode scanners" as devices. Chromium-only (= CG-11.13) |
| WA-16.22 | Presentation API | Missing [1E] | Held (rule) | "cast" hits are BroadcastChannel and castlabs only. Chromium-only (= CG-14.17) |
| WA-16.23 | Input Device Capabilities | Missing [1E] | Held (rule) | No hits. Chromium-only |
| WA-16.24 | Managed device attributes | Missing [1E] | Held (rule) | DV 6.x "device attributes" are USB and HID descriptors. ChromeOS managed only |
| WA-17.6 | Server Timing | Missing | Confirmed missing | No `Server-Timing` or `serverTiming` hits; DM 6.18 names only the Resource Timing family. Baseline |
| WA-17.13 | Soft navigations | Missing [1E] | Held (rule) | ESS1 fact-check row only. Chromium-only |
| WA-17.14 | Memory and profiling | Missing [1E] | **Actually excluded** | FP 6.15: "`measureUserAgentSpecificMemory` \| Not implemented (FP-D10)"; FP 9.4 FP-D10 (a) has a trigger ("A second engine ships it…"). For profiling, JS 6.10 says only: "Any page-facing profiling API keeps JS Self-Profiling's cross-origin limits" (no admission; held by the rule). The mapper searched this exact name and reported no hits |
| WA-17.18 | Scroll performance timing | Missing [1E] | Held (rule) | No hits. No spec or WG resolution exists |
| WA-22.10 | Test Utils | Missing | Confirmed missing | DT-5 covers WPT testdriver commands, not `TestUtils.gc()` |
| WA-22.12 | Origin trials | Missing | Confirmed missing | Origin trials appear only as other vendors' status notes. DV 6.15's per-origin "Experimental device APIs" test-site list is a developer setting ("not an admission"), not a trial-token mechanism |

## CG rows (24)

| # | Feature | Mapper said | Verdict | Evidence |
|---|---|---|---|---|
| CG-1.2 | CSS Snapshot 2026 | Missing | Confirmed missing | "Snapshot" hits are spec pins and DOM snapshots only. No stable-set floor |
| CG-1.9 | CSS Namespaces (`@namespace`) | Missing | **Actually partial** | ST 6.x anchor table: "Selectors \| Selectors 4, CSS Shadow 1, Nesting 1" and "Cascade and values \| … CSSOM 1". Selectors 4 defines `ns\|E` and CSSOM 1 defines `CSSNamespaceRule`. Missing: css-namespaces-3 (the `@namespace` rule and its default-namespace semantics) is not anchored |
| CG-4.28 | `shape()` | Missing | **Actually partial** | LY 8.1 LY-2: "For each layout-related Interop focus area … score ≥ the best shipping engine's", and LY-D10 admits Interop areas, so the rule binds it. Missing: any design (AN 6.x interpolates "basic shapes and `path()`" only) |
| CG-5.7 | Spatial navigation | Missing | Held (ST-D18) | A CSS module (css-nav-1), shipped behind a flag only. ST 9.4 ST-D18 (a) holds it, as the CG mapper did for CG-1.15 and CG-4.29 ("rule applies, not named") |
| CG-6.5 | `text-size-adjust` | Missing | Held (ST-D18), borderline | css-size-adjust is an editor's draft: Chromium partial, WebKit prefixed only, Gecko none, so not two unprefixed engines. If prefixed WebKit counts, it reverts to Confirmed missing |
| CG-9.10 | EyeDropper | Missing | Held (rule) | Only `<input type=color>` pickers. Single-engine: DM 6.3, DM 9.1, AH 6.15 (= WA-12.8) |
| CG-11.9 | WebGL extensions | Missing | **Actually partial** | GR 6.20 privacy table: "the tier table fixes limits, features … per bucket; `WEBGL_debug_renderer_info` returns the bucket name"; GR-4 runs the full Khronos WebGL suites. Missing: the exposed extension list per bucket (GR-D11's tier table is written for WebGPU). The mapper searched "WEBGL_" and missed this |
| CG-11.13 | Shape Detection | Missing | Held (rule) | Single-engine (= WA-16.21) |
| CG-13.8 | H.266 / VVC | Missing | **Actually excluded** | MD 9.4 MD-D1 (a) fixes a closed codec set (VVC not in it) with an add trigger: "A format is shipped by two other engines and used on ≥ 0.1%† of corpus pages: add it". No engine ships VVC. Naming it in MD 6.2's "Not exposed" list is a record tidy-up, not work |
| CG-13.19 | AC-3, E-AC-3, Dolby Vision | Missing | **Actually excluded** (needs a record) | Outside MD-D1 (a)'s closed set, so not exposed. But Chromium partial plus WebKit could meet MD-D1's add trigger, and no licence or OS-decoder route is recorded (MD-D3 names only H.264, AAC and HEVC). The licence row stays open |
| CG-13.20 | ALAC | Missing | **Actually excluded** | Outside MD-D1 (a)'s closed set; WebKit-only through the OS. MD 5: "OS-only formats make the web differ by OS" |
| CG-14.16 | Remote Playback | Missing | Confirmed missing | No hits (Cast, AirPlay, `disableRemotePlayback`); MD 6.20 has no row. Two engines |
| CG-14.17 | Presentation API | Missing | Held (rule) | Single-engine (= WA-16.22) |
| CG-14.19 | Audio Session | Missing | Held (rule) | No hits. WebKit-only |
| CG-15.2 | Media Capture extensions | Missing | Confirmed missing | MD 6.10 and 6.20 cover Media Capture and Streams only; no blur, framing, PTZ or voice-isolation hits. Partial in three engines |
| CG-15.3 | Image Capture | Missing | Held (rule) | No hits. Chromium-only |
| CG-15.6 | Capture Handle | Missing | Held (rule) | No hits. Chromium-only |
| CG-15.9 | Captured Surface Control, captured mouse events | Missing | Held (rule) | No `CaptureController` hits. Chromium-only |
| CG-15.12 | Content hints | Missing | Confirmed missing | No `contentHint` or `degradationPreference` hits. Two engines |
| CG-15.13 | HTML Media Capture | Missing | Confirmed missing | As WA-3.9 |
| CG-15.18 | WebRTC extensions, SVC, priority | Missing | Confirmed missing | MD 6.10 and MD-D30 name codecs, jitter buffers and statistics; no `scalabilityMode`, simulcast, DSCP or `RTCIceTransport` hits |
| CG-15.19 | WebRTC Identity | Missing | Held (rule) | Gecko-only, effectively dead |
| CG-16.1 | WebXR Device API | Missing | Confirmed missing | Only HGBA-2 9.1 (a vr.org citation) and HGBA-2 8's sensor prose; HGBA-2 8 non-goals do not name XR. Chromium plus visionOS WebKit could meet the two-engine bar, so the rule does not clearly hold it |
| CG-16.2 | WebXR modules | Missing | Confirmed missing | As CG-16.1 |

## Summary

| Verdict | WA | CG | Total |
|---|---|---|---|
| Confirmed missing | 19 | 8 | **27** |
| Actually covered | 2 | 0 | **2** |
| Actually partial | 1 | 3 | **4** |
| Actually excluded (explicit text) | 3 | 3 | **6** |
| Held by a recorded admission rule | 52 | 10 | **62** |
| **Total** | **77** | **24** | **101** |

Of the 62 held rows, 60 rest on DM 6.3, DM 9.1 and AH 6.15 and 2 on ST-D18 (CG-5.7, CG-6.5). On the stricter reading, where a holdover needs a per-feature ledger entry, those 62 stay Missing and the Missing total is 89, not 27. The 12 changes based on spec text (2 covered, 4 partial, 6 excluded) hold under either reading.

**Changed on evidence (12):**
- WA-1.18 Out-of-order HTML streaming → Covered (ST 9.1, `<template for>`, "No hold").
- WA-2.14 Processing instructions → Covered (ST 9.1, ST 6.5, ST 11.2).
- WA-12.10 Web Speech → Partial (MD 6.20 assigns it to "a separate spec", which does not exist).
- WA-16.4 Battery Status → Excluded (FP 6.15 "Device, battery or pressure state | Never").
- WA-16.6 Compute Pressure → Excluded, weak (same FP 6.15 row).
- WA-17.14 Memory and profiling → Excluded (FP-D10 refuses `measureUserAgentSpecificMemory` with a trigger; JS 6.10 constrains any profiling API).
- CG-1.9 `@namespace` → Partial (Selectors 4 and CSSOM 1 are anchored; css-namespaces-3 is not).
- CG-4.28 `shape()` → Partial (LY-2 and LY-D10 bind Interop 2026 areas; no design).
- CG-11.9 WebGL extensions → Partial (GR 6.20 `WEBGL_debug_renderer_info` and per-bucket features; GR-4).
- CG-13.8 VVC → Excluded (MD-D1 closed set and add trigger).
- CG-13.19 AC-3, E-AC-3, Dolby Vision → Excluded under MD-D1, but the licence route still needs a record.
- CG-13.20 ALAC → Excluded (MD-D1 closed set; OS-only).

**Changed on the admission rule (62):** WA-2.3, 2.16, 3.10, 3.13, 4.11, 4.16, 5.11, 5.13, 6.23, 8.9, 8.20, 9.10, 10.6, 10.9, 10.11, 11.2, 11.3, 11.5, 11.7, 11.8, 11.11, 11.12, 12.3, 12.7, 12.8, 13.5, 13.13, 14.5–14.12, 15.3, 15.5, 15.6, 15.7, 15.10, 15.11, 16.3, 16.7, 16.11, 16.19, 16.20, 16.21, 16.22, 16.23, 16.24, 17.13, 17.18 (52); CG-5.7, 6.5 (ST-D18), CG-9.10, 11.13, 14.17, 14.19, 15.3, 15.6, 15.9, 15.19 (10).

**Two mapper statements to correct:**
- `map_css_graphics_media.md` says "No admission rule covers non-CSS web APIs in general". DM 6.3 and DM 9.1 do, and AH 6.15 reads the rule as applying to any web-facing feature.
- `map_web_platform_apis.md` defines Excluded as including "held by a recorded rule" but files the [1E] rows as Missing. Either the status key or those rows should change.

**Confirmed missing and multi-engine (the real work list):** WA-3.8, 3.9, 4.15, 5.9, 5.12, 8.16, 9.11, 11.1, 11.6, 12.6, 15.1, 15.4, 16.2, 16.9, 16.10, 17.6 (Baseline or two to three engines); WA-22.10 and 22.12 (process gaps); WA-12.12 (UA policy); CG-1.2 (process gap); CG-14.16, 15.2, 15.12, 15.13, 15.18, 16.1, 16.2.
