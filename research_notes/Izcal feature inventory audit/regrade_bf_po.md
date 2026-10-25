# Izcal feature inventory audit: re-grade of Covered and Excluded rows (BF and PO maps)

Re-graded 24 October 2026 against `reports/` on the `feature-audit` worktree. This pass follows `verify_covered_excluded.md` (the spot-check) and keeps its corrected verdicts, except for the departures listed in §4.

**Scope.** Part 1 covers every Covered row in `map_browser_product_features.md` (94) and `map_os_integration_operations.md` (105). Part 2 covers every Excluded or Deferred row in both maps (38 BF and 16 PO). For each row the cited section was opened, and further searches were made where the citation was thin.

**Grades, Part 1.**
- **Designed:** a spec specifies Izcal's behaviour or architecture for the feature.
- **Owned:** the feature is named as in scope, with an owner and a milestone or target, and needs no Izcal-specific design beyond a standard or OS document (signing, notarisation, licensing, a platform target). It counts as covered.
- **Mentioned:** the feature is named or implied, but its behaviour is not specified and it needs Izcal design. This grade is a gap. It is also used where an owner and a stage are named but the behaviour is not, which is how the spot-check's "Only partial" rows are graded; the evidence cell says what is missing.
- **Not there:** the citation doesn't support the row.

**Grades, Part 2.**
- **Refused:** a decision or non-goal says no. "(implicit)" means a general rule covers the feature without naming it.
- **Deferred:** a later stage, with an owner spec.
- **Deferred, no owner:** a later stage is named, but no spec owns the feature. This is a gap for later.
- **Misread → X:** the citation doesn't refuse or defer the feature. X is the corrected grade (Designed, Owned, Mentioned or Missing).

## 1. Part 1: Covered rows

### 1.1 Browser product features (94 rows)

| # | Feature | Grade | Owner (spec §) | Milestone/stage | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| BF-1.1 | Multiple windows | Designed | Shell 6.3 `shell.window`, 6.8 | Stage 1, SH-M1 | Window list/create/close/bounds and tab moves are broker operations; spot-check confirmed. |
| BF-1.2 | Tab strip basics | Designed | Shell 6.8 "The tab strip" | Stage 1, SH-M1 | Tablist, roving tabindex, Menu-key context menu, close button naming specified. |
| BF-1.3 | Pinned tabs | Mentioned | Shell 6.8, SH-D27 | Stage 1, SH-M1 | Only "Pinned tabs are Stage 1" plus a pinned-state field; behaviour unspecified. |
| BF-1.10 | Tab search | Designed | Shell 6.8 "Recently closed and tab search" | Stage 1, SH-M3 | Searches open tabs by title, URL, container; never sends a query. |
| BF-1.12 | Mute tab or site | Mentioned | Shell 6.3, 6.8 | Stage 1, SH-M1 | Mute operation and state only; site mute and audible indicator absent. |
| BF-1.13 | Reopen closed tab or window | Designed | Shell 6.8 | Stage 1, SH-M3 | Tab, window, group closes recorded, searchable, restore whole set, crash-safe. |
| BF-1.14 | Session restore at startup | Designed | Shell 6.8 "Session store and restore", SH-21 | Stage 1, SH-M3 | Broker writes atomically every 5 s; contents and failure reporting specified. |
| BF-1.16 | Crash recovery | Designed | Shell 6.8, 6.14, SH-21, SH-30 | Stage 1, SH-M3/SH-M4 | "Page stopped" state with Reload; restore ≥99% after crash; recovery window. |
| BF-1.21 | Full screen | Designed | Shell 6.8 "Fullscreen and zoom" | Stage 1, SH-M1 | Hides chrome, exit control at top edge, Esc and F11 always exit. |
| BF-2.1 | Unified address and search bar | Designed | Shell 6.8 "The address bar" | Stage 1, SH-M1 | Broker-produced display string, domain emphasis, search sent only on Enter. |
| BF-2.2 | Suggestions | Designed | Shell 6.8; Q-UI-1 | Stage 1, SH-M1 | Candidate sources, order and provenance label specified. |
| BF-2.10 | Security indicator and URL display | Designed | Shell 6.8, SH-D14; UI 4.1–4.2 | Stage 1, SH-M1/SH-M2 | Neutral "Not secure" chip, no EV, no padlock; spot-check confirmed. |
| BF-3.1 | Bookmarks and bookmark bar | Designed | Shell 6.7 "Surfaces", SH-D8, SH-D9 | Stage 1, SH-M3/SH-M5 | Star control, hidden-by-default bar as toolbar, folders as menus. |
| BF-3.3 | History view and search | Designed | Shell 6.7, SH-19, SH-20 | Stage 1, SH-M3/SH-M5 | Search-first page grouped by day, filters, retention, range/site/container clearing. |
| BF-3.7 | New tab page | Designed | Shell 6.7 "Surfaces" | Stage 1, SH-M5 | Local page: search field, top sites, recently closed; no network on open. |
| BF-4.11 | Export my data | Designed | Shell 6.12 "Export", SH-D20 | Stage 1, SH-M3 (M7) | Versioned `izcal-export` v1 JSON Lines schema with published JSON Schema. |
| BF-5.1 | Downloads UI | Designed | Shell 6.10, 6.3 `shell.downloads`, SH-D18 | Stage 1, SH-M4 | Toolbar bubble, downloads page, pause/resume/cancel/open/show actions. |
| BF-5.2 | Download location and ask-where-to-save | Designed | Shell 6.9 Downloads group | Stage 1, SH-M4 | Settings row: directory, ask where to save, auto-open off by default. |
| BF-5.3 | Dangerous download warnings | Designed | Shell 6.10, SH-D18, SH-R7; PH-D7 | Stage 1 | Two-level warning with reason and confidence; spot-check confirmed. |
| BF-5.4 | Save page | Mentioned | S1 §6 "Save for offline"; PH 6.4.10 | Stage 2 (snapshot only) | Only Stage 2 offline snapshot; Ctrl+S save-to-file, dialog, format undesigned. |
| BF-5.6 | Save as PDF | Designed | PD 6.19, PD-D19–D21, PD-M4 | Stage 2 (writer core Stage 1) | PDF writer designed; print-to-PDF product lands Stage 2; spot-check confirmed. |
| BF-5.7 | Built-in PDF viewer | Designed | PD 6.15, PD-D2 | Stage 1 (M7) | `hg-pdf-viewer` scrolling, zoom, thumbnails, outline, find, print, save specified. |
| BF-5.8 | PDF form fill, annotate and sign | Designed | PD-D12, PD-D17, PD 9.5 | Stage 1 forms/annotate (PD-M3); Stage 2 sign | AcroForm editing, incremental save; S1 §6 staging of annotation conflicts. |
| BF-6.1 | Password manager | Designed | Shell 6.12, 6.3 `shell.password`; S1 §6 (A-S1-51) | Stage 1 | Broker service over OS credential stores; reveal needs fresh OS authentication. |
| BF-6.4 | Passkey manager | Owned | S1 §6 "Passkeys and credentials" (A-S1-51); shell 6.12 | Stage 1 | WebAuthn via platform authenticators; OS keychain manages passkeys, no Izcal list. |
| BF-6.5 | Password and passkey import/export | Designed | Shell 6.12, SH-D19, SH-R2 | Stage 1, SH-M3 (M7) | CSV import with preview; export CSV behind re-auth; passkey export unaddressed. |
| BF-6.6 | Password manager PIN / device unlock | Designed | Shell 6.12; 6.4 Credential class | Stage 1 | OS authentication before reveal or export; spot-check confirmed. |
| BF-7.1 | Private browsing window | Designed | NW 6.1; SG 6.9; shell 6.7 | Stage 1 | Ephemeral containers, memory-only storage, nothing written; window-opening UI unstated. |
| BF-7.3 | Private window over Tor | Mentioned | NW 6.13, NW-D6 | Stage 2 (container routes) | Tor routes per container designed; no spec opens a private window on one. |
| BF-7.4 | Cross-site tracker blocking | Designed | CB (whole spec), CB-D19 | Stage 1 | Six blocking layers and default lists specified. |
| BF-7.5 | Cookie partitioning | Designed | NW 6.9, NW-D7, NW-D16 | Stage 1 | Partitioned by (T, C, X) by default; Storage Access path specified. |
| BF-7.7 | Fingerprinting protection | Designed | D-S1, OD-6; S1 §6 "Anti-fingerprinting" | Stage 1 baseline, Stage 2 full | Noise as output transform; discrete surfaces normalised by construction. |
| BF-7.8 | Tracking-parameter removal | Designed | CB 6.5, CB-D8 | Stage 1 | Parameter list with per-entry evidence and scope; stripping rules specified. |
| BF-7.10 | Per-site shields panel | Designed | CB 6.12, CB-D19; Shell 6.6 | Stage 1, SH-M2 | Graduated per-site controls specified; spot-check confirmed. |
| BF-7.11 | Built-in ad blocking | Designed | CB 6.x, CB-D5 (Ads list), CB-D1 | Stage 1, CB-M3 | Ads list admission criteria designed; on by default from CB-M3. |
| BF-7.12 | Cookie banner handling | Designed | CB 6.15, CB-D22 | Stage 2 | Hide first, no clicks; cite CB 6.15, not superseded S1 §6 text. |
| BF-7.14 | Site data manager | Designed | Shell 6.6 Storage row, SG-D5/SG-D6; Shell 6.9 | Stage 1, SH-M2 | Usage, quota, persist state, clear-site-data stating what remains. |
| BF-7.15 | GPC / Do Not Track | Designed | S1 §6 GPC row; Shell 6.9 Privacy group | Stage 1 | `Sec-GPC: 1` by default; DNT explained as request, not promise. |
| BF-7.16 | Telemetry controls | Designed | D-S4, SH-D29; Shell 6.3, 6.14 | Stage 1 | Opt-in, schema-published telemetry; crash reports opt-out per crash with preview. |
| BF-8.1 | Safe browsing interstitials | Designed | PH-D6, PH-12; Shell 6.4; UI 4.5 | Stage 1 | `security-interstitial` class; proceed per navigation, never persisted. |
| BF-8.4 | HTTPS-only / HTTPS-First | Designed | NW 6.7, NW 6.5.3; Shell 6.9 | Stage 1 | Public sites HTTPS first with interstitial before plain HTTP. |
| BF-8.5 | Secure DNS settings | Designed | NW 6.6, NW-D8; NW 6.14 | Stage 1 | Own resolver, same-provider DoH upgrade, user-chosen resolver; no shell settings row. |
| BF-8.7 | Certificate error interstitials | Designed | Shell 6.4; NW 6.5.3, 6.18; UI 4.5 | Stage 1 | Back to safety, proceed behind Details; HSTS-preload errors unclickable. |
| BF-8.10 | "Managed by your organisation" indicator | Designed | Shell 6.9, 6.14; NW 6.14 | Stage 1, SH-M4 | Policy rows show source and admin contact; page name inconsistent (`about:policies`/`izcal://policy`). |
| BF-8.12 | Auto-update | Designed | Shell 6.11, SH-24; S1 13.2 Updates (TUF), 7.8 | Stage 1 | Non-modal update row, relaunch restores tabs; TUF verification under UPD-R. |
| BF-9.1 | Page info panel | Designed | Shell 6.6, SH-D14 | Stage 1, SH-M2 | Site panel rows, behaviour and data fetching specified. |
| BF-9.2 | Permission prompts | Designed | Shell 6.4, 6.5, SH-D2, SH-D12 | Stage 1, SH-M2 | Prompt classes, anatomy, timing, ledger; "Allow this time" first. |
| BF-9.3 | Quieter notification prompts | Designed | Shell 6.4, SH-D17 | Stage 2 | Interaction required, quiet chip first, prompt caps per tab/profile. |
| BF-9.4 | Site settings manager | Designed | Shell 6.9, SH-D15 | Stage 1, SH-M4 | Registry rows carry scopes (global, container, site), defaults, reasons. |
| BF-9.5 | Auto-revoke unused permissions | Designed | Shell 6.4 | Stage 1 | Grants retired after 30†/90† days unused, visible "expired" state. |
| BF-9.6 | Device access choosers | Designed | Shell 6.4 device-pairing class; DV 6.18 | Stage 1 (experimental setting) | Broker-owned listbox chooser, no default selection; text in shell, not DV. |
| BF-9.8 | Autoplay controls | Designed | MD 6.12, MD-D13; S1 §6 | Stage 1 | Audible playback needs activation or per-site allow; no block-all option. |
| BF-9.12 | Protocol handler / external app prompts | Designed | Shell 6.4; SEC 6.14a; PH 6.4.6 | Stage 1 | Gesture-gated prompt naming scheme, handler and origin; PH adds URL checks. |
| BF-10.1 | Find in page | Designed | Shell 6.8 find bar, ED-D17, SH-32 | Stage 1, SH-M5 | Broker-coordinated find; case, diacritics, whole word, regex options. |
| BF-10.2 | Page zoom | Designed | Shell 6.8; AX-D19; GR 6.28 | Stage 1, SH-M1 | Per-site and global zoom 25–500% with reset; pinch via visual viewport. |
| BF-10.7 | Spellcheck | Designed | ED 6.8, ED-D7, ED-D8, ED-D22 | Stage 2 (ED-D8 restaged 21 Oct) | Own engine and lexicons, lexicon gate, interim OS tier; map's Stage 1 stale. |
| BF-11.1 | Picture-in-picture | Designed | MD 6.12 | Stage 2 | Video PiP shell-owned OS window; Document PiP restrictions designed. |
| BF-11.5 | Live captions | Designed | ML-D2 (ML-ASR), MD-D25 | Stage 1 (ML-M3) or Stage 2 | ML-ASR caption generation specified; Q-ML-1 open on grant. |
| BF-11.9 | DRM playback | Designed | D-S7, S1 13.1a, MD-D32 | Stage 1 (M7) | Protected media mode via OS webviews plus native PlayReady. |
| BF-12.1 | Tab sleeping / memory saver | Designed | Shell 6.8 cold-tab discard; FP-D15 | Stage 1 | Discard rules and "Sleeping" moon-glyph state specified; no user levels. |
| BF-12.2 | Always-keep-active site list | Designed | FP-D15; UI 10 (A-UI-1) | Stage 2 | Per-site keep-running control exempt from discard; no list view. |
| BF-12.8 | Hardware acceleration toggle | Mentioned | Shell 6.14 recovery window, SH-D22 | Stage 1, SH-M4 | Only recovery-window "Start without hardware acceleration"; no Settings row specified. |
| BF-13.1 | Extension support | Designed | EX spec (EX-D1–EX-D16), D-S3 | Stage 2 | MV3 in component host (EX-D3); Stage 1 ships none. |
| BF-13.2 | Extension store | Designed | EX 6.12, EX-D10 | Stage 2 | Directory, review, decision log, signing after review designed. |
| BF-13.3 | Extension management UI | Designed | Shell 6.3 `shell.extensions`, 6.15; EX 6.12 Extensions page | Stage 2 | List, version, enabled state, grants; enable, disable, revoke; grants per container. |
| BF-13.4 | Cross-ecosystem extension support | Designed | EX 6.1 Ring A, EX-D1, EX-D4 | Stage 2 | MV3 namespaces in Ring A under published conformance contracts (spot-check confirmed). |
| BF-13.8 | Themes | Designed | Shell 6.2, SH-D6 | Stage 1 | User theme is custom-property values edited in Appearance; limited CSS field. |
| BF-13.10 | Light/dark mode for the browser UI | Designed | Shell 6.2; UI 4.10 | Stage 1 | Built-in light and dark themes follow the OS by default. |
| BF-13.13 | Keyboard shortcuts | Designed | Shell 6.16, SH-4 | Stage 1 | F6 cycling, roving tabindex, shortcut sheet on `?`, screen-reader key conflicts tested. |
| BF-13.17 | Experimental feature flags | Designed | Shell 6.9 "No hidden settings", SH-D15 | Stage 1, SH-M4 | Separate Diagnostics page behind warning; risk listed; reset on upgrade; no about:config. |
| BF-14.1 | Screen reader support | Designed | AX spec (AX-D3, D4, D9, D12); shell 6.16; UI-1 | Stage 1 | Full AT adapter design; NVDA, JAWS, VoiceOver, Orca audit each release. |
| BF-14.2 | High contrast / forced colours | Designed | AX 6.11; shell 6.16 "Colour and contrast" | Stage 1 | 4.5:1/3:1 in forced colours; forced colours and increased contrast tested separately. |
| BF-14.3 | Focus highlight and caret browsing | Designed | AX 6.12, AX-D20; shell 6.16 "Focus" | Stage 1 | F7 engine caret browsing on `hg-caret`; 2 px outline at 3:1. |
| BF-14.4 | Keyboard-only browsing | Designed | Shell 6.16, SH-4; UI-2 | Stage 1 | Every action has a keyboard path; 100% reachable, 0 traps. |
| BF-14.5 | AI image descriptions | Designed | ML 6.x ML-IMG, ML-D2, ML-D4; MD-D25; HGBA-3 B10 | Stage 1, ML-M1 | Model set, `accessibility.model` grant, never exposed to the page. |
| BF-14.6 | Live caption | Designed | ML-ASR (ML model table), MD-D25 | Stage 1 ML-M3, or Stage 2 | ML-ASR specified: streaming cues, timestamps, confidence; follows captions setting. |
| BF-14.7 | Magnifier support / full-page zoom | Designed | AX 6.8; shell 6.16 "Zoom, reflow and text", 6.8 | Stage 1 | Win32 system caret mirroring for magnifiers; page zoom 25–500%, reflow at 400%. |
| BF-14.9 | Talk to type / dictation | Owned | AX 6.9; ED-D5; ED-4 | Stage 1 (OS hand-off) | OS dictation arrives as `insertReplacementText`; ED-4 test matrix (spot-check confirmed). |
| BF-15.14 | On-device models and their management | Designed | ML 6.3 model manager, ML-D4, ML-D10, ML-D11 | Stage 1, ML-M3 | Model manager lists installed models with size, purpose; removal; grant-off deletes. |
| BF-15.15 | AI master off switch | Designed | HGBA-3 B10, D13; SH-D16, SH-8; S1 §6 "AI switch" | Stage 1 | Off by default, per-profile, policy lock, no AI surfaces when off. |
| BF-15.17 | Agent / automation server for external AI tools | Designed | DT spec IIP and adapters, D-S11, DT-D10 | Stage 1 (BiDi/CDP); MCP when AI on | IIP with BiDi, CDP, DAP adapters; MCP when AI is on. |
| BF-16.8 | Proxy settings | Designed | NW 6.13 "Proxies", NW 6.14 | Stage 1 (OS), Stage 2 (per-container editing) | Per-container direct, OS, PAC, HTTP(S) CONNECT, SOCKS5 configuration. |
| BF-17.4 | Default browser setting and prompt | Designed | Shell 6.11, SH-D21; S1 §6 | Stage 1 | OS-sanctioned API once, end of onboarding; "Not now" final; Windows default-apps surface. |
| BF-18.1 | Developer tools | Designed | DT spec, D-S11, DT-D10 | Stage 1 | Own `hg-devtools` frontend over IIP; full DevTools spec. |
| BF-18.4 | Report a broken site / feedback | Designed | CB 6.11, CB-D18; shell 6.6 | Stage 1 | Report contents, redaction, consent, replay specified. General product feedback absent. |
| BF-18.5 | Internal status pages | Designed | Shell 6.14 "About pages", 6.9 | Stage 1, SH-M4 | Each about: page's contents listed; registry test for every page. |
| BF-18.6 | Remote debugging and automation | Designed | DT spec (IIP, BiDi, CDP, DAP); S1 M2 | Stage 1 | Same adapters as BF-15.17, session-scope checks. |
| BF-18.7 | Beta / preview channels | Mentioned | S1 7.8; shell 6.9 "Update channel" row | Stage 1 (named) | One line plus settings row; switching, downgrade, profiles, side-by-side unspecified. |
| BF-19.1 | Import from other browsers | Designed | Shell 6.12, SH-D19, SH-R2; S1 13.2 | Stage 1, SH-M3 (M7) | Chrome/Firefox import with preview and undo; no Safari importer. |
| BF-19.2 | First-run onboarding | Designed | Shell 6.14, SH-D23, SH-6; UI 4.6 | Stage 1 | Four skippable screens, listed in order, ≤ 90 s. |
| BF-19.4 | Reset / refresh settings | Designed | Shell 6.14 recovery table, 6.9 | Stage 1 | Restores registry defaults; keeps bookmarks, history, passwords, containers, tabs. |
| BF-19.5 | Crash reporting with consent | Designed | D-S4; S1 §6 "Crash reports"; shell 6.14 | Stage 1 | Opt-out, per-crash prompt, minidumps only, viewable before sending. |
| BF-19.6 | Troubleshooting / safe mode | Designed | Shell 6.14 recovery window, SH-D22; D-S2 | Stage 1, SH-M4 | Five-action recovery window incl. themes and extensions off. |
| BF-21.4 | Offline page saving | Designed | S1 §6 "Save for offline" | Stage 2 | User-invoked frozen DOM-and-resources snapshot; B1 archive option (spot-check confirmed). |

### 1.2 OS integration and operations (105 rows)

| # | Feature | Grade | Owner (spec §) | Milestone/stage | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| PO-1.9 | Tarball / portable build | Mentioned | S1 13.2 "Packaging" | Stage 1 | Only "a tarball on Linux"; profile location, portability, TUF update of unpacked install unspecified (spot-check). |
| PO-1.14 | First-run experience and import | Designed | SH 6.12, 6.14, SH-D23; SEC SH-R2 | Stage 1, SH-M5 (M6) | Four skippable screens; import preview, 24 h undo, CSV passwords, no cookie decryption. |
| PO-2.1 | [SB] Windows Authenticode signing | Owned | SEC 6.16 controls, SEC-16 (a) | Stage 1, P1 (before M7) | "Windows signing in a dedicated, environment-protected job"; certificates rotate yearly; OS process. |
| PO-2.3 | [SB] macOS Developer ID and hardened runtime | Owned | SEC 6.16, 6.7 table; SEC-16 | Stage 1, P1 | Developer ID signing; Hardened Runtime on updater, broker, crash handler (SEC 6.7). |
| PO-2.4 | [SB] macOS notarisation and stapling | Owned | S1 13.2; SEC-16 (a) | Stage 1, P1 | "OS-signed (and notarized on macOS)"; stapling not named (minor). |
| PO-2.5 | macOS JIT entitlement | Designed | SEC 6.7; JS 6.8 | Stage 1 | `allow-jit` on JIT executable only; jitless processes use helper binary lacking it. |
| PO-2.7 | Signed update payloads | Designed | SEC 6.16 UPD-R1–R3, SEC-17; S1 7.8 | Stage 1 | Izcal TUF client, split roles, offline root; every data channel a TUF target. |
| PO-2.9 | Reproducible builds, SBOM, SLSA | Designed | S1 7.8; SEC-13, SEC-15 | Stage 1 P1 (engine images M7; browser binary Stage 2) | Provenance+SBOM attestations, immutable releases; SEC-15 independent-builder reproducibility from M7. |
| PO-3.2 | Elevation-free updates on Windows | Designed | SEC-D14 (SEC 6.7) | Stage 1 | Decided: per-user install, no elevated component; per-machine is Stage 2 trigger. |
| PO-3.4 | [SB] Release channels | Mentioned | S1 7.8; SH 6.9 | Stage 1 | One line "Nightly, Beta and Release channels"; promotion, cadence, switching unspecified (spot-check). |
| PO-3.10 | Relaunch notification | Designed | SH 6.11 "Updates"; SH-24 | Stage 1 | Non-modal row; "Later" respected 7 days, then restart at next safe moment. |
| PO-3.11 | Update while running | Designed | S1 1 (staged updates); SH 6.11; SH-24 | Stage 1 | Applies without relaunch when possible; SH-24: 100% tabs, windows, ledger survive. |
| PO-3.12 | Emergency out-of-band release | Designed | SEC UPD-R4, UPD-R5, SEC-D11 | Stage 1 | Emergency release class with provenance; TUF-signed mitigation channel, closed switch list. |
| PO-3.15 | Supported-OS policy and end of support | Mentioned | GR-D21 | Stage 1 | Support rule exists; dropped-OS warning and last-build availability absent (spot-check). |
| PO-4.1 | [SB] Component update channel | Designed | SEC UPD-R1; NW-D17 (6.13); S1 13.2 | Stage 1 | UPD-R1: every data channel (lists, CRLs, blocklist, phishing, models) a TUF target. |
| PO-4.2 | [SB] Root certificate store | Designed | NW-D5, NW-D24 (NW 6.5.2) | Stage 1, NW-M2 | Izcal root store; OS stores as labelled fallback if NW-53 gate not passed. |
| PO-4.3 | Root program governance | Designed | NW 6.5.2 | Stage 1 | Policy, inclusion review, monitoring, distrust, CA/B Forum and CCADB participation. |
| PO-4.4 | Revocation data | Designed | NW 6.5.1, NW-D21; S1 13.2 | Stage 1, NW-M2 (both CRLs and filters) | NW-D21 brings CRLite-style filters forward to NW-M2; S1's "filters Stage 2" superseded. |
| PO-4.5 | Certificate Transparency policy and log list | Designed | S1 13.2; NW 6.5.1 | Stage 1 | "CT is enforced in Stage 1"; log list via trust-data channel. |
| PO-4.6 | [SB] Safe-browsing lists | Designed | PH (PH-D1–D15); SEC-D7 | Stage 1 | Local-list check before DNS and redirects; own bundle, 15-minute poll. |
| PO-4.7 | Public Suffix List | Owned | H2 2.3.2; S1 13.2; NW-D7 | Stage 1 | Shipped as standards data in network service; update path not stated. |
| PO-4.8 | HSTS preload list | Designed | NW-D25 (NW 6.5.3) | Stage 1 | Izcal's own list, published criteria, 12-week re-checks. |
| PO-4.10 | DRM content decryption module | Designed | D-S7 (S1 13.1a); MD-D32 | Stage 1 (M7) | Licensed components only, protected playback affordance; Linux Clear Key until licensed. |
| PO-4.12 | Spellcheck dictionaries | Designed | ED 6.8, ED-D7, ED-D8, ED-D22; TX-D14 | Stage 2 for interim OS tier (ED-D8 restaged 21 Oct) | Own lexicons shipped in engine image; map's "Stage 1 interim tier" is stale. |
| PO-4.13 | Fonts and font fallback data | Designed | TX-D1, TX-D14, TX-D24; FP-D13 | Stage 1 (TX-W1 onward) | One signed Izcal font package, OS-family tier, COLRv1 emoji. |
| PO-4.14 | On-device ML models | Designed | ML-D7, ML-D10, ML-11 | Stage 1 (AX grant); Stage 2 (AI switch) | `.hgmodel` signed container; zero weights in installer; download at explicit enable. |
| PO-4.15 | Locale and Unicode data | Designed | H2 2.3.2; TX (`hg-ucd`, `hg-unicode`) | Stage 1 | UCD, CLDR, tz as standards data; Izcal's own Unicode algorithms, no ICU. |
| PO-4.16 | Tracking-protection and filter lists | Designed | CB-D13, CB-D17; NW-D26 | Stage 1 | Izcal's own behaviour-classified lists, TUF targets with witnessed log. |
| PO-4.17 | Remote kill switches and config | Designed | SEC UPD-R5, SEC-D11 | Stage 1 | Closed switch list with expiry; no general server-pushed config. |
| PO-5.1 | [SB] Crash capture and minidumps | Designed | S1 1 (crash handler), D-S4; SEC SH-R5 | Stage 1 | Separate native crash-handler process, minidumps only, no page content. |
| PO-5.2 | Crash report submission and consent | Designed | D-S4 (S1 6 "Crash reports"); SH 6.14 | Stage 1 | Opt-out, first-run disclosure, per-crash prompt; local crash list missing. |
| PO-5.5 | Session restore after a crash | Designed | SH 6.8 "Session store and restore"; SH 6.14 | Stage 1, SH-M3 | Atomic store every 5 s; failed restore never silent; recovery window. |
| PO-5.6 | Safe mode / troubleshoot mode | Designed | D-S2, SH-D22 (SH 6.14) | Stage 1, SH-M4 | Native recovery window, five actions, reachable ≤ 3 s. |
| PO-5.10 | Local feature flags page | Designed | SH 6.9 "No hidden settings" | Stage 1 | No `about:config`; separate Diagnostics page, reset on upgrade. |
| PO-5.11 | Troubleshooting and support pages | Designed | SH 6.14 "About pages"; SEC 6.7 | Stage 1 | `about:diagnostics` contents specified; `about:policies`, `izcal://sandbox`. |
| PO-5.13 | Network and performance trace capture | Designed | DT 6.3 "Exports" (`hg-export`, `hg-redact`) | HAR Stage 1; qlog, Perfetto Stage 2; OTLP Stage 3 | HAR with `_izcal` fields, redaction by default; map omits OTLP's Stage 3. |
| PO-6.13 | Lockable preferences | Mentioned | SH 6.9; H3 B10; ML-D4 | Stage 1 (display only) | Spot-check: locked-row display specified; lockable set and policy delivery absent. |
| PO-6.18 | Native messaging hosts | Designed | EX 6.11, EX-D9 | Stage 2 (EX-M3) | Broker-mediated; connect-time grant naming the host program. Spot-check confirmed. |
| PO-6.20 | AI and generative-feature controls | Designed | H3 B10, D13; ML-D4 | Stage 1 | Off by default; admins force off and lock, never on. Policy delivery missing (PO-6.1–6.6). |
| PO-7.1 | Integrated Windows auth | Designed | NW 6.1, NW-D20; NW 6.14 | Stage 1 (NW-M4) | SSPI in sandboxed OS-adapter; ambient auth only for allowlisted hosts, default empty. |
| PO-7.2 | Kerberos on macOS and Linux | Designed | NW 6.1, NW-D20 | Stage 1 | GSSAPI in the OS-adapter process under the same allowlist. |
| PO-7.4 | Basic and Digest auth dialogs | Designed | NW 6.14 "HTTP authentication" | Stage 1 (NW-M4) | Prompt renderer via CON-R2; cross-origin subresource prompts refused; 407 per container. |
| PO-7.5 | Client certificates (mutual TLS) | Designed | NW 6.14, 6.1 | Stage 1 (NW-M4) | Selection via CON-R1, remembered per (T, C, host); signing grant per handshake. |
| PO-7.7 | Proxy support: manual, PAC, WPAD, SOCKS | Designed | NW 6.13, 6.1 | Stage 1 | Per-container proxies; PAC in jitless resolver utility; WPAD only off public networks. |
| PO-7.8 | Enterprise and locally installed roots | Designed | NW 6.5, 6.1 | Stage 1 | OS-added anchors passed to `hg-pki`, marked locally trusted. Spot-check confirmed. |
| PO-7.10 | DoH policy and resolver choice | Designed | NW 6.6, NW-D8; NW 6.14 | Stage 1 | Own stub with OS configuration, user-chosen DoH; policy sets DoH mode and resolver. |
| PO-7.11 | Encrypted Client Hello controls | Designed | NW 6.5; NW 6.14 | Stage 1 | ECH on or off by policy. Spot-check confirmed. |
| PO-7.13 | Private network access controls | Designed | NW 6.1 (LNA tickets); NW 6.14 | Stage 1 | LNA tickets per frame and worker; policy sets LNA defaults. |
| PO-7.14 | Captive portal detection | Designed | NW 6.13 "Captive portals" | Stage 1 | OS state, RFC 8908/8910, TLS-failure inference; opens in ephemeral container. |
| PO-8.4 | Default browser agent and prompts | Designed | SH 6.11; UI R19, UP3 | Stage 1 | One OS-sanctioned ask at end of onboarding; a "no" is final. |
| PO-8.8 | Downloads: Mark of the Web and quarantine | Designed | SH 6.10; SEC SH-R6; NW 6.14 | Stage 1 | Broker applies `Zone.Identifier`/`com.apple.quarantine` at completion; Linux residual shown. |
| PO-8.10 | Open download / show in folder | Designed | SH 6.10; SH 6.3 `shell.downloads` | Stage 1 | `shell.downloads`: open, show in folder, copy path; risky types need a prompt. |
| PO-9.1 | OS accessibility API: Windows | Designed | AX (AX-D3, AX-D4, AX-D9) | Stage 1 | UIA and MSAA/IAccessible2 1.3 adapters, synchronous answers from local memory. |
| PO-9.2 | OS accessibility API: macOS | Designed | AX §2/§6 OS interfaces | Stage 1 | NSAccessibility with HIServices text markers and macOS 26 web constants. |
| PO-9.3 | OS accessibility API: Linux | Designed | AX, AX-D12 | Stage 1 | AT-SPI2 on the accessibility bus through a sandboxed bridge. |
| PO-9.4 | Accessibility API mappings conformance | Designed | AX 6.2 (role tables) | Stage 1 | Roles generated from pinned ARIA, HTML-AAM, SVG-AAM, Core-AAM tables; accname. |
| PO-9.5 | IME and text services | Designed | ED 6.x (ED-D5 to ED-D7); S1 13.2 | Stage 1 | TSF `ITextStoreACP2`, IMM32, NSTextInputClient, text-input-v3 with IBus/Fcitx. |
| PO-9.6 | Touch keyboard and handwriting | Designed | ED 6.13, ED-D18 | Stage 1 | `inputmode`/`enterkeyhint` mapped to InputScope etc.; show rules on touch focus. |
| PO-9.10 | OS credential store for secrets | Designed | SH 6.12; SEC SH-R1, SEC-34 | Stage 1 | Keychain, Credential Manager, Secret Service D-Bus via `zbus`, no libsecret. |
| PO-9.12 | OS biometric re-authentication | Designed | SH 6.12 | Stage 1 | Fresh OS authentication before reveal or CSV export. Spot-check confirmed. |
| PO-9.13 | OS spellchecker | Designed | ED 6.8, ED-D8 (b) | Stage 2 (restaged 21 Oct; map said Stage 1) | Interim spell utility, OS local provider only, never script-visible. Stage corrected. |
| PO-9.17 | Clipboard and drag-and-drop | Designed | ED 6.10 (ED-D10–D12), 6.11 (ED-D13) | Stage 1 | Only the broker touches OS clipboard, incl. Wayland primary selection; brokered DnD. |
| PO-9.18 | Screen capture backends | Designed | MD capture service (MD 6.x); GR 6.27 | Stage 1 | ScreenCaptureKit picker, GraphicsCapturePicker, xdg-desktop-portal ScreenCast. |
| PO-9.19 | Camera, microphone and OS privacy prompts | Designed | MD capture service | Stage 1 | Capture service opens devices through OS pickers and consent. macOS TCC unnamed. |
| PO-9.21 | Audio output and device routing | Designed | MD 6.8 audio service | Stage 1 | Core Audio, WASAPI shared, PipeWire; setSinkId/selectAudioOutput per (site, container). |
| PO-9.25 | OS proxy and network-change notifications | Designed | NW 6.1 | Stage 1 | OS proxy, resolver and network-change events read by the broker as config data. |
| PO-9.26 | OS certificate stores and trust evaluation | Designed | NW 6.1, 6.5.2 | Stage 1 | Broker reads OS-added anchors and distrust; OS adapter signs with client keys. |
| PO-10.1 | GPU driver blocklist and workarounds | Designed | GR 6.18, GR-D19; SEC GPU-R8 | Stage 1 | TUF data channel within 7 days of advisory, from Izcal's device testing. |
| PO-10.2 | GPU process isolation and device-loss recovery | Designed | GR 6.17, 6.18; H2 5.5 | Stage 1 | Device-loss epochs, LossReason mapping, CPU path; no GPU work in broker. |
| PO-10.3 | Hardware acceleration toggle | Mentioned | SH 6.14, SH-D22 (recovery window) | Stage 1 (recovery only) | Only recovery-window "Start without hardware acceleration"; no Settings row (as BF-12.8). |
| PO-10.4 | Graphics API backends | Designed | GR (`hg-gpu-hal`), H2 D3 | Stage 1, GR-M1 | Metal, D3D12, Vulkan only plus CPU raster; per-backend version floors (GR 6.4). |
| PO-10.5 | Hardware video decode and encode | Designed | MD 6.x (MD-D10) | Stage 1 | Hardware-decode process designed; gap: hardware encode only in staffing rows. |
| PO-10.6 | Codec patent licensing via OS decoders | Owned | S1 13.2 "Codecs"; MD-D3 | Stage 1 (legal review before M7) | H.264/AAC/HEVC only via OS decoders; legal review, no Izcal design needed. |
| PO-10.7 | Hardware-backed DRM paths | Designed | S1 13.1a (D-S7) | Stage 1 (M7) | Protected media mode plus native PlayReady via Media Foundation, licensing track. |
| PO-10.8 | Power management and OS QoS | Designed | H2 4.3; FP 6.7, 6.8; GR 6.9 | Stage 1 | QoS classes, timer tolerance, wake grid, battery frame cap (spot-check confirmed). |
| PO-10.9 | Battery and memory saver modes | Designed | FP-D15, FP 6.8; S1 6 | Stage 1 (discard), Stage 2 ("limit on battery") | One memory policy for discard/bfcache; per-site battery limit; no global toggle by design. |
| PO-10.11 | HDR and wide colour | Designed | GR 6.5 (GR-D16); MD-D9 | Stage 1 | Compositor display conversion once; BT.2390 EETF tone mapping. |
| PO-10.12 | Variable refresh rate and vsync source | Designed | GR 6.9 (GR-D15); AN | Stage 1 | Highest needed display rate; per-renderer BeginFrame decimation. |
| PO-10.13 | [SB] Dark mode, system theme and accent colour | Designed | SH 6.2 theming, 6.9 Appearance; ST UA variants | Stage 1 | Light/dark themes follow OS; gap: OS accent colour never mentioned. |
| PO-10.14 | High contrast and forced colours | Designed | AX 6.11 (AX-D18); UI 6 | Stage 1 | `hg-ax-prefs` reads OS settings; forced colours follow OS palette. |
| PO-10.15 | OS reduced motion and text scaling | Designed | AX 6.11 (AX-D18, AX-D19); AN 6.11 | Stage 1 | One zoom model with OS text scale; reduced motion honoured. |
| PO-11.1 | [SB] Windows x64 | Owned | S1 D-S5; GR-D21 | Stage 1 (sandbox M5, complete before M8) | Platform target with milestones; per-OS work designed in each spec. |
| PO-11.3 | [SB] macOS on Apple silicon and Intel | Owned | GR-D21; S1 D-S5 | Stage 1 | Intel while patched, Metal macOS 14+; universal binary vs two builds undecided. |
| PO-11.4 | [SB] Linux x86-64 on Wayland and X11 | Designed | GR 6.10 presentation table; ED 6.x (ED-D5); GR-D21 | Stage 1 | Wayland dmabuf/explicit-sync and X11 Present/DRI3 presentation rows. |
| PO-11.5 | Linux ARM64 | Mentioned | FP-25; JS ISAs | — (no Linux ARM64 target) | Spot-check: FP-25 never names Linux; no aarch64-linux target listed. |
| PO-11.6 | [SB] Windows sandbox primitives | Designed | SEC 6.7; S1 D-S5 | Stage 1 (M5) | Per-process AppContainer/LPAC, integrity levels, ACG/CIG table. |
| PO-11.7 | Windows exploit mitigations | Designed | SEC 6.7; SEC-D5 | Stage 1 (broker CIG measured at M5) | CET, CFG, ACG, CIG per process tier in SEC 6.7 table. |
| PO-11.8 | [SB] macOS sandbox primitives | Designed | SEC 6.7; SEC-D6 | Stage 1 | Custom deny-default SBPL per process; Enhanced Security; SDK 27 risk. |
| PO-11.9 | [SB] Linux sandbox primitives | Designed | SEC 6.7; FP-D1 | Stage 1 | Namespaces, seccomp-bpf, Landlock ABI per process; reduced tier reported. |
| PO-13.3 | [SB] WebDriver classic | Designed | DT 6.3 (`hg-bidi`, `izcaldriver`); DT-5 | Stage 1 (M2 slice, M7 full) | One-time enablement, loopback, Host/Origin validated, random session URLs. |
| PO-13.4 | WebDriver BiDi | Designed | DT 6.3; DT-5 | Stage 1 (M7) | All BiDi modules; user contexts map to containers; pipe transport. |
| PO-13.5 | CDP-compatible remote debugging endpoint | Designed | DT 6.3 (DT-D10, DT-D12); S1 D-S11 | Stage 1 (JS, DOM, CSS, Network, Page, Overlay); Stage 2 rest | Izcal-authored profile; per-session loopback WebSocket, user-enabled; pipe. |
| PO-13.6 | Safari-style remote automation switch | Designed | DT 6.3 "WebDriver's HTTP server" | Stage 1 | "runs only after a one-time enablement in settings, after Safari". |
| PO-14.7 | Full keyboard access and focus management | Designed | UI-2; SH 6.16; AX 6.12 (AX-D20, AX-D21) | Stage 1 (ship blocker) | F6 region cycling; 100% shell actions keyboard-operable; shortcut sheet. |
| PO-14.8 | Accessibility settings in the browser | Designed | SH 6.9 "Accessibility"; AX 6.11–6.12 | Stage 1 | Caret browsing, minimum font size, zoom, focus indicators; policies unspecified. |
| PO-15.2 | [SB] Third-party licence notices | Designed | SH 6.14 About pages; SH G209 | Stage 1 | `about:licences` MIT inventory generated from the build. |
| PO-15.8 | Encryption export classification | Owned | S1 13.2 "Privacy and legal" | Stage 1 (before first public binary) | EAR 5D002 legal filing; regulatory, no Izcal design needed. |
| PO-15.12 | Codec, DRM and font licences | Owned | S1 13.2 "Codecs"; S1 13.1a licensing track; TX-D24 | Stage 1 (codec review before M7) | Codec review, Widevine/PlayReady/castlabs enquiries; Izcal's own font set. |
| PO-16.1 | [SB] Vulnerability disclosure policy and intake | Owned | S1 13.2 "Security response"; SEC 6.17 | Stage 1 (before first public binary) | Published policy, security contact, embargo process; policy document only. |
| PO-16.4 | Public security advisories | Owned | S1 13.2; SEC 6.17; SEC-25 | Stage 1 | Machine-readable advisories; advisory scope set by SEC-25. |
| PO-16.5 | [SB] Fix-time targets | Designed | SEC-D1, SEC-1, SEC-25 | Stage 1 | 72 h exploited, 7 days critical/high; time zero defined; published. |
| PO-16.7 | Coordination and embargoes | Owned | S1 13.2; SEC 6.17 | Stage 1 (before first public binary) | Embargo for own fixes and coordinated disclosures; process document. |
| PO-16.8 | Continuous fuzzing and sanitiser builds | Designed | S1 7.5, 7.2; SEC-24 | Stage 1 | cargo-fuzz targets per parser on OSS-Fuzz; Fuzzilli REPRL; pre-release campaigns. |
| PO-17.1 | [SB] Update and download servers and CDN | Owned | SEC 6.16 (SEC-17); NW-D17; S1 13.2 "Updates" | Stage 1 | Untrusted GitHub/CDN hosting, TUF integrity root; capacity/cost not estimated. |
| PO-17.5 | Safe-browsing and reputation service | Designed | PH 6.0–6.2 (`hg-phish-ingest`, `hg-phish-compile`); PH 9.2 | Stage 1 | Server-side list build pipeline and governance, staffed (spot-check confirmed). |

## 2. Part 2: Excluded and Deferred rows

### 2.1 Browser product features (38 rows, all labelled Excluded in the map)

| # | Feature | Grade | Owner (spec §) | Milestone/stage | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| BF-1.4 | Tab groups | Deferred | Shell 9.1, SH-D27; Q-SH-9 | Stage 2 | Shell 9.1 Stage 2 row; group model is open question Q-SH-9. |
| BF-1.5 | Saved and synced tab groups | Deferred, no owner | Shell 9.1 (groups Stage 2; sync Stage 3 or never) | Stage 2 / Stage 3 or never | Saving a group named nowhere; sync part has no spec (Q-SH-5 open). |
| BF-1.6 | Two-level tab stacks | Refused (implicit) | UI §8 Non-goals, UP5 | — | "does not compete on tab-strip layout"; stacks not named. |
| BF-1.7 | Vertical tabs | Deferred | Shell 9.1, SH-D27; UI R18 | Stage 2 | Stage 2 one-click toggle; conflicts with S1 §6 out-of-scope list. |
| BF-1.8 | Split view | Deferred | Shell 9.1, SH-D27 | Stage 2 | Stage 2 table stakes; same S1 §6 conflict as BF-1.7. |
| BF-1.9 | Workspaces / Spaces | Refused | S1 §6 out-of-scope list | Out of scope Stages 1–2 | S1 §6 names "spaces" explicitly as chrome-as-differentiation. |
| BF-1.23 | Tabs from other devices | Deferred, no owner | Shell 9.1, SH-D20; Q-SH-5 | Stage 3 or never | Multi-device "Stage 3 or never"; sync spec not commissioned; owner decision open. |
| BF-2.7 | Quick actions / command palette | Deferred | Shell 9.1, SH-D27; Q-SH-8 | Stage 2 (trial) | Measured Stage 2 trial, removed under 5%† weekly use. |
| BF-2.12 | AI search mode in address bar | Refused (implicit) | SH-D16; Shell 6.8 | — | AI-off: zero surfaces, no AI suggestions; AI-on case never designed. |
| BF-2.13 | Sponsored / contextual suggestions | Refused | Shell 6.8, 6.7 | — | "sponsored or AI-generated suggestions do not exist"; "no sponsored tiles". |
| BF-3.9 | News feed on new tab page | Refused (implicit) | SH-D29; Shell 6.7 | — | Shell makes no network requests; a feed would need one. |
| BF-3.10 | Dashboard widgets on start page | Misread → Missing | — (Shell 6.7, SH-D29 refuse networked only) | — | Local widgets neither refused nor designed; "widget" absent from shell spec. |
| BF-4.3 | Browser sign-in / account | Deferred, no owner | SH-D20; Shell 9.1; Q-SH-5 | Stage 3 or never | No account in Stage 1; sync/account is open owner question Q-SH-5. |
| BF-4.4 | Sync | Deferred, no owner | Shell 9.1, SH-D20; Q-SH-5; S1 §9 | Stage 3 or never | "Sync is commissioned as its own spec" (SH-D20); none exists; Q-SH-5 open. |
| BF-4.5 | End-to-end encrypted sync | Deferred, no owner | As BF-4.4 | Stage 3 or never | Follows sync; no encryption model recorded. |
| BF-4.6 | Send tab to device | Deferred, no owner | Shell 9.1 "Sync and multi-device" | Stage 3 or never | Falls under multi-device row; no owner spec. |
| BF-4.7 | Send files and notes between devices | Deferred, no owner | Shell 9.1 "Sync and multi-device" | Stage 3 or never | Same multi-device row; not named. |
| BF-4.9 | Sync troubleshooting and data reset | Deferred, no owner | As BF-4.4 | Stage 3 or never | Follows sync; no owner spec. |
| BF-5.5 | Print and print preview | Misread → Designed | LY 6.7, GR 6.27 (GR-M3); Shell 6.19 print UI | Stage 1 layout/raster; Stage 2 UI | Layout, `@page`, preview raster Stage 1; print UI Stage 2, unowned (as PO-9.14). |
| BF-5.10 | PDF summarise and ask | Misread → Missing | — (ML-D2 defers web Summarizer API only) | — | No PDF summarise product surface in PD or shell. |
| BF-6.3 | Breached and weak password alerts | Misread → Mentioned | Shell 6.12 (no owner) | — | "no breach-check service in Stage 1" scopes; weak/reused checks never mentioned. |
| BF-6.8 | Address autofill | Deferred | Shell 9.5 (confirms D-S3/S1 13.2); ED 6.6 preview | Stage 2 | "autofill beyond passwords ... Stage 2"; address store undesigned. |
| BF-6.9 | Payment card autofill | Deferred | Shell 6.9, 9.5; D-S3 | Stage 2 | "Payment autofill is Stage 2 and has no row until it ships". |
| BF-7.17 | IP masking in private browsing | Refused | S1 §6 out-of-scope ("browser-operated VPN service") | — | IP hiding only via user-supplied routes (NW 6.13, Stage 2). |
| BF-10.4 | Reader mode | Deferred, no owner | UI R18/§6; S1 §6; PD 9.1 (reflow with reader view) | Stage 2 | Named Stage 2 only; shell spec never mentions reader view. |
| BF-10.6 | Page translation | Misread → Missing | — (ML-D2 defers web Translator API only) | — | No product page-translation surface; shell "translat" hits are string localisation only. |
| BF-10.8 | Cloud / enhanced spellcheck | Refused | ED §4.4, ED-D7 | — | Cloud spellcheck "Break"; offline only; grammar model Stage 2 under AI grant. |
| BF-12.4 | Performance issue alerts | Refused (implicit) | Shell 6.8 cold-tab discard; UP2 | — | "never asked to close tabs to save memory". |
| BF-13.5 | Legacy MV2 support | Refused | EX-D2 | — | "MV3 only; MV2 refused with a published migration mapping". |
| BF-13.18 | Language settings | Misread → Designed | Shell 6.9 "Language and writing"; ED-D8, ED 6.8 | Stage 2 (hidden until ED-D8 ships) | Shell 6.9 specifies the group's rows and defaults for Stage 2; not a refusal. |
| BF-15.1 | AI assistant panel | Deferred, no owner | UI §6 Stage 3 "Agent surfaces"; HGBA-3 §7, B10 | Stage 3 | Only a Stage 3 line in UI §6; no spec designs or owns a panel. |
| BF-15.3 | Page summarisation | Misread → Mentioned | ML-D2 (ML 6.x "Stage 2 tasks") | Stage 2 (task API only) | ML-D2 lists a summarise task API; no product feature, surface or owner. |
| BF-15.8 | Writing help | Deferred | ED 6.8 (ED-D9); ML-D2; Shell 6.9 Language and writing | Stage 2, under AI grant | ED-D9 stages the learned grammar model to Stage 2; ED and ML own it. |
| BF-16.6 | Feed reader | Deferred, no owner | S1 §9 Stage 2 "optional W-1 components (feeds, …)" | Stage 2 | Named in S1 §9 only; no component spec owns feeds. |
| BF-16.7 | Built-in VPN / secure proxy | Refused | S1 §6 out-of-scope list | — | Explicit: "a browser-operated VPN service" is out of scope for Stages 1–2. |
| BF-17.5 | OS notifications for web notifications | Misread → Designed | Shell 6.13, SH-D17; Q-SH-10 open | Stage 2 | Shell 6.13 designs broker decision, centre, mute, snooze, digest; OS channel question open. |
| BF-17.11 | App installs from the browser (iOS EU) | Refused (implicit) | S1 13.2 "iOS" | — (Stages 1–2 out of scope) | iOS platform is out of scope; feature not named itself. |
| BF-19.8 | Lost-device lock and erase | Refused (implicit) | SH-D20; Shell 9.1 "Stage 3 or never"; Q-SH-5 | Stage 3 or never | No account or sync, so remote lock/erase cannot exist; not named. |

### 2.2 OS integration and operations (16 rows)

| # | Feature | Grade | Owner (spec §) | Milestone/stage | Evidence (≤ 15 words) |
|---|---|---|---|---|---|
| PO-1.7 | Flatpak (Flathub) | Deferred | S1 13.2 "Packaging"; SEC-8 (reduced tier on `izcal://sandbox`) | Stage 2 | Zypak-style redirection, documented reduced isolation; SEC-8 reports the reduced tier. |
| PO-2.10 | iOS alternative-engine entitlement | Refused | S1 13.2 "iOS" | — (revisit if DMA enforcement changes) | "Out of scope for Stages 1–2", with a stated revisit trigger. |
| PO-4.11 | Codec plug-ins | Refused | S1 13.2 "Codecs" (A-S1-85); MD-D3 | — | No OpenH264, dav1d or libvpx shipped; patented codecs only via OS decoders. |
| PO-7.15 | Browser VPN / proxy privacy service | Refused | S1 §6 out-of-scope list; S1 §6 "Containers"; NW-D6 | — | S1 §6 names "a browser-operated VPN service"; routes are user-supplied. |
| PO-9.7 | [SB] OS notifications | Deferred | Shell 6.13, SH-D17; Q-SH-10 | Stage 2 | Stage 1 denies honestly; Shell 6.13 owns Stage 2 delivery; OS-vs-centre open. |
| PO-9.14 | [SB] OS printing | Deferred, no owner | S1 §6 "Printing"; GR 6.27 (raster/vector only); PD 6.16 | Stage 1 layout; Stage 2 print UI | Shell 6.19 lists print UI under PD; PD-M4 treats it as dependency; nobody designs. |
| PO-9.15 | Save as PDF | Deferred | PD 6.19 (`hg-pdf-write`/`hg-pdf-print`), PD-D19–21 | Stage 2 (PD-M4) | Writer designed in PD 6.19; ships with Stage 2 print UI. |
| PO-11.10 | Android | Deferred, no owner | H2 S-8; S1 §1, §9; CB "Interfaces for later stages" | Stage 2 | Only isolation rule S-8 and interface notes; no Android shell or packaging spec. |
| PO-11.11 | iOS/iPadOS | Refused | S1 13.2 "iOS" | — (Stages 1–2) | "Out of scope for Stages 1–2." |
| PO-13.1 | Embedding API (WebView) | Deferred, no owner | S1 13.2 "Embedding API"; CB (lists as data) | Stage 2 | One S1 row; no spec designs the WebView API (JS `hg-js-host` is the JS embedder). |
| PO-13.2 | Embedded runtime distribution and update | Misread → Missing | — (S1 13.2 defers only the API) | — | Shared-runtime distribution and update never mentioned in any report. |
| PO-14.1 | [SB] UI localisation at scale | Misread → Mentioned | Shell 4.4 (T12), 6.17 "Strings"; S1 13.2 "Text and input" | — (Stage 1 English-only; no later stage) | Strings Fluent-externalized; "English-only forever" rejected; no translation stage, locales or owner. |
| PO-16.2 | Bug bounty | Deferred | SEC-D10 (b); S1 13.2 "Security response" | After funding (D-S8), not a stage | SEC-D10 decided: external audit pre-release, "bounty when D-S8 allows". |
| PO-17.2 | [SB] Web Push service | Deferred | NW 6.14 "Web Push" (client); S1 13.2 "Service workers" (autopush) | Stage 2 | NW 6.14 designs the system-service push client; self-hosted autopush is Stage 2 ops. |
| PO-17.3 | Sync service | Misread → Mentioned | Shell 9.1 "Stage 3 or never"; SH-D20; Q-SH-5 open | — (open owner question) | "Is sync ever in scope?" is open; not a deferral, no owner. |
| PO-17.4 | Extension store and signing service | Misread → Designed | EX 6.12, EX-D10 | Stage 2 | Directory, review, re-review before re-signing and distribution designed in EX 6.12. |

## 3. Counts

### 3.1 Part 1: Covered rows

| Map | Rows | Designed | Owned | **Still covered** | Mentioned | Not there | Error rate |
|---|---|---|---|---|---|---|---|
| BF | 94 | 86 | 2 | **88** | 6 | 0 | 6.4% |
| PO | 105 | 86 | 13 | **99** | 6 | 0 | 5.7% |
| Both | 199 | 172 | 15 | **187** | 12 | 0 | 6.0% |

The spot-check measured a 24% error rate in its sample, which was weighted toward thin rows: 18 of every 25 rows it checked had thin evidence. Across the whole of both maps the rate is 6%. Every Mentioned row except PO-10.3 was already in the spot-check's sample.

### 3.2 Part 2: Excluded and Deferred rows

| Map | Rows | Refused | of which implicit | Deferred | Deferred, no owner | Misread → Designed | Misread → Mentioned | Misread → Missing | Misread total |
|---|---|---|---|---|---|---|---|---|---|
| BF | 38 | 12 | 6 | 7 | 11 | 3 | 2 | 3 | 8 (21%) |
| PO | 16 | 4 | 0 | 5 | 3 | 1 | 2 | 1 | 4 (25%) |
| Both | 54 | 16 | 6 | 12 | 14 | 4 | 4 | 4 | 12 (22%) |

Only 16 of the 54 rows are refusals. The BF map labels all 38 of its rows Excluded, but 18 of them are deferrals and 8 are misreads.

## 4. Lists

**Mentioned (Part 1, 12 rows):**
- BF-1.3 pinned tabs
- BF-1.12 mute tab or site
- BF-5.4 save page
- BF-7.3 private window over Tor
- BF-12.8 hardware acceleration toggle
- BF-18.7 beta / preview channels
- PO-1.9 tarball / portable build
- PO-3.4 [SB] release channels
- PO-3.15 supported-OS policy and end of support
- PO-6.13 lockable preferences
- PO-10.3 hardware acceleration toggle (new; not in the spot-check sample, and the same gap as BF-12.8)
- PO-11.5 Linux ARM64

**Not there (Part 1):** none.

**Deferred, no owner (Part 2, 14 rows):**
- BF-1.5 saved and synced tab groups
- BF-1.23 tabs from other devices
- BF-4.3 sign-in / account
- BF-4.4 sync
- BF-4.5 E2E encrypted sync
- BF-4.6 send tab to device
- BF-4.7 send files between devices
- BF-4.9 sync troubleshooting
- BF-10.4 reader mode
- BF-15.1 AI assistant panel
- BF-16.6 feed reader
- PO-9.14 [SB] OS printing (print UI and OS print path)
- PO-11.10 Android
- PO-13.1 embedding API

**Misread (Part 2, 12 rows):**

| Row | Feature | Corrected grade |
|---|---|---|
| BF-3.10 | Start-page widgets | Missing |
| BF-5.5 | Print and preview | Designed (Stage 1 layout and raster); the print UI is Deferred, no owner |
| BF-5.10 | PDF summarise | Missing |
| BF-6.3 | Breach / weak password alerts | Mentioned |
| BF-10.6 | Page translation | Missing |
| BF-13.18 | Language settings | Designed (Stage 2) |
| BF-15.3 | Page summarisation | Mentioned |
| BF-17.5 | OS notifications | Designed (Stage 2) |
| PO-13.2 | Embedded runtime distribution | Missing |
| PO-14.1 | [SB] UI localisation | Mentioned |
| PO-17.3 | Sync service | Mentioned |
| PO-17.4 | Extension store and signing | Designed (Stage 2) |

**Where this pass departs from the spot-check:**
- **Sync rows.** The spot-check confirmed BF-1.23, 4.3, 4.6 and 4.7 as exclusions, and BF-1.5 as implicitly refused. Here they are Deferred, no owner. Shell 9.1 says "Stage 3 or never", SH-D20 commissions a separate sync spec that does not exist, and Q-SH-5 is open. None of these is a refusal. PO-17.3 keeps the spot-check's Misread → Mentioned, which is the same gap under a different label.
- **BF-6.3.** The spot-check says Missing. It is graded Mentioned here, because Shell 6.12 names the breach-check service. Weak and reused password checks are absent.
- **BF-15.3.** The spot-check says "Missing or Partial". It is graded Mentioned, because ML-D2 names a Stage 2 summarise task. There is no product surface and no owner.
- **BF-5.5.** The spot-check says Partial. The engine half is Designed. The print UI half is Deferred, no owner, as in PO-9.14.
- **PO-9.13.** The spot-check says "Only partial (wrong stage)". The design in ED 6.8 and ED-D8 (b) is complete, and only the stage was wrong, so the row is Designed (Stage 2).
- **PO-9.14, PO-11.10, PO-13.1.** The spot-check confirms these Deferred rows. They are Deferred, no owner, because only S1 or HGBA-2 name them, or, for the print UI, Shell 6.19 and PD-M4 each point to the other.

**Stage corrections on Covered rows:**
- BF-10.7, PO-4.12 and PO-9.13 move to Stage 2, after ED-D8 was restaged on 21 October.
- PO-4.4: CRLite-style filters land at NW-M2 in Stage 1. NW-D21 supersedes S1 13.2's "Stage 2".
- PO-5.13: OTLP export is Stage 3 (DT 6.3).

**Inconsistency to resolve when the maps merge.** BF-17.5 and PO-9.7 are the same feature, delivered through Shell 6.13 in Stage 2. BF-17.5 is graded Misread → Designed and PO-9.7 Deferred, because the two maps gave them different starting statuses. Shell 6.13 is a one-paragraph design: the broker decides, the OS is one delivery channel, and mute, snooze, digest and auto-revoke are specified. Per-OS integration is not, and the OS-channel question (Q-SH-10) is open. Both rows have an owner and a stage, so neither is a gap.
