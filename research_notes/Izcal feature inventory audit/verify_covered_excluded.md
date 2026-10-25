# Izcal feature inventory audit: check of Covered and Excluded verdicts

Checked 24 October 2026 against `reports/` on the `feature-audit` worktree (the maps were compiled at 217e02d).

**What was checked.**
- **Covered.** 25 Covered rows from each of the four maps, 100 in all. About 18 per map were chosen because their evidence was thin: a scoping statement, a claim by reference, a backlog line, a single mention, or an anchor-table entry. About 7 per map were taken at random, as every Nth Covered row. For each row the cited spec section was opened and its quotes grepped.
- **Excluded and Deferred.** Every row in all four maps (127 rows) was checked against its cited decision, non-goal or admission rule.
- **Superseded text.** Each cited spec's revision header and decision log were checked for reversed or superseded decisions.

**Verdicts.**
- **Covered rows:** *Confirmed covered*, *Only partial* (the spec names or scopes the feature but does not state its behaviour; the missing part is given), or *Not there* (wrong citation, or the text is absent).
- **Excluded rows:** *Confirmed*, or *Misread* (with the reason). *Confirmed (deferral)* means the citation is right, but it defers the feature rather than refusing it. *Confirmed-implicit* means a general rule covers the feature without naming it. *Confirmed as hold, wrong status* means an admission rule holds the feature, so the row should be Deferred (admission), not Excluded.

**Standard.** The strict standard is the one the BF, PO and WA maps state: the spec states the behaviour, not only the name. The CSS map defines a looser rule, under which an anchor table, a WPT floor or a milestone exit counts as Covered. CSS rows are graded by the strict standard, and the table notes when the map's own rule would pass them.

A checker's first-pass reading was re-verified by hand for every verdict that changes a status. Three first-pass readings were corrected:

| Row | First-pass reading | Hand check | Result |
|---|---|---|---|
| WA-11.10 | Quote absent | The quote exists in `Izcal devices.md` 6.3 (WebUSB, line 385) | Now Confirmed |
| CG-6.18 | Wrong citation | LY 6.5.2 does list "ruby layout (Ruby 1)" | Citation stands; row stays Only partial |
| CG-1.21 | `speak` absent from ST | `speak` is in ST 6.16 | Re-graded as an admission hold |

---

## 1. Covered rows (100)

### 1.1 Browser product features (`map_browser_product_features.md`)

| Row | Feature | Map's citation | Sample | Verdict | Note |
|---|---|---|---|---|---|
| BF-1.3 | Pinned tabs | Shell 6.8, SH-D27 | thin | **Only partial** | Shell 6.8 says only "Pinned tabs are Stage 1", plus a `pinned state` field in the tab and session records. What pinning does is not specified: position, width, close protection, where links open. |
| BF-1.12 | Mute tab or site | Shell 6.3, 6.8 | thin | **Only partial** | There is a `mute` operation and a `muted state`. Site-level mute and an audible-tab indicator are absent. |
| BF-5.4 | Save page | S1 §6 "Save for offline"; PH 6.4.10 | thin | **Only partial** | S1 §6 designs the Stage 2 offline snapshot only. No spec designs saving a page to a file (Ctrl+S, the file dialog, the format). PH 6.4.10 assumes the feature exists. |
| BF-5.6 | Save as PDF | PD 6.19, PD-D19–21 | thin | Confirmed covered | The writer is designed. The product lands in Stage 2. |
| BF-7.3 | Private window over Tor | NW 6.13, NW-D6; S1 §6 | thin | **Only partial** | NW 6.13 designs Tor routes per container (an external Tor client, Stage 2). No spec opens a private window on a route. |
| BF-7.12 | Cookie banner handling | CB 6.15, CB-D22; S1 §6 | thin | Confirmed covered | The behaviour is in CB 6.15: hide first, no clicks. The evidence quote is S1 §6's "rule-based auto-reject", which CB-D22 replaced (see §3). |
| BF-9.3 | Quieter notification prompts | Shell 6.4 | thin | Confirmed covered | The quote is present. Lands in Stage 2. |
| BF-11.1 | Picture-in-picture | MD 6.12; UI 10; S1 13.2 | thin | Confirmed covered | MD 6.12 designs both video PiP and Document PiP. |
| BF-11.5 | Live captions | ML-D2, MD-D25 | thin | Confirmed covered | ML-ASR is specified. Open question Q-ML-1 may put captions under the AI grant. |
| BF-12.2 | Always-keep-active list | FP-D15; UI 10 | thin | Confirmed covered | A per-site control that is exempt from discard. There is no list view. |
| BF-13.2 | Extension store | EX-D10 | thin | Confirmed covered | EX 6.12 designs the directory, review and distribution. |
| BF-13.4 | Cross-ecosystem extensions | EX-D1, EX-D4 | thin | Confirmed covered | EX 6.1 Ring A. |
| BF-14.9 | Dictation | AX 6.9; ED-D5 | thin | Confirmed covered | The OS hands dictation over as `insertReplacementText`, and ED-4's test matrix includes it. |
| BF-18.7 | Beta / preview channels | S1 7.8; shell 6.9 | thin | **Only partial** | One line ("Nightly, Beta and Release channels") and an "Update channel" settings row. Channel switching, downgrade, profile compatibility and side-by-side installs are not specified. |
| BF-21.4 | Offline page saving | S1 §6 | thin | Confirmed covered | The table cell states the behaviour. There is no surface that lists saved pages. |
| BF-6.6 | Password manager unlock | Shell 6.12 | thin | Confirmed covered | OS authentication before a password is revealed or exported. |
| BF-12.8 | Hardware acceleration toggle | Shell 6.14, SH-D22 | thin | **Only partial** | The only control is the recovery window's "Start without hardware acceleration". There is no Settings row. |
| BF-11.9 | DRM playback | D-S7, S1 13.1a, MD-D32 | thin | Confirmed covered | S1 13.1a is designed and current. |
| BF-1.1 | Multiple windows | Shell 6.3 | random | Confirmed covered | |
| BF-2.10 | Security indicator | Shell 6.8, SH-D14, UI 4.1–4.2 | random | Confirmed covered | UI 4.1 is the rewritten prompt-layer text, not the withdrawn version. |
| BF-5.3 | Dangerous downloads | Shell 6.10, SH-D18 | random | Confirmed covered | |
| BF-7.10 | Per-site shields | CB 6.12, CB-D19 | random | Confirmed covered | |
| BF-9.6 | Device choosers | DV; shell 6.4 | random | Confirmed covered | The chooser text is in the shell spec, not DV. |
| BF-13.13 | Keyboard shortcuts | Shell 6.16 | random | Confirmed covered | |
| BF-19.2 | First-run onboarding | Shell 6.14, SH-D23 | random | Confirmed covered | |

### 1.2 OS integration and operations (`map_os_integration_operations.md`)

| Row | Feature | Map's citation | Sample | Verdict | Note |
|---|---|---|---|---|---|
| PO-1.9 | Tarball / portable build | S1 13.2 "Packaging" | thin | **Only partial** | Says only "a tarball on Linux". Not specified: where the profile lives, running from any folder, how the TUF updater updates an unpacked install. |
| PO-2.4 | macOS notarisation and stapling | S1 13.2; SEC-16 (a) | thin | Confirmed covered | Stapling is not named, which is minor. |
| PO-2.5 | macOS JIT entitlement | SEC 6.7; JS 6.8 | thin | Confirmed covered | |
| PO-3.2 | Elevation-free Windows updates | SEC-D14 | thin | Confirmed covered | |
| PO-3.4 | [SB] Release channels | S1 7.8 | thin | **Only partial** | One line in S1 7.8 and one settings row. Not specified: promotion, cadence, switching to an older channel, profiles. This is the same gap as PO-1.10, which the map marks Partial. |
| PO-3.15 | Supported-OS policy and end of support | GR-D21 | thin | **Only partial** | The GR-D21 rule exists. The warning on a dropped OS and keeping the last build available are absent, and the map's own evidence says so. |
| PO-4.3 | Root program governance | NW 6.5.2 | thin | Confirmed covered | |
| PO-4.7 | Public Suffix List | H2 2.3.2; S1 13.2; NW-D7 | thin | Confirmed covered | The update path is not stated. |
| PO-5.10 | Local feature flags page | SH 6.9 | thin | Confirmed covered | |
| PO-5.11 | Troubleshooting pages | SH 6.14; SEC 6.7 | thin | Confirmed covered | |
| PO-6.13 | Lockable preferences | SH 6.9; H3 B10; ML-D4 | thin | **Only partial** | How a locked row is displayed is specified. Which settings can be locked is not, and no policy delivery exists (PO-6.1 to 6.6 are Missing). |
| PO-6.18 | Native messaging hosts | EX-D9 (EX 6.11) | thin | Confirmed covered | Lands in Stage 2. The map handles this inconsistently with PO-17.4 (§2.2). |
| PO-7.11 | ECH controls | NW 6.5; NW 6.14 | thin | Confirmed covered | |
| PO-9.12 | OS biometric re-authentication | SH 6.12 | thin | Confirmed covered | |
| PO-10.6 | Codec patent licensing | S1 13.2 "Codecs" | thin | Confirmed covered | |
| PO-11.5 | Linux ARM64 | FP-25; AX lab; JS ISAs | thin | **Only partial** | FP-25 measures size "per architecture" and never names Linux. The coding standard's target list has no `aarch64-unknown-linux-gnu`. PO-11.2 has the same evidence and the map marks it Partial. |
| PO-13.6 | Remote automation switch | DT 6.3 | thin | Confirmed covered | |
| PO-15.8 | Encryption export classification | S1 13.2 | thin | Confirmed covered | |
| PO-4.1 | Component update channel | SEC UPD-R1; NW-D17 | random | Confirmed covered | |
| PO-5.1 | Crash capture | S1 1, D-S4 | random | Confirmed covered | |
| PO-7.8 | Enterprise and local roots | NW 6.5 | random | Confirmed covered | |
| PO-10.8 | Power management and QoS | H2 4.3; FP 6.7/6.8; GR 6.9 | random | Confirmed covered | |
| PO-13.3 | WebDriver classic | DT 6.3; DT-5 | random | Confirmed covered | |
| PO-17.5 | Safe-browsing service | PH | random | Confirmed covered | |
| PO-9.13 | OS spellchecker | ED-D8 | random | **Only partial (wrong stage)** | ED-D8 and ED 6.8 were "Staged to Stage 2" by the 21 October owner decision. The map says Stage 1. |

### 1.3 Web platform APIs (`map_web_platform_apis.md`)

| Row | Feature | Map's citation | Sample | Verdict | Note |
|---|---|---|---|---|---|
| WA-1.10 | `ancestorOrigins` | HTML claim; DM 6.17 | thin | Confirmed covered | The claim text exists (DM 6.3, DM-D15), and the feature is in WHATWG HTML. |
| WA-1.11 | Origin API | HTML claim | thin | **Only partial** | Named nowhere and has no milestone. The interface is a recent HTML addition, and no report records the HTML pin date, so whether the claim reaches it cannot be checked. |
| WA-1.14 | Initial `about:blank` | DM 6.9; HTML claim | thin | Confirmed covered | |
| WA-1.16 | Simple dialogs | DM 6.7, DM-D15 | thin | **Only partial** | DM hands drawing to the broker and shell. The prompt-class table in shell 6.4 has no class for `alert`, `confirm`, `prompt` or `beforeunload`. Surface, focus and origin labelling are not specified. |
| WA-3.6 | `showPicker()` | HTML claim; LY-D11 | thin | **Only partial** | LY 6.3 says pickers "are browser-process UI". No shell or UI spec owns the select, date or colour pickers. |
| WA-3.11 | Colour input `alpha`/`colorspace` | HTML claim; LY-D11 | thin | **Only partial** | Same unowned picker. Also depends on the HTML pin. |
| WA-4.7 | `CloseWatcher` | HTML claim; DM 6.12 | thin | Confirmed covered | |
| WA-5.3 | JSON and CSS module scripts | JS 6.14; HTML claim | thin | Confirmed covered | |
| WA-6.3 | Aborting | DOM claim; DM 4.2 | thin | Confirmed covered | |
| WA-7.6 | Error reporting | HTML claim; DM 6.7 | thin | Confirmed covered | |
| WA-10.2 | Module service workers | DM 6.3 corpus | thin | Confirmed covered | |
| WA-13.9 | Secure contexts | DM 6.3; HTML claim | thin | Confirmed covered | |
| WA-16.5 | Device Memory | DM 6.16 | thin | **Only partial** | Its only mention is a fingerprinting row. It is a separate W3C spec, outside the DM 6.3 corpus, and ships in Chromium only, with no DM-D15 exception. By the map's own rule it should be Partial [1E]. |
| WA-18.18 | Upsert | JS-1/JS-70 | thin | **Only partial** | Not named. Covered only if it is at Stage 4 at the pinned commit, which no report records. |
| WA-18.20 | Uint8Array base64/hex | JS-1/JS-70 | thin | Confirmed covered | In ES2026 (JS-M2). |
| WA-18.23 | JSON.parse source text | JS-1/JS-70 | thin | **Only partial** | Not named, and its stage is unconfirmed. |
| WA-20.11 | Resizable Wasm buffers | JS-3 | thin | **Only partial** | Not named. JS-3 holds `wasm/jsapi/` only to per-directory floors, and the map itself makes coverage conditional. |
| WA-22.7 | Infra | DM-D21 | thin | **Not there** | DM-D21 is the IDL-inputs decision. Infra is named in no report. Low impact. |
| WA-1.3 | Navigation API | DM 6.9 | random | Confirmed covered | |
| WA-2.11 | `hidden="until-found"` | ED 6.12 | random | Confirmed covered | |
| WA-4.12 | User activation | DM 6.5 (DM-D20) | random | Confirmed covered | |
| WA-7.8 | Shared workers | DM 6.10, DM-D8 | random | Confirmed covered | |
| WA-9.8 | OPFS | SG-D9, SG 6.7 | random | Confirmed covered | |
| WA-13.12 | Local Network Access | NW 6.1, 6.7, 6.11 | random | Confirmed covered | The grant prompt is not in shell 6.4's examples. |
| WA-17.15 | Reporting API | NW 6.11 | random | Confirmed covered | |

### 1.4 CSS, graphics and media (`map_css_graphics_media.md`)

"Loose rule" means the map's own definition of Covered. All twelve Only partial rows below pass it.

| Row | Feature | Map's citation | Sample | Verdict (strict) | Note |
|---|---|---|---|---|---|
| CG-1.14 | Media pseudo-classes | ST 6.7.1 | thin | **Only partial** | A generic state-source mechanism. No pseudo-class is named, no media-spec writer exists, and the source table is a future ST-M0 deliverable. |
| CG-2.11 | `@supports selector()` | ST-scope | thin | **Only partial** | Only "Conditional 5" in the anchor table and the `css-conditional` floor. How `selector()` is evaluated is not designed. |
| CG-2.12 | `font-tech()` | ST-scope | thin | **Only partial** | Zero hits for `font-tech`. |
| CG-3.4 | `contrast-color()` | GR-scope, MD 6.20 | thin | **Only partial** | Zero hits. The Color 5 anchor has no floor percentage. |
| CG-11.2 | Path2D | GR-scope | thin | **Only partial** | Zero hits. GR 6.15 has no path objects. |
| CG-12.16 | Image density correction | MD 6.6 | thin | **Only partial** | One clause. Which Exif fields and rules apply is not stated, and there is no WPT floor. |
| CG-2.1 | Values 3 | ST-scope, ST 6.9 | thin | Confirmed covered | |
| CG-4.11 | Box Alignment 3 | LY anchor | thin | **Only partial** | An anchor entry and the `css-align` floor. No section designs alignment beyond abspos self-alignment. |
| CG-6.8 | Fonts 5 | TX-scope | thin | Confirmed covered | TX 6.4 and 6.8 name `size-adjust`, the metric overrides and `font-size-adjust`. |
| CG-6.19 | Lists and counters 3 | LY anchor, staffing | thin | **Only partial** | Counter scoping (reset, set, increment) and marker generation are not designed. `css-lists` is not among the LY-1 floors. |
| CG-7.15 | `backdrop-filter` | GR 6.3, 6.7 | thin | Confirmed covered | |
| CG-3.9 | `corner-shape` | GR-D17, LY 6.14 | thin | **Only partial** | Outline and hit-testing only. Border and background painting of shaped corners is not designed. Single-engine. |
| CG-1.17 | Shadow DOM styling | ST 6.9, 6.7 | thin | Confirmed covered | |
| CG-1.19 | Registered custom properties | ST 6.9 | thin | Confirmed covered | |
| CG-6.18 | Ruby layout | LY 6.5.2, anchor | thin | **Only partial** | LY 6.5.2 lists "ruby layout (Ruby 1)" as one item, so the citation is right. Annotation placement, `ruby-position` and overhang are not designed. |
| CG-3.12 | Images 4 | GR-scope, SV 6.8 | thin | Confirmed covered | |
| CG-14.22 | ImageDecoder | MD 6.20 | thin | **Only partial** | One table cell. Frame and track access and streaming input are not designed. |
| CG-14.7 | MSE in workers, ManagedMediaSource | MD 6.7, 6.20 | thin | **Only partial** | ManagedMediaSource is designed. MSE in workers and `MediaSourceHandle` are not. |
| CG-1.3 | CSS Syntax 3 | ST 6.6 | random | Confirmed covered | |
| CG-4.7 | Flexbox | LY 6.5.3 | random | Confirmed covered | |
| CG-6.10 | WOFF and OpenType | TX 6.2 | random | Confirmed covered | |
| CG-10.3 | SMIL | AN 6.8, SV 6.9 | random | Confirmed covered | |
| CG-12.5 | WebP | MD 6.2 | random | Confirmed covered | |
| CG-13.14 | MP3 | MD 6.2 | random | Confirmed covered | |
| CG-14.18 | Audio Output Devices | MD 6.8 | random | Confirmed covered | |

---

## 2. Excluded and Deferred rows (127)

### 2.1 Browser product features (38 rows, all with status Excluded)

Sixteen rows have status Excluded but "Deferred" in their Lands-in column: BF-1.4, 1.7, 1.8, 2.7, 5.5, 5.10, 6.8, 6.9, 10.4, 10.6, 13.18, 15.1, 15.3, 15.8, 16.6 and 17.5. The map's own rules make these Deferred (Stage N), or Covered/Partial where a Stage 2 design exists.

| Row | Feature | Cited decision | Verdict | Note |
|---|---|---|---|---|
| BF-1.4 | Tab groups | Shell 9.1, SH-D27, Q-SH-9 | Confirmed (deferral) | Stage 2. |
| BF-1.5 | Saved/synced groups | Shell 9.1, Q-SH-5 | Confirmed-implicit | Saving a group is neither designed nor deferred. |
| BF-1.6 | Two-level tab stacks | UI §8, UP5 | Confirmed-implicit | |
| BF-1.7 | Vertical tabs | SH-D27, UI R18 | Confirmed (deferral) | Spec conflict: S1 §6's out-of-scope list against SH-D27 "Stage 2 table stakes". |
| BF-1.8 | Split view | SH-D27 | Confirmed (deferral) | Same conflict as BF-1.7. |
| BF-1.9 | Spaces | S1 §6 | Confirmed | Explicit. S1 §6 names "spaces", although the map says it does not. |
| BF-1.23 | Tabs from other devices | Shell 9.1, SH-D20 | Confirmed | |
| BF-2.7 | Command palette | SH-D27, Q-SH-8 | Confirmed (deferral) | A Stage 2 trial. |
| BF-2.12 | AI search mode | Shell 6.8, SH-D16 | Confirmed-implicit | Refused only for the AI-off case. |
| BF-2.13 | Sponsored suggestions | Shell 6.8, 6.7 | Confirmed | |
| BF-3.9 | News feed on new tab | Shell 6.7, SH-D29 | Confirmed-implicit | |
| BF-3.10 | Start-page widgets | Shell 6.7, SH-D29 | **Misread** | Only networked widgets are refused. Local widgets are neither refused nor designed, so the row is Missing. |
| BF-4.3 | Sign-in / account | SH-D20 | Confirmed | |
| BF-4.4 | Sync | Shell 9.1, SH-D20, Q-SH-5 | Confirmed (deferral) | Still an open owner question. |
| BF-4.5 | E2E encrypted sync | As BF-4.4 | Confirmed-implicit | |
| BF-4.6 | Send tab to device | Shell 9.1 | Confirmed | |
| BF-4.7 | Send files between devices | Shell 9.1 | Confirmed | |
| BF-4.9 | Sync troubleshooting | As BF-4.4 | Confirmed-implicit | |
| BF-5.5 | Print and preview | S1 §6, shell 6.19, GR 6.27 | **Misread** | Paged layout, `@page`, `window.print()` and the preview raster are Stage 1 designs (LY 6.7, GR 6.27). Only the print UI is deferred, so the row is Partial. |
| BF-5.10 | PDF summarise | ML-D2 | **Misread** | ML-D2 defers the web Summarizer API, not a product feature. The row is Missing. |
| BF-6.3 | Breach / weak password alerts | Shell 6.12 | **Misread** | "no breach-check service in Stage 1" scopes Stage 1; it is not a refusal, and weak or reused passwords are never mentioned. The row is Missing. |
| BF-6.8 | Address autofill | S1 13.2 | Confirmed (deferral) | |
| BF-6.9 | Payment autofill | Shell 6.9 | Confirmed (deferral) | |
| BF-7.17 | IP masking in private mode | S1 §6 | Confirmed | |
| BF-10.4 | Reader mode | UI R18, §6 | Confirmed (deferral) | |
| BF-10.6 | Page translation | ML-D2 | **Misread** | ML-D2 defers the web Translator API. No product decision exists. The row is Missing. |
| BF-10.8 | Cloud spellcheck | ED §4.4, ED-D7 | Confirmed | |
| BF-12.4 | Performance alerts | Shell 6.8, UP2 | Confirmed-implicit | |
| BF-13.5 | MV2 | EX-D2 | Confirmed | |
| BF-13.18 | Language settings | Shell 6.9 | **Misread** | Shell 6.9 designs the group for Stage 2, so the row is Covered or Partial (Stage 2). |
| BF-15.1 | AI assistant panel | UI §6 | Confirmed (deferral) | Stage 3. |
| BF-15.3 | Page summarisation | ML-D2 | **Misread** | Same as BF-10.6. Missing or Partial. |
| BF-15.8 | Writing help | ML-D2, ED-D9 | Confirmed (deferral) | |
| BF-16.6 | Feed reader | S1 §9 | Confirmed (deferral) | |
| BF-16.7 | Built-in VPN | S1 §6 | Confirmed | |
| BF-17.5 | OS notifications | SH-D17, Q-SH-10 | **Misread** | Shell 6.13 partly designs Stage 2 delivery (OS channel, mute, snooze, digests). The row is Partial (Stage 2). |
| BF-17.11 | iOS app installs | S1 13.2 | Confirmed-implicit | A platform deferral. |
| BF-19.8 | Lost-device lock and erase | SH-D20 | Confirmed-implicit | |

### 2.2 OS integration and operations (16 rows)

| Row | Feature | Map status | Cited decision | Verdict | Note |
|---|---|---|---|---|---|
| PO-2.10 | iOS engine entitlement | Excluded | S1 13.2 "iOS" | Confirmed | |
| PO-4.11 | Codec plug-ins | Excluded | S1 13.2 "Codecs" | Confirmed | |
| PO-7.15 | Browser VPN / proxy | Excluded | S1 6; NW-D6 | Confirmed (weak) | A statement in a design row, not a decision ID. |
| PO-11.11 | iOS/iPadOS | Excluded | S1 13.2 "iOS" | Confirmed | |
| PO-14.1 | [SB] UI localisation | Excluded | S1 13.2; SH-D24; SH 4.4 | **Misread** | SH 4.4 rejects "English-only forever", and strings are externalized. SH-D24 is the accessibility decision. Translation is planned, with no stage, so the row is Partial or Deferred. |
| PO-1.7 | Flatpak | Deferred (Stage 2) | S1 13.2 | Confirmed | |
| PO-9.7 | OS notifications | Deferred (Stage 2) | SH-D17 | Confirmed | |
| PO-9.14 | OS printing | Deferred (Stage 2) | S1 13.2; GR 6.27 | Confirmed | |
| PO-9.15 | Save as PDF | Deferred (Stage 2) | S1 6; PDF | Confirmed | |
| PO-11.10 | Android | Deferred (Stage 2) | H2 S-8; S1 | Confirmed | |
| PO-13.1 | Embedding API | Deferred (Stage 2) | S1 13.2 | Confirmed | |
| PO-13.2 | Embedded runtime distribution | Deferred (Stage 2) | S1 13.2 | **Misread** | Only the API is deferred. A shared runtime is never mentioned, so the row is Missing. |
| PO-16.2 | Bug bounty | Deferred (Stage 2) | S1 13.2; SEC-D10 | Confirmed | |
| PO-17.2 | Web Push service | Deferred (Stage 2) | S1 13.2; NW 6.14 | Confirmed | |
| PO-17.3 | Sync service | Deferred (Stage 3) | S1 9; Q-SH-5 | **Misread** | "Stage 3 or never" with Q-SH-5 open is an open owner question, not a deferral. |
| PO-17.4 | Extension store and signing | Deferred (Stage 2) | EX-D10 | **Misread** | Designed in EX 6.12 and EX-D10, so the row is Covered (Stage 2), as PO-6.18 is. |

### 2.3 Web platform APIs (30 rows)

| Row | Feature | Cited decision | Verdict | Note |
|---|---|---|---|---|
| WA-1.19 | XSLT | ST-D4; DM-D15 | Confirmed | |
| WA-2.15 | Permission elements | S1 13.2; SHUI 4.4 | Confirmed | |
| WA-4.6 | Interest invokers | AX-O13 (ST-D18) | **Misread** | AX-O13 only maps the feature "while admitted". ST-D18 is the CSS admission rule. No decision holds `interestfor`. |
| WA-5.4 | Text module imports | JS 6.14; JS-1, JS-70 | **Misread** | JS 6.14 lists text modules in the designed loader surface and records no hold for them. |
| WA-6.20 | Ink API | GR (LY-D10 process) | Confirmed | |
| WA-7.5 | `isInputPending` | DM 6.7; DM-D15 | Confirmed | |
| WA-8.11 | Direct Sockets | S1 §6 | Confirmed | |
| WA-9.5 | Storage Buckets | SG-D4 | Confirmed | |
| WA-9.12 | Removed storage | DM-D15 | Confirmed | |
| WA-10.5 | Push API | S1 13.2; NW 6.14 | Confirmed | |
| WA-10.7 | Notifications | SH-D17 | Confirmed | |
| WA-10.8 | Background Sync | S1 13.2 | Confirmed | |
| WA-10.10 | Background Fetch | NW 6.11 | Confirmed | |
| WA-11.9 | Document PiP | S1 13.2; SHUI | **Misread** | Designed in MD 6.12 (line 468), listed in MD 6.20, and part of MD-M2. The row is Covered, with a staging conflict against S1 13.2's "Stage 2". |
| WA-11.10 | Isolated Web Apps | DV 6.3, 6.10; S1 §6 | Confirmed | The quote is in DV 6.3 (line 385). S1 §6 does not mention IWAs, which is a minor citation slip. |
| WA-12.9 | Local Font Access | TX-D17 | Confirmed | |
| WA-12.11 | VirtualKeyboard API | ED-D18 | Confirmed | |
| WA-14.2 | `requestStorageAccessFor` / RWS | NW 6.9 | Confirmed | |
| WA-14.16 | IP Protection | S1 §6 | **Misread** | S1 §6 refuses a browser-operated VPN, which is a different thing. The refusal is implicit only. |
| WA-17.16 | Network Error Logging | NW-D15 | Confirmed | |
| WA-18.17 | Explicit resource management | JS-1, JS-70 | **Misread** | Not named. No hold is recorded, and the reports state no uniform "until Stage 4" rule. |
| WA-18.24 | `Promise.allKeyed` | JS-1 | **Misread** | Same as WA-18.17. |
| WA-18.25 | Source phase / `import defer` | JS 6.14; Q-JS-12 | **Misread** | JS 6.14 designs both. Q-JS-12 is an open question, not a hold. |
| WA-18.26 | Unshipped proposals | JS 6.14; JS 9.1 | Confirmed | Decorators and ShadowRealm are named. The rest are implicit. |
| WA-20.12 | Newer Wasm proposals | JS 9.1 | Confirmed | Partly implicit. |
| WA-21.1 | Translator / Language Detector | ML-D2; H2 4.5 | Confirmed | |
| WA-21.2 | Summarizer, Writer and related APIs | ML-D2 | Confirmed | |
| WA-21.3 | Prompt API | ML-D2; H3 B10 | Confirmed | |
| WA-21.4 | WebNN | H2 4.5; ML-D2 | Confirmed | |
| WA-21.5 | WebMCP | S1 §9; H3 B9/B10; DM 6.15 | Confirmed | The "inert surface" text is in H3 B10, not DM 6.15. |

### 2.4 CSS, graphics and media (43 rows)

| Row | Feature | Map status | Cited decision | Verdict | Note |
|---|---|---|---|---|---|
| CG-1.15 | Selectors 5 | Excluded | ST-D18 | Confirmed as hold, wrong status | ST-D18 holds features, it does not refuse them. The row is Deferred (admission). |
| CG-1.21 | CSS Speech | Excluded | ST-D18; AX 6.3 | Confirmed as hold, wrong status | AX 6.3 says "`speak-as` if admitted". `speak` is read in ST 6.16, as the map states. |
| CG-2.6 | `random()` | Excluded | ST-D18 | Confirmed as hold, wrong status | |
| CG-3.13 | `object-view-box` | Excluded | ST-D18/LY-D10 | Confirmed as hold, wrong status | |
| CG-3.15 | Fill and Stroke 3 | Excluded | ST-D18 | Confirmed as hold, wrong status | |
| CG-4.22 | Overflow 5 carousels | Excluded | LY-D10 | Confirmed as hold, wrong status | |
| CG-4.29 | Exclusions | Excluded | LY-D10 | Confirmed as hold, wrong status | |
| CG-4.30 | Regions | Excluded | LY-D10 | Confirmed as hold, wrong status | Named in LY 5's lessons table, although the map says it is not. |
| CG-4.31 | Page floats and others | Excluded | LY-D9; LY-D10 | Confirmed | LY-D9 names page floats. |
| CG-4.32 | CSS Layout API | Excluded | LY-D10 | Confirmed as hold, wrong status | DOM 6.9 still lists a "layout" worklet. |
| CG-5.11 | GCPM | Excluded | LY-D9 | Confirmed | |
| CG-6.17 | Local Font Access | Excluded | TX-D17 | Confirmed | |
| CG-7.10 | Animation Worklet | Excluded | AN-D9 | Confirmed | |
| CG-8.4 | CSS Painting API | Excluded | ST-D18 | Confirmed as hold, wrong status | DOM 6.9 lists a "paint" worklet. |
| CG-8.6 | CSS parser API | Excluded | ST-D18 | Confirmed as hold, wrong status | |
| CG-9.7 | `interactivity: inert` | Excluded | ST-D18 | Confirmed as hold, wrong status | |
| CG-10.5 | SVG fonts | Excluded | SV-D7 | Confirmed | |
| CG-10.8 | MathML beyond Core | Excluded | LY-D21 | Confirmed | The decision exists, but the map's quote appears in no report. |
| CG-12.8 | HEIC | Excluded | MD-D1 | Confirmed | |
| CG-12.9 | TIFF | Excluded | MD-D1 | Confirmed | |
| CG-12.10 | JPEG 2000 | Excluded | MD-D1 | Confirmed | |
| CG-13.5 | MPEG-DASH | Excluded | MD 6.20 | Confirmed | A coverage-table statement, not a decision ID. |
| CG-13.12 | Theora | Excluded | MD-D1 | Confirmed | |
| CG-1.20 | `@function`, `@mixin` | Deferred (admission) | ST 9.1, ST-D18 | Confirmed | |
| CG-2.4 | `if()` | Deferred (admission) | ST 9.1 | Confirmed | |
| CG-2.7 | `progress()`, `*-mix()`, `calc-size()` | Deferred (admission) | LY 6.15, AN-D9 | Confirmed | Partly implicit. |
| CG-2.9 | Link parameters | Deferred (admission) | SV-D13 | Confirmed | |
| CG-2.15 | Scroll-state queries | Deferred (admission) | ST 9.1, LY-D10 | Confirmed | |
| CG-3.10 | Gap decorations | Deferred (admission) | LY-D10 | Confirmed | |
| CG-3.14 | `image-animation` | Deferred (admission) | MD-D19 | Confirmed | It waits on the CSS spec text, not on an admission rule. |
| CG-4.2 | `margin-trim` | Deferred (admission) | LY 6.5.1 | Confirmed | |
| CG-4.6 | `reading-flow` | Deferred (admission) | LY-D10, AX | Confirmed | |
| CG-4.10 | Grid Lanes | Deferred (admission) | LY 6.5.4 | Confirmed | |
| CG-4.13 | Multicol 2 | Deferred (admission) | LY-D10 | Confirmed | LY 6.7 wording ("`column-wrap` and `::column` run in ordinary browsing") conflicts with the hold. |
| CG-4.19 | Anchor positioning 2 | Deferred (admission) | LY-D10 | Confirmed | |
| CG-5.2 | Scroll snap 2 events | Deferred (admission) | LY 6.10 | Confirmed | LY 6.10's "admitted per LY-D10" reads as if already admitted. |
| CG-5.9 | Fragmentation 4 | Deferred (admission) | LY 6.7 | Confirmed | Same wording risk as CG-5.2. |
| CG-7.8 | Web Animations 2 | Deferred (admission) | AN-D9 | Confirmed | |
| CG-9.3 | Customizable `select` | Deferred (admission) | LY 6.3, LY-D10 | Confirmed | |
| CG-9.11 | Delegated ink | Deferred (admission) | GR 6.9, 9.1 | Confirmed | |
| CG-11.6 | HTML-in-canvas | Deferred (admission) | GR 6.15, 9.1 | Confirmed | |
| CG-16.6 | Save as PDF | Deferred (Stage 2) | PD-M4 | Confirmed | |
| CG-16.11 | Reader view | Deferred (Stage 2) | S1 §6, PDF 9.1 | Confirmed (weak) | The feature has no row of its own in any spec. |

---

## 3. Covered rows that cite superseded or reversed text

All four maps were scanned for Covered rows citing reversed or superseded text: the HGBA-3 3.2.x library text, the withdrawn in-page trusted prompt region, HGBA-2 sections superseded at 2.3.5, the retired Stage 1 amendments, A-S1-75, D-S6 (a), PD-D8 (a), C-11, the original form of DT-D12, and SEC 0.2's panic-unwind option. **No Covered row rests on one of them.** Three rows carry stale text or a stale stage:

| Row | Map | What is stale | Effect |
|---|---|---|---|
| BF-7.12 | BF | The evidence quotes S1 §6's "rule-based auto-reject", which CB-D22 replaced with hide-first and no clicks. | The status stands. Cite CB 6.15 instead. |
| PO-9.13 | PO | Lands in "Stage 1", but ED-D8's interim OS spelling tier was "Staged to Stage 2" on 21 October (ED 6.8 line 334). | The stage is wrong. |
| PO-4.12 | PO | Says "Stage 1 (interim OS tier)". Same restaging as PO-9.13. | The stage is wrong. |

**Stale spec text that the maps inherit.** These are not supersessions, but no Covered status depends on them:

| Spec text | What is stale | Map rows that cite it |
|---|---|---|
| S1 13.2 "Text and input" and "Everyday features" | Still say the interim OS tier is Stage 1, against ED-D8 and the batch-4 shell confirmation. | None for the stage |
| S1 13.2 Codecs | Still says platform decoders, against MD-D3; the MD 9.5 amendment is not applied. | CG-13.11, 14.8, 14.9, 14.25, 16.5 cite the parts that are still consistent |
| DT 6.3 | Still calls attach consent the "trusted-region prompt", which is pre-SH-D2 wording. | None depends on the wording |
| GR 6.27 | Still says the PDF writer has "no owner yet", though C-21 is settled. | None depends on it |

---

## 4. Summary

### 4.1 Error rate for Covered rows (sample of 25 per map)

An error is a Covered row that is Only partial or Not there.

| Map | Confirmed | Only partial | Not there | Error rate | Thin rows | Random rows |
|---|---|---|---|---|---|---|
| Browser product features (BF) | 19 | 6 | 0 | **24%** | 6/18 | 0/7 |
| OS integration and operations (PO) | 19 | 6 | 0 | **24%** | 5/18 | 1/7 |
| Web platform APIs (WA) | 16 | 8 | 1 | **36%** | 9/18 | 0/7 |
| CSS, graphics and media (CG), strict | 13 | 12 | 0 | **48%** | 12/18 | 0/7 |
| CSS, graphics and media (CG), map's own rule | 25 | 0 | 0 | 0% | — | — |
| **All four (strict)** | **67** | **32** | **1** | **33%** | 32/72 (44%) | 1/28 (4%) |

All quoted evidence was found, except the one Not there (WA-22.7) and the CG-10.8 quote (an Excluded row). The errors are errors of judgement: a feature that is named, scoped or given a state field was counted as specified. Only one of the 28 random rows was wrong (PO-9.13, a stale stage), so rows with a dedicated design section are sound. The risk is concentrated in rows with thin evidence.

### 4.2 Error rate for Excluded and Deferred rows

| Map | Rows | Confirmed | Misread | Misread rate | Status-label fixes on confirmed rows |
|---|---|---|---|---|---|
| BF | 38 | 30 | 8 | 21% | 16 rows labelled Excluded are deferrals |
| PO | 16 | 12 | 4 | 25% | — |
| WA | 30 | 23 | 7 | 23% | — |
| CG | 43 | 43 | 0 | 0% | 12 Excluded rows are admission holds (Deferred (admission)) |

### 4.3 Every changed verdict

**Covered rows that become Partial (32):**

| Map | Rows |
|---|---|
| BF | BF-1.3 pinned tabs, BF-1.12 mute, BF-5.4 save page, BF-7.3 private window over Tor, BF-12.8 hardware acceleration toggle, BF-18.7 beta channels |
| PO | PO-1.9 tarball, PO-3.4 release channels, PO-3.15 supported-OS policy, PO-6.13 lockable preferences, PO-11.5 Linux ARM64, PO-9.13 OS spellchecker (also restaged to Stage 2) |
| WA | WA-1.11 Origin API, WA-1.16 simple dialogs, WA-3.6 `showPicker()`, WA-3.11 colour input, WA-16.5 Device Memory (Partial [1E]), WA-18.18 Upsert, WA-18.23 JSON.parse source text, WA-20.11 resizable Wasm buffers |
| CG (strict) | CG-1.14, 2.11, 2.12, 3.4, 3.9, 4.11, 6.18, 6.19, 11.2, 12.16, 14.7, 14.22 |

**Covered row that becomes Missing or Not there (1):** WA-22.7 Infra. DM-D21 does not cover it, and no report names it.

**Stage correction on a Covered row (1):** PO-4.12 lands in Stage 2, not Stage 1, for the interim OS spelling tier.

**Excluded or Deferred rows misread (19):**

| Row | New status |
|---|---|
| BF-3.10 | Missing |
| BF-5.5 | Partial (Stage 1 print layout, Stage 2 print UI) |
| BF-5.10 | Missing |
| BF-6.3 | Missing |
| BF-10.6 | Missing |
| BF-13.18 | Covered or Partial (Stage 2) |
| BF-15.3 | Missing or Partial |
| BF-17.5 | Partial (Stage 2) |
| PO-14.1 | Partial or Deferred (unstaged) |
| PO-13.2 | Missing |
| PO-17.3 | Open (Q-SH-5), not Deferred (Stage 3) |
| PO-17.4 | Covered (Stage 2) |
| WA-4.6 | Missing, or Covered under the HTML claim if `interestfor` is in HTML at the pin |
| WA-5.4 | Covered, subject to stage |
| WA-11.9 | Covered (MD 6.12), with a staging conflict |
| WA-14.16 | Missing (implicit refusal only) |
| WA-18.17 | Missing (no recorded hold) |
| WA-18.24 | Missing (no recorded hold) |
| WA-18.25 | Partial (designed in JS 6.14; Q-JS-12 is open) |

**Status relabels on confirmed rows (28):**

| Change | Rows |
|---|---|
| BF, Excluded to Deferred (Stage N), 16 rows. The six of them also misread above take the new status listed there. | BF-1.4, 1.7, 1.8, 2.7, 5.5, 5.10, 6.8, 6.9, 10.4, 10.6, 13.18, 15.1, 15.3, 15.8, 16.6, 17.5 |
| CG, Excluded to Deferred (admission), 12 rows | CG-1.15, 1.21, 2.6, 3.13, 3.15, 4.22, 4.29, 4.30, 4.32, 8.4, 8.6, 9.7 |

**Citation fixes that leave the status unchanged:**

| Row | Fix |
|---|---|
| BF-7.12 | Cite CB 6.15, not S1 §6's superseded phrase. |
| BF-1.9 | S1 §6 names "spaces", so the refusal is explicit. |
| WA-11.10 | Drop S1 §6; the quote is in DV 6.3. |
| WA-21.5 | The inert-surface text is in H3 B10, not DM 6.15. |
| CG-4.30 | Regions is named in LY 5. |
| CG-10.8 | The quote is not in the reports; the decision is LY-D21. |

### 4.4 Is a full re-check needed?

- **CSS, graphics and media: yes, re-check every Covered row under the strict standard.** The map's own Covered rule counts an anchor table, a WPT floor or a milestone exit as coverage. Under the strict standard, 12 of 18 thin rows failed, though every random row passed. With 185 Covered rows, roughly 40 to 60 are probably scope-only. This is a flaw in the rule itself, not scattered mistakes, so it affects the whole map. Relabel the 12 admission holds at the same time.
- **Web platform APIs: targeted re-check of every claim-by-reference, single-mention or "stage assumed" Covered row.** Half of them failed (9 of 18), and all the random rows held up. In the same pass:
  - apply the [1E] rule to Covered rows, as WA-16.5 shows it was not;
  - flag every row that needs an unowned shell surface: pickers, script dialogs, and probably WA-3.2 input types;
  - re-grade every Excluded row that rests on "until Stage 4".
- **Browser product features: a full pass over the Excluded rows** (8 misread, 16 mislabelled). For Covered rows, only a targeted pass over single-mention rows is needed (24% failed, all of them thin).
- **OS integration and operations: targeted only.**
  - Covered rows whose own evidence cell already lists a missing half (PO-3.15, 6.13).
  - Single-line mentions.
  - Every "Lands in" stage after the 21 October restaging of ED-D8.
  - Stage-2-but-designed rows marked Deferred.
  - Likely the same class but outside the sample: PO-10.3 hardware acceleration toggle, the same evidence as BF-12.8.
