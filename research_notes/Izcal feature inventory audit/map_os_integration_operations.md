# Izcal feature inventory audit: mapping of OS integration and operations

Compiled 24 October 2026 against the reports on main at 217e02d.

**Method.** Each row of `os_integration_operations.md` (237 rows, PO-1.1 to PO-17.9) was searched across all 33 files in `reports/` with `grep -i` on at least three distinct terms, and the hit sections were read. Status rules: **Covered** = a spec states behaviour; **Partial** = mentioned or partly specified, gap named; **Excluded** = refused by a recorded decision or non-goal; **Deferred (Stage n)** = postponed by a recorded decision with an owner; **Missing** = no spec mentions it (search terms listed). **[SB]** = ship blocker in the inventory. Short names: S1 = `HGBA Stage 1 architecture design.md`; H2 = HGBA-2; H3 = `Future software browser engine.md`; SH = `Izcal shell.md`; UI = `HGBA shell UI design.md`; SEC = `Izcal security scope.md`; NW = `Izcal networking.md`; GR = graphics and rendering; MD = images and media; TX = fonts and text; ED = editing; AX = accessibility; DT = DevTools; EX = extensions; AH = app host; PH = phishing protection; CB = content blocking; FP = resource footprint; ML = ML service; DV = devices.

## 1. Packaging and installation

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-1.1 | [SB] Windows consumer installer | Partial | S1 13.2 "Packaging"; SEC-D14; FP-25 | Stage 1 | "a signed installer on Windows" (S1 13.2); per-user, no elevated component (SEC-D14); ≤ 50 MiB† (FP-25). Missing: installer technology, online stub vs offline bundle, install location, first-launch flow, repair |
| PO-1.2 | [SB] Windows enterprise MSI | Missing | — (SEC-D14's reversal trigger assumes "Enterprise deployment (Stage 2) needs per-machine installs") | — | Searched "MSI", "silent install", "SCCM", "Intune", "per-machine". Only SEC-D14 names per-machine installs, as a Stage 2 trigger with no owner |
| PO-1.3 | MSIX / Microsoft Store package | Missing | — | — | Searched "MSIX", "Microsoft Store", "Windows Store". MSIX appears only in AH's history tables as a comparison |
| PO-1.4 | [SB] macOS disk image and pkg | Partial | S1 13.2 "Packaging"; SEC-16 | Stage 1 | "Signed and notarized `.dmg` on macOS" covers consumers. Missing: a signed `.pkg` for MDM deployment ("pkg" appears nowhere) |
| PO-1.5 | Mac App Store distribution | Missing | — | — | Searched "Mac App Store", "App Store" (only AH/EX history rows). No decision; the JIT, custom SBPL and own updater make it a likely non-goal, but none is recorded |
| PO-1.6 | [SB] Linux deb and rpm packages | Partial | S1 13.2 "Packaging"; SEC 6.5 updater threat model | Stage 1 | ".deb, .rpm and a tarball on Linux in Stage 1". Missing: the signed APT/YUM repository, and how system-package-manager updates coexist with the TUF updater (SEC lists "macOS and Linux packaging interplay (notarization, distribution packages)" as **Open**) |
| PO-1.7 | Flatpak (Flathub) | Deferred (Stage 2) | S1 13.2 "Packaging"; SEC 6.7 | Stage 2 | "Flatpak is Stage 2 through zypak-style sandbox redirection … the Flatpak build documents its reduced isolation" |
| PO-1.8 | Snap | Partial | GR G67 | — | Snap appears only in GR G67's artifact rule ("installers, Snap, AppImage … contain no Vulkan loader"); no decision whether a Snap ships, nor its sandbox interplay (SEC 6.7 covers Flatpak only) |
| PO-1.9 | Tarball / portable build | Covered | S1 13.2 "Packaging" | Stage 1 | Linux tarball in Stage 1. No portable Windows or macOS form (not requested by the inventory beyond Linux) |
| PO-1.10 | Side-by-side channels | Partial | S1 7.8 | Stage 1 | Nightly, Beta and Release channels exist. Missing: whether channels install side by side, with separate profiles, icons and update tracks |
| PO-1.11 | Initial preferences / partner distributions | Missing | — | — | Searched "initial preferences", "master preferences", "distribution folder", "partner build", "OEM" |
| PO-1.12 | Install-time options | Missing | — (AH 6.12 covers apps only) | — | Searched "desktop shortcut", "Start menu", "taskbar pin". Only AH's app install "offers a desktop shortcut" |
| PO-1.13 | [SB] Uninstall, repair and leftover cleanup | Missing | — (AH 6.12, AH-13 cover installed apps only) | — | Searched "uninstall", "repair", "leftover". Every hit is the app host's app uninstall; the browser's own uninstaller and keep/delete-profile choice are unspecified |
| PO-1.14 | First-run experience and import | Covered | SH 6.12, 6.14; SEC SH-R2; S1 13.2 "Everyday features" | Stage 1 (SH-M5 at M6) | Four skippable onboarding screens; import preview, undo for 24 h†; Chrome and Firefox bookmarks/history, Chrome passwords via user-exported CSV, never cookies. Safari and Edge importers are not listed |
| PO-1.15 | Profile management | Partial | S1 13.2 "Profiles"; SH 4.1, SH-D26; SH 6.8 | Stage 1 (format) | Versioned profile format with forward-only migrations; "profile lock" named as a restore failure reason; "profiles later" (SH 4.1); containers are the identity model (SH-D26). Missing: several profiles per install, a picker, lock semantics, refresh of a damaged profile beyond the recovery window's settings reset |
| PO-1.16 | VDI, roaming profiles and multi-session | Missing | — | — | Searched "VDI", "roaming", "Citrix", "multi-session", "terminal server" |
| PO-1.17 | Kiosk mode | Missing | — | — | Searched "kiosk" (only DV's grant-lifetime note, WebCrypto's aside and AH's Chrome Apps history), "single-app mode", "--kiosk" |

## 2. Code signing and platform trust

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-2.1 | [SB] Windows Authenticode signing | Covered | SEC 6.16 controls, SEC-16 (a) | Stage 1 (P1, before M7) | "Windows signing in a dedicated, environment-protected job; certificates rotate at least yearly†"; "100% of release packages are OS-signed". Not decided: EV vs OV vs a managed signing service |
| PO-2.2 | [SB] SmartScreen app reputation | Missing | — | — | Searched "SmartScreen" (hits are phishing/editing comparisons only), "app reputation", "reputation build" for the signing identity. The launch-day reputation lead time is not planned |
| PO-2.3 | [SB] macOS Developer ID and hardened runtime | Covered | SEC 6.16, 6.7 table; SEC-16 | Stage 1 (P1) | "OS code signing (Apple Developer ID and notarization)"; Hardened Runtime on broker, crash handler and updater; Enhanced Security |
| PO-2.4 | [SB] macOS notarisation and stapling | Covered | S1 13.2; SEC-16 (a) | Stage 1 | "100% of release packages are OS-signed (and notarized on macOS)". Stapling is not named |
| PO-2.5 | macOS JIT entitlement | Covered | SEC 6.7; JS 6.8 | Stage 1 | "`allow-jit` is a Hardened Runtime entitlement on the executable … jitless processes run from a helper binary that lacks it" |
| PO-2.6 | [SB] Linux repository and package signing | Partial | SEC-16 (a); SEC 6.5 updater "Open" | Stage 1 | SEC-16 requires release packages to be "OS-signed", and the updater threat model leaves "distribution packages" open. Searched "OpenPGP", "GPG", "APT repository", "package repository": 0 hits. No repository key, metadata signing or Flathub path |
| PO-2.7 | Signed update payloads | Covered | SEC 6.16 UPD-R1–R3, SEC-17; S1 7.8 | Stage 1 | Izcal's own TUF client; roles split; offline root; every data channel a TUF target |
| PO-2.8 | Third-party DLL injection blocking | Partial | SEC 6.7 table | Stage 1 (M5 measurement) | CIG in renderers and services; for the broker "CIG and extension-point blocking **to be measured at M5**". Missing: a crash-driven blocklist of injected modules, and its policy override |
| PO-2.9 | Reproducible builds, SBOM, SLSA | Covered | S1 7.8; SEC-13, SEC-15 | Stage 1 (P1; browser binary P2) | Build attestations (provenance and SBOM), immutable releases, SLSA L3 reusable workflow, engine images reproducible from M7, browser binary in Stage 2 |
| PO-2.10 | iOS alternative-engine entitlement | Excluded | S1 13.2 "iOS" | — | "Out of scope for Stages 1–2 … Revisit if DMA enforcement changes" |

## 3. Updates and release channels

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-3.1 | [SB] Background auto-updater | Partial | S1 1 ("Updater … On update check"); SEC UPD-R2; SH 6.11 | Stage 1 | The updater process verifies and applies; launched by the broker. Missing: updating while the browser is closed (scheduled task, launch agent, systemd timer) and the check schedule |
| PO-3.2 | Elevation-free updates on Windows | Covered | SEC-D14 | Stage 1 | Per-user install, "no elevated component exists to attack". Per-machine installs with an elevated service are a Stage 2 decision |
| PO-3.3 | Delta (differential) updates | Missing | — | — | Searched "delta update", "differential" (hits are oracle tests), "binary diff", "patch payload". CB notes "TUF specifies no delta mechanism" for lists only |
| PO-3.4 | [SB] Release channels | Covered | S1 7.8 | Stage 1 | "Nightly, Beta and Release channels"; SH 6.9 exposes the channel |
| PO-3.5 | Release cadence | Missing | — | — | Searched "release cadence", "four-week", "every four weeks", "cadence" (hits are engine/test cadences). No major-release cadence is set |
| PO-3.6 | Extended support channel | Missing | — | — | Searched "extended stable", "ESR", "extended support" (one retired V8 note) |
| PO-3.7 | Staged (percentage) rollout | Missing | — (AH 6.7 mentions staged rollout for apps only) | — | Searched "staged rollout", "percentage rollout", "rollout", "halt". No rollout or halt control for browser releases |
| PO-3.8 | Version pinning and rollback | Missing | — | — | Searched "version pin", "rollback", "roll back". "Rollback" hits are TUF anti-rollback (the attack) and AH's 7-day app rollback; no admin pin or target-version rollback |
| PO-3.9 | Update policy controls | Partial | SH 6.9 "Updates and recovery"; SH 6.11 | Stage 1 | User settings: update channel and relaunch policy. Missing: admin policies (check period, suppression windows, channel targeting, update URL override, disable updates) |
| PO-3.10 | Relaunch notification | Covered | SH 6.11; SH-24 | Stage 1 | Non-modal update row; "Later" respected 7 days†, then restart at the next safe moment; states what will be restored |
| PO-3.11 | Update while running | Covered | S1 1 ("staged updates"); SH 6.11; SH-24 | Stage 1 | Apply without relaunch when possible; SH-24: 100% of tabs, windows and ledger entries survive across three versions |
| PO-3.12 | Emergency out-of-band release | Covered | SEC UPD-R4, UPD-R5, SEC-D11 | Stage 1 | Emergency release class signed by the `releases` role; TUF-signed mitigation channel with a closed switch list |
| PO-3.13 | Weekly security updates | Partial | SEC-D1 (JS-16), SEC-25 | Stage 1 | Fix bounds: 72 h† exploited, 7 days† critical/high. Missing: a scheduled security-release train between majors |
| PO-3.14 | Release notes publishing | Partial | H3 B10 (ML inventory); S1 13.4 UA-1; SEC 6.17 | Stage 1 | Release notes are named as an output (ML inventory changes "reported in the release notes"; UA-1 published per release; advisories machine-readable). Missing: a release-notes process and enterprise notes |
| PO-3.15 | Supported-OS policy and end of support | Covered | GR-D21 | Stage 1 | "Izcal supports an OS version while its vendor ships security updates … A vendor ends security updates: that OS version is dropped at the next release". Missing: the warning shown on a dropped OS and keeping the last build available |

## 4. Component and data updates

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-4.1 | [SB] Component update channel | Covered | SEC UPD-R1; NW-D17 (6.13); S1 13.2 "Updates" | Stage 1 | Every data channel (lists, CRLs, GPU blocklist, phishing bundle, models, emergency config) is a TUF target; published endpoint list |
| PO-4.2 | [SB] Root certificate store | Covered | NW-D5, NW-D24 (6.5.2); S1 13.2 "Certificates" | Stage 1 (first store at NW-M2) | Izcal's own root program and store; OS stores interim in development builds and as a labelled fallback if NW-53 is not passed at first release |
| PO-4.3 | Root program governance | Covered | NW 6.5.2; D-S8 note | Stage 1 | Policy, inclusion review, monitoring, distrust, CA/B Forum and CCADB participation; 4–6 people† ongoing |
| PO-4.4 | Revocation data | Covered | S1 13.2 "Certificates"; NW 6.5.1 | Stage 1 (CRLs), Stage 2 (CRLite filters) | Pushed CRLs in Stage 1, CRLite-style filters in Stage 2; OCSP stapling honoured; kill records |
| PO-4.5 | Certificate Transparency policy and log list | Covered | S1 13.2; NW 6.5.1 | Stage 1 | "CT is enforced in Stage 1"; log list filtered by Izcal's log policy, delivered by the trust-data channel |
| PO-4.6 | [SB] Safe-browsing lists | Covered | PH (PH-D1–PH-D15); SEC-D7 | Stage 1 | Local-list check before DNS and every redirect; own bundle on the 15-minute† poll. Malware URLs are a documented gap until a commercial licence (SEC-D7) |
| PO-4.7 | Public Suffix List | Covered | H2 M-1 2.3.2 standards-data exception; S1 13.2 "Shipped data"; NW-D7 | Stage 1 | Shipped as standards data; a static global of the network system service |
| PO-4.8 | HSTS preload list | Covered | NW-D25 (6.5.3) | Stage 1 | Izcal's own list with published criteria and 12-week† re-checks; Chromium's list not used |
| PO-4.9 | Extension and add-on blocklist | Partial | EX 6.10 (distribution, review); SEC UPD-R1 | Stage 2 | "a remote kill whose use is disclosed in the decision log"; extension updates are TUF targets. Missing: the blocklist format, client behaviour on a kill (disable vs remove, offline) and its policy override |
| PO-4.10 | DRM content decryption module | Covered | D-S7 (S1 13.1a); MD-D32 | Stage 1 (M7) | Licensed components only: protected media mode (WebView2/WKWebView), native PlayReady, Clear Key; Linux Clear Key only until licensing succeeds |
| PO-4.11 | Codec plug-ins | Excluded | S1 13.2 "Codecs" (A-S1-85) | — | "no OpenH264 binary and no `dav1d` or `libvpx` dependency is shipped"; H.264/AAC/HEVC only through OS decoders |
| PO-4.12 | Spellcheck dictionaries | Covered | ED-D7, ED-D8, ED-D22 (ED 6.8) | Stage 1 (interim OS tier); Izcal lexicons per ED-D8 | Izcal's own lexicons "validated at build time and shipped in the engine image"; Hunspell and its dictionaries are out under M-1. The settings group is Stage 2 (SH 6.9) |
| PO-4.13 | Fonts and font fallback data | Covered | TX-D1, TX-D14, TX-D24; FP-D13 | Stage 1 (TX-W1 onward) | One signed Izcal font package; interim OS tier; COLRv1 emoji from Fluent Emoji (MIT) plus Izcal artwork |
| PO-4.14 | On-device ML models | Covered | ML-D7, ML-D10, ML-11; S1 13.2 "Updates" | Stage 1 (accessibility grant); Stage 2 (AI switch) | Models are update-channel payloads on a second signing chain, downloaded only at explicit enable |
| PO-4.15 | Locale and Unicode data | Covered | H2 M-1 2.3.2 (UCD, CLDR, tz exceptions); TX (`hg-ucd`, `hg-unicode`) | Stage 1 | Standards data shipped as data; Izcal's own Unicode algorithms, no ICU |
| PO-4.16 | Tracking-protection and filter lists | Covered | CB (CB-D13, CB-D17); NW-D26; S1 6 | Stage 1 | Izcal's own lists (EasyList/EasyPrivacy excluded by licence), TUF targets with a witnessed log |
| PO-4.17 | Remote kill switches and config | Covered | SEC UPD-R5, SEC-D11 | Stage 1 | Closed list only (jitless, Wasm off, Wasm interpreter off, WebGPU off, one codec off), each with an expiry; general server-pushed config is deliberately not provided |

## 5. Crash reporting, telemetry and experimentation

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-5.1 | [SB] Crash capture and minidumps | Covered | S1 1 (crash handler), D-S4; SEC SH-R5 | Stage 1 | Separate native crash-handler process, minidumps only, filtered stacks; per-OS handler mechanism not detailed |
| PO-5.2 | Crash report submission and consent | Covered | D-S4; S1 6 "Crash reports"; SH 6.14 | Stage 1 | Opt-out with first-run disclosure, per-crash prompt, viewable before sending; dumps deleted after 30 days†. Missing: local crash list and quotable report IDs |
| PO-5.3 | Crash server and symbolication | Partial | S1 7.8 | Stage 1 | "Symbol server and crash reporting follow D-S4". Missing: ingestion server, symbol store, signature grouping, regression flagging, retention and access |
| PO-5.4 | OS crash and hang reporting integration | Missing | — | — | Searched "Windows Error Reporting", "WER", "spin dump", "hang detect", "hang report" |
| PO-5.5 | Session restore after a crash | Covered | SH 6.8 "Session store and restore"; SH 6.14 recovery triggers | Stage 1 | Atomic session store every 5 s†; failed restore never silent; recovery window after three failed restores |
| PO-5.6 | Safe mode / troubleshoot mode | Covered | D-S2, SH-D22 (SH 6.14) | Stage 1 | Native recovery window: without GPU, themes and extensions off, reset settings, send crash report, quit |
| PO-5.7 | Usage telemetry | Partial | D-S4; S1 6 "Usage telemetry" | Stage 1 | Decision: opt-in, "nothing sent without the user being able to see what is sent". Missing: the metrics pipeline, the metric set, the viewer and the ingestion side |
| PO-5.8 | Telemetry consent and deletion | Partial | D-S4; S1 13.2 "Privacy and legal"; SH 6.14 | Stage 1 (before M7) | Consent model decided; "GDPR and CCPA handling for crash reports and opt-in telemetry, written before M7". Missing: deletion of data already sent, regional defaults |
| PO-5.9 | Field trials and server-side experiments | Missing | — | — | Searched "field trial", "server-side experiment", "studies", "experimentation". SH-22 relies on lab studies and opt-in telemetry; no experiment platform and no decision refusing one |
| PO-5.10 | Local feature flags page | Covered | SH 6.9 "No hidden settings" | Stage 1 | Deliberately no `about:config`; diagnostic and experimental settings on a separate Diagnostics page, reset on upgrade unless persistent |
| PO-5.11 | Troubleshooting and support pages | Covered | SH 6.14 "About pages"; SEC 6.7 (`izcal://sandbox`) | Stage 1 | `about:version`, `about:build`, `about:diagnostics`, `about:policies`, `izcal://sandbox` |
| PO-5.12 | User feedback reporting | Partial | SH 6.14 (`about:diagnostics` "record this UI bug"); CB-D18 | Stage 1 | A UI-bug recording action and the content-blocking breakage report over OHTTP. Missing: a general send-feedback form and where it goes |
| PO-5.13 | Network and performance trace capture | Covered | DT 6.3 "Exports" | Stage 1 (HAR), Stage 2 (qlog, Perfetto) | HAR with `_izcal` fields, qlog, Perfetto, OTLP to a local endpoint |

## 6. Enterprise policy and management

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-6.1 | [SB] Windows Group Policy (ADMX/ADML) | Missing | — (S1 1 gives the broker "profile and policy" ownership) | — | Searched "ADMX", "Group Policy", "GPO", "Software\Policies". Many specs consume "enterprise policy" but none says how it is delivered or read |
| PO-6.2 | [SB] MDM on Windows | Missing | — | — | Searched "MDM", "Intune", "OMA-URI", "mobile device management" |
| PO-6.3 | [SB] macOS configuration profiles | Missing | — | — | Searched "configuration profile", "managed preference", "MDM", "plist" |
| PO-6.4 | Apple declarative device management for Safari | Missing | — | — | Searched "declarative", "DDM". Safari-only; the Izcal analogue is PO-6.3 |
| PO-6.5 | [SB] Linux policy files | Missing | — | — | Searched "policies.json", "policy file" (one SEC attacker row), "/etc" |
| PO-6.6 | Cross-platform policy file | Missing | — | — | Searched "policies.json", "policy file", "policy schema" |
| PO-6.7 | Cloud browser management | Missing | — | — | Searched "cloud management", "management console", "admin console" |
| PO-6.8 | Policy precedence and `about:policy` | Partial | SH 6.14 (`about:policies`); NW 6.14 (`izcal://policy`); SH 6.9 | Stage 1 | A page shows policies in force and their sources; policy-set rows are locked and labelled. Missing: mandatory vs recommended, machine vs user scope, merge rules, error display. The two specs name the page differently |
| PO-6.9 | Policy breadth (the catalogue) | Partial | SH 6.9 registry ("whether it can be set by policy"); NW 6.14; H3 B10; DV 6.15; JS-D13; EX 6.6; AH 6.7 | Stage 1 | Policies are scattered: network surface (DoH, QUIC, ECH, proxy lock, auth allowlist, LNA, anchors, client-cert auto-select, HTTPS-First exceptions, list set, Web Push); AI lock; device pre-grant; JIT per-URL lists; extension force-install; app allowlists. Missing: one catalogue and schema |
| PO-6.10 | Managed bookmarks | Missing | — | — | Searched "managed bookmark", "bookmarks policy", "admin bookmarks" |
| PO-6.11 | Extension allow/block lists and force-install | Partial | EX 6.6, 6.10; Q-EX-5 | Stage 2 | "policy may force-install or block"; policy grants shown with their source; enterprise sources in a distinct trust tier. Q-EX-5 leaves the details open "when the enterprise profile is designed" |
| PO-6.12 | URL allow/block lists | Partial | PH 6.11 (enterprise policy); NW 6.14 | Stage 1 | PH says a local URL blocklist "is content blocking's user/enterprise list domain"; NW lists "content-blocking list set" as a policy. Missing: an admin URL allow/block list design in CB |
| PO-6.13 | Lockable preferences | Covered | SH 6.9; H3 B10; ML-D4 | Stage 1 | "A policy can set only registry settings … Policy-set rows are visible, marked 'Set by your organisation'"; AI and the accessibility model can be forced off and locked, never on. Depends on PO-6.1–6.6 for delivery |
| PO-6.14 | Enterprise reporting | Missing | — | — | Searched "enterprise report", "inventory report", "management console" |
| PO-6.15 | DLP / content-analysis connectors | Missing | — | — | Searched "data loss prevention", "DLP", "content analysis" |
| PO-6.16 | Enterprise sign-in and profile separation | Partial | SH-D26 (containers); SH 6.12 (no account) | Stage 1 keying, Stage 2 container UI | A "work" container separates state; there is no account or sign-in. Missing: binding a managed profile to an identity provider |
| PO-6.17 | Legacy browser support / IE mode | Missing | — | — | Searched "IE mode", "legacy browser", "Internet Explorer", "site list". IE mode is cited only as the precedent for protected media mode (S1 13.1a) |
| PO-6.18 | Native messaging hosts | Covered | EX-D9 (EX 6.11); EX-M3 | Stage 2 | Broker-mediated; connect-time grant naming the host program; `hg-ext-native`. The native-messaging policy category is not specified |
| PO-6.19 | Enterprise admin documentation set | Missing | — | — | Searched "admin guide", "deployment guide", "administrator documentation" |
| PO-6.20 | AI and generative-feature controls | Covered | H3 B10, D13; ML-D4; S1 6 "AI switch" | Stage 1 | Off by default; administrators can force off and lock `ai.enabled` and `accessibility.model`, never force on |

## 7. Enterprise networking, authentication and certificates

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-7.1 | [SB] Integrated Windows auth (Negotiate/Kerberos/NTLM) | Covered | NW-D20 (NW 6.1, 6.14) | Stage 1 | SSPI in a sandboxed OS-adapter process; only for hosts on an enterprise or user allowlist, default empty |
| PO-7.2 | Kerberos on macOS and Linux | Covered | NW-D20 (NW 6.1) | Stage 1 | GSSAPI in the OS-adapter process under the same allowlist. The macOS Kerberos SSO extension is not named |
| PO-7.3 | OS single sign-on to cloud identity | Missing | — | — | Searched "single sign-on", "SSO", "Entra", "Azure AD", "PRT" |
| PO-7.4 | Basic and Digest auth dialogs | Covered | NW 6.14 "HTTP authentication" | Stage 1 | Prompt renderer through the consent path; cross-origin subresource prompts refused; proxy 407 per container |
| PO-7.5 | [SB] Client certificates (mutual TLS) | Covered | NW 6.1, 6.14 | Stage 1 | Selection through CON-R1, remembered per (T, C, host); signing in the OS adapter, key never leaves the OS store; policy auto-select per host pattern |
| PO-7.6 | Smart cards and PKCS#11 modules | Partial | NW 6.1 ("client-key signing (smart cards and OS key stores)"); EX 6.3 (refuses `pkcs11`) | Stage 1 | Smart cards through OS key stores are covered. Missing: loading PKCS#11 modules (Linux NSS DB / p11-kit), CAC/PIV testing |
| PO-7.7 | [SB] Proxy support: manual, PAC, WPAD, SOCKS | Covered | NW 6.13, 6.1 | Stage 1 | Per-container proxies, PAC in a proxy-resolver utility with the jitless engine, WPAD only on non-public networks, SOCKS5, MASQUE; proxy policy and lock |
| PO-7.8 | [SB] Enterprise and locally installed roots | Covered | NW 6.5 ("OS trust changes are honoured both ways"), NW 6.1 | Stage 1 | User- and enterprise-added OS roots trusted, marked locally trusted, exempt from CT, shown in the site panel |
| PO-7.9 | TLS version and cipher policies | Missing | — | — | Searched "minimum TLS", "cipher policy", "disabled ciphers", "FIPS" (hits are primitive standards). NW 6.14's policy surface has no TLS-version, cipher or PQ toggle |
| PO-7.10 | DoH policy and resolver choice | Covered | NW-D8 (6.6); NW 6.14 | Stage 1 | Own stub resolver with OS configuration; policy sets DoH mode and resolver |
| PO-7.11 | Encrypted Client Hello controls | Covered | NW 6.5; NW 6.14 | Stage 1 | ECH on or off by policy |
| PO-7.12 | Intranet and single-word navigation | Missing | — | — | Searched "single-label", "single-word", "intranet" (hits: HSTS exclusion and NW-D20's rejected intranet heuristic for auth). The address bar (SH 6.8) does not say how a single-label entry resolves |
| PO-7.13 | Private network access controls | Covered | NW 6.1 (LNA tickets); NW 6.14 ("LNA defaults") | Stage 1 | Local Network Access tickets per frame, applied to the connected address; policy sets defaults |
| PO-7.14 | Captive portal detection | Covered | NW 6.13 "Captive portals" | Stage 1 | OS portal state, RFC 8908/8910, inferred from TLS failures; no own probe endpoint; opens in an ephemeral container |
| PO-7.15 | Browser VPN / proxy privacy service | Excluded | S1 6 "Containers"; NW-D6 | — | "Routes are user-supplied (WireGuard config, SOCKS, Arti), not a vendor VPN service"; no embedded Tor client |
| PO-7.16 | OS VPN and per-app VPN compatibility | Missing | — | — | Searched "per-app VPN", "Always On VPN", "OS VPN", "VPN" (hits are routes and essay reviews). Likely satisfied by the broker's OS sockets, but not stated or tested |
| PO-7.17 | Reputation and phishing protection for enterprise | Partial | PH 6.11 "Enterprise policy"; PH (Chrome Enterprise lesson) | Stage 1 | "No user or policy off switch; a policy can only add blocks". Missing: password-reuse alerts (searched "password reuse", "password protection": 0 hits) |

## 8. Default browser and OS shell integration

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-8.1 | [SB] Default-browser registration on Windows | Partial | SH 6.11; S1 6 "Default-browser handling"; UI R19 | Stage 1 | The ask opens the OS default-apps surface once. Missing: the registration itself (Registered Applications, ProgIDs, capabilities, EEA one-click path). Searched "ProgID", "Registered Applications" |
| PO-8.2 | [SB] Default browser on macOS | Partial | SH 6.11 | Stage 1 | OS-sanctioned API, asked once. Missing: Launch Services scheme and UTI claims. Searched "Launch Services", "LSSetDefault" |
| PO-8.3 | [SB] Default browser on Linux | Partial | SH 6.11 | Stage 1 | Same rule. Missing: desktop entry, MIME associations, `xdg-settings`. Searched "xdg-settings", "mimeapps", "desktop entry" |
| PO-8.4 | Default browser agent and prompts | Covered | SH 6.11; UI R19 | Stage 1 | One ask at the end of onboarding; a "no" is final; no re-check that nags. A background agent is in effect refused |
| PO-8.5 | Enterprise default-browser assignment | Missing | — | — | Searched "default associations", "DefaultBrowserSettingEnabled", "default browser" with "policy" |
| PO-8.6 | Protocol and file-type associations | Partial | SEC SH-R8; SH 6.10; PH 6.4.6 | Stage 1 | Launching external handlers needs a gesture and a per-origin prompt. Missing: registering Izcal as the OS handler for .pdf/.html/.svg, `mailto:` and custom schemes; EX defers `protocol_handlers` and file handlers to "Stage 3, with the OS-integration design", which does not exist |
| PO-8.7 | Taskbar and dock integration | Partial | TX-16 (TX 6.x) | Stage 1 | Only the rule that jump lists, Dock and taskbar menus show broker-generated strings. Missing: jump-list and dock-menu entries, progress, badges, per-profile icons |
| PO-8.8 | Downloads: Mark of the Web and quarantine | Covered | SEC SH-R6; SH 6.10; NW 6.14 | Stage 1 | Broker sets `Zone.Identifier` and `com.apple.quarantine` on every written file; Linux residual stated |
| PO-8.9 | Antivirus scan on download | Partial | SH 6.10 (verdict source "the OS reputation interface") | Stage 1 | One mention. Missing: IAttachmentExecute / AMSI hand-off and the macOS equivalent. Searched "AMSI", "IAttachmentExecute", "Attachment Services", "antivirus" |
| PO-8.10 | Open download / show in folder | Covered | SH 6.10; SH 6.3 `shell.downloads` | Stage 1 | Open via the OS handler after a press; Show in folder, Copy path |
| PO-8.11 | Background mode and start at login | Missing | — | — | Searched "background mode", "start at login", "startup boost", "after the last window". Stage 1 has no notifications (SH 6.13), so there is no current need, but no decision |
| PO-8.12 | OS scripting and automation hooks | Missing | — | — | Searched "AppleScript", "Apple Events", "Shortcuts", "command-line activation" |
| PO-8.13 | Set as desktop background | Missing | — | — | Searched "desktop background", "wallpaper", "set as" |
| PO-8.14 | Continuity and cross-device handoff | Missing | — | — | Searched "Handoff", "send tab", "cross-device", "continuity". Follows from Sync, which is Stage 3 "if wanted" (S1 9) |

## 9. OS service integration

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-9.1 | [SB] OS accessibility API: Windows | Covered | AX (AX-D3, AX-D4, AX-D9); S1 5.8 | Stage 1 | UIA and MSAA with IAccessible2, adapters in the UI host on Windows |
| PO-9.2 | [SB] OS accessibility API: macOS | Covered | AX; S1 5.8 | Stage 1 | NSAccessibility adapter |
| PO-9.3 | [SB] OS accessibility API: Linux | Covered | AX, AX-D12 | Stage 1 | AT-SPI2 over D-Bus through a sandboxed bridge, including the recovery window |
| PO-9.4 | Accessibility API mappings conformance | Covered | AX (HTML-AAM, Core-AAM, accname; AX-O tables) | Stage 1 | Author tree computed as HTML-AAM and accname specify |
| PO-9.5 | [SB] IME and text services | Covered | ED-D5 to ED-D8; S1 13.2 "Text and input" | Stage 1 (M6) | TSF in the UI host, NSTextInputClient, Wayland text-input plus IBus/Fcitx |
| PO-9.6 | Touch keyboard and handwriting | Covered | ED-D18 (ED 6.13) | Stage 1 | `inputmode`/`enterkeyhint` mapped to InputScope and OS hints; InputPane; handwriting commits treated as IME input |
| PO-9.7 | [SB] OS notifications | Deferred (Stage 2) | SH-D17 (SH 6.13); S1 6 "Notification prompts" | Stage 2 | Stage 1 tells sites "notifications are not available in this version"; delivery and OS channels in Stage 2 |
| PO-9.8 | System media controls | Partial | MD 6.12 (MD-D13, MD-D25) | Stage 1 | "Media Session metadata and actions reach the OS media controls through the broker". Missing: the SMTC, MPNowPlayingInfoCenter and MPRIS adapters and hardware media keys |
| PO-9.9 | OS share sheet | Missing | — | — | Searched "share sheet", "Web Share", "navigator.share", "DataTransferManager", "NSSharingService" |
| PO-9.10 | [SB] OS credential store for secrets | Covered | SH 6.12; SEC SH-R1, REC-R2, SEC-34; S1 13.2 | Stage 1 | macOS Keychain, Windows Credential Manager/DPAPI user scope, Secret Service D-Bus called directly |
| PO-9.11 | Passkeys and platform authenticators | Partial | S1 6 "Passkeys and credentials"; SEC SH-R1; DV 6.x ("FIDO/CTAP remains in the credential subsystem") | Stage 1 | Platform authenticators on Windows and macOS. Missing: the CTAP2 roaming-authenticator path (USB/NFC/BLE) that Linux needs; DV refuses FIDO HID to pages and names a "credential subsystem" no spec defines |
| PO-9.12 | OS biometric re-authentication | Covered | SH 6.12 | Stage 1 | "revealing a password requires a fresh OS authentication"; CSV export behind re-authentication |
| PO-9.13 | OS spellchecker | Covered | ED-D8 (interim OS tier) | Stage 1 | OS spelling tier as interim, in the utility pool, never script-visible |
| PO-9.14 | [SB] OS printing | Deferred (Stage 2) | S1 6 "Printing"; S1 13.2 "Everyday features"; GR 6.27 | Stage 1 layout; Stage 2 UI | Paged layout, `@page`, `window.print()` in Stage 1; "the print UI and a general PDF writer are Stage 2". Per-OS print paths (Windows Protected Print, CUPS/IPP, portal, system dialog) are not specified |
| PO-9.15 | Save as PDF | Deferred (Stage 2) | S1 6 "Printing", "PDF viewer"; PDF spec (C-21) | Stage 2 | Print-to-PDF goes through `hg-pdf-write`; "a general PDF writer" is Stage 2 |
| PO-9.16 | [SB] OS file pickers | Partial | S1 13.2 "Permissions" (File System Access); SEC SH-R9; SH 6.9 ("ask where to save") | Stage 1 | Handles from pickers are specified. Missing: the native open/save dialogs (IFileDialog, NSOpenPanel/NSSavePanel, the portal FileChooser) and who hosts them. Searched "file picker", "IFileDialog", "NSOpenPanel", "FileChooser" |
| PO-9.17 | Clipboard and drag-and-drop | Covered | ED (6.10 and related; ED-D11 drag IPC) | Stage 1 | OS clipboard formats, Wayland primary selection, OS drag and drop through the broker |
| PO-9.18 | Screen capture backends | Covered | MD capture service; GR 6.27 | Stage 1 | ScreenCaptureKit picker, `GraphicsCapturePicker`, xdg-desktop-portal ScreenCast; tab capture by the compositor |
| PO-9.19 | Camera and microphone and OS privacy prompts | Covered | MD capture service; S1 1 | Stage 1 | Capture service opens devices through OS pickers and consent. macOS TCC for cameras is not named (only DV's TCC notes) |
| PO-9.20 | OS location services | Partial | S1 13.2 "Permissions"; DM 6.x (Geolocation bindings) | Stage 1 | Permission and bindings exist. Missing: the provider (Windows Location, Core Location, GeoClue). Searched "Core Location", "GeoClue", "location service" |
| PO-9.21 | Audio output and device routing | Covered | MD (audio service); S1 1 | Stage 1 | Core Audio, WASAPI shared mode, PipeWire/PulseAudio; output-device selection |
| PO-9.22 | OS speech synthesis and recognition | Missing | — (EX stages `tts` to Stage 3) | — | Searched "speechSynthesis", "Web Speech", "text-to-speech", "SAPI", "AVSpeech", "speech dispatcher" |
| PO-9.23 | OS device stacks | Partial | DV (DV-D1 to DV-D15) | Stage 1 boundary | USB, HID, serial, Bluetooth, NFC covered. Missing: game controllers (Gamepad API over XInput/GameController/evdev); DV refuses gamepad collections to WebHID |
| PO-9.24 | Screen and system wake locks | Partial | DM 6.x (Screen Wake Lock binding, OS side assigned to shell and security) | Stage 1 | Binding listed. Missing: the OS calls (SetThreadExecutionState, IOPMAssertion, inhibit portal) and Izcal's own wake lock during video and calls |
| PO-9.25 | OS proxy and network-change notifications | Covered | NW 6.1 | Stage 1 | "OS proxy settings, resolver configuration and network-change events are configuration data … read by the broker" |
| PO-9.26 | OS certificate stores and trust evaluation | Covered | NW 6.1, 6.5.2 | Stage 1 | Broker reads OS stores for added anchors and distrust; OS adapter signs with client keys |
| PO-9.27 | Parental controls integration | Missing | — | — | Searched "parental", "Family Safety", "Screen Time", "Family Link", "supervised" |

## 10. Graphics, media hardware, power and display

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-10.1 | [SB] GPU driver blocklist and workarounds | Covered | GR-D19 (GR 6.18); SEC GPU-R8, SEC-10 (e) | Stage 1 | Data through the TUF channel within 7 days† of an advisory, keyed by device class, driver range and OS, from Izcal's device lab |
| PO-10.2 | GPU process isolation and device-loss recovery | Covered | GR 6.17, 6.18; H2 5.5 | Stage 1 | Two GPU process kinds; device-loss epochs; CPU path with identical pixels |
| PO-10.3 | Hardware acceleration toggle | Covered | D-S2/SH-D22 (recovery window) | Stage 1 | "Start without hardware acceleration", with "remember". Missing: an admin policy for it |
| PO-10.4 | Graphics API backends | Covered | GR (`hg-gpu-hal`) | Stage 1 | Metal, Direct3D 12, Vulkan only, plus the canonical CPU raster; no D3D11 or OpenGL by design |
| PO-10.5 | Hardware video decode and encode | Covered | MD (MD-D10, hardware-decode process) | Stage 1 | VideoToolbox, Media Foundation/D3D12 Video, VA-API/Vulkan Video |
| PO-10.6 | Codec patent licensing via OS decoders | Covered | S1 13.2 "Codecs" | Stage 1 (legal review before M7) | H.264, AAC, HEVC only via OS decoders |
| PO-10.7 | Hardware-backed DRM paths | Covered | D-S7 (S1 13.1a) | Stage 1 (M7) | WebView2 supports PlayReady at the hardware level; native PlayReady via Media Foundation |
| PO-10.8 | Power management and OS QoS | Covered | H2 4.3; FP (6.7, 6.8); GR 6.9 | Stage 1 | OS QoS classes and thread priorities; throttling; wake grid; battery-saver frame cap |
| PO-10.9 | Battery and memory saver modes | Covered | FP-D15; S1 6 "Tab memory", "Per-site resource meter"; SH 6.8 | Stage 1 (discard), Stage 2 ("limit on battery") | Automatic discard and bfcache under one policy; the person is never asked to close tabs |
| PO-10.10 | [SB] HiDPI and per-monitor scaling | Partial | GR-D17 (device scale as a rational); GR presentation table (fractional-scale); SH 5 | Stage 1 | Fractional scales handled in paint; Wayland fractional-scale used when present. Missing: Per-Monitor v2 awareness and window moves across mixed-DPI monitors |
| PO-10.11 | HDR and wide colour | Covered | GR-D16 (GR 6.5) | Stage 1 | Display conversion once in the compositor; BT.2390 tone mapping; output descriptor per window |
| PO-10.12 | Variable refresh rate and vsync source | Covered | GR-D15 (GR 6.9) | Stage 1 | Per-OS timing source; display at the highest needed rate; per-renderer decimation |
| PO-10.13 | [SB] Dark mode, system theme and accent colour | Covered | SH 6.9 "Appearance"; UI 6; ST (UA variants) | Stage 1 | Theme follows the OS; `prefers-color-scheme` UA variants. OS accent colour is not mentioned |
| PO-10.14 | High contrast and forced colours | Covered | AX (R0, 6.11); UI 6 | Stage 1 | Forced colours follow the OS; system colours are the user's palette |
| PO-10.15 | OS reduced motion and text scaling | Covered | AX 6.11; AN | Stage 1 | OS text scale sets the default text scale; reduce motion honoured |
| PO-10.16 | Window management integration | Partial | SH 6.2 "Native window frame" | Stage 1 | OS frame gives snap layouts, window menu and title-bar accessibility; custom title bars Stage 2. Missing: Wayland decorations (GNOME offers no server-side decorations), macOS full-screen Spaces and backdrop materials |
| PO-10.17 | Touch, pen and touchpad gestures | Partial | GR 6.x (pinch zoom, LY-D13) | Stage 1 | Pinch zoom specified. Missing: swipe back/forward, pen pressure, macOS trackpad gestures |
| PO-10.18 | Multi-monitor window placement | Partial | SH 6.8 (session store keeps "window bounds and display"); SH 5 | Stage 1 | Restore records the display. Missing: unplugged-monitor handling and per-monitor full screen |

## 11. Platform targets and OS security primitives

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-11.1 | [SB] Windows x64 | Covered | D-S5; GR-D21 | Stage 1 (sandbox at M5, complete before M8) | Windows 10 while patched (ESU to 12 October 2027) and 11 |
| PO-11.2 | Windows on ARM64 | Partial | FP-25 (per architecture: x86-64, AArch64, riscv64); JS ISA targets | Stage 1 | AArch64 is an engine ISA, but no spec names Windows on Arm, and the coding standard's target list has no `aarch64-pc-windows-msvc` |
| PO-11.3 | [SB] macOS on Apple silicon and Intel | Covered | GR-D21; D-S5 | Stage 1 | Intel Macs in while Apple patches them (to macOS 26); Metal on macOS 14+. Universal binary vs two builds not decided |
| PO-11.4 | [SB] Linux x86-64 on Wayland and X11 | Covered | GR presentation table; ED-D5; GR-D21 | Stage 1 | Wayland (dmabuf, fifo, fractional-scale, text-input) and X11 back ends |
| PO-11.5 | Linux ARM64 | Covered | FP-25; AX lab (riscv64/Linux boards); JS ISAs | Stage 1 | AArch64 and riscv64 Linux builds measured per architecture |
| PO-11.6 | [SB] Windows sandbox primitives | Covered | SEC 6.7; D-S5 | Stage 1 (M5) | AppContainer/LPAC, untrusted integrity, lockdown token, job, alternate desktop, win32k lockdown |
| PO-11.7 | Windows exploit mitigations | Covered | SEC 6.7; SEC-D5 | Stage 1 | CET, CFG, ACG and CIG per tier; broker CIG measured at M5 |
| PO-11.8 | [SB] macOS sandbox primitives | Covered | SEC 6.7; SEC-D6 | Stage 1 | Custom deny-default SBPL; Enhanced Security; Endpoint Security evaluation; SDK 27 risk recorded |
| PO-11.9 | [SB] Linux sandbox primitives | Covered | SEC 6.7; FP-D1 (Linux spawner) | Stage 1 | Namespaces, seccomp-bpf, Landlock; packages ship an AppArmor profile; Flatpak and hardened kernels run a reported reduced tier |
| PO-11.10 | Android | Deferred (Stage 2) | H2 S-8; S1 1 ("Android is S-8 in Stage 2"); S1 9 | Stage 2 | Android in Stage 2; Play distribution and System WebView questions not addressed |
| PO-11.11 | iOS/iPadOS | Excluded | S1 13.2 "iOS" | — | Out of scope for Stages 1–2 |

## 12. Web apps installed into the OS

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-12.1 | Web app manifest processing | Missing | — | — | Searched "web app manifest", "Web Application Manifest", "PWA", "manifest.json" (hits are AH's own `.izapp` manifest and history rows) |
| PO-12.2 | PWA install into the OS | Partial | PH 6.4.9 (assumes installed web apps); AH 2–3 (PWA as "the incumbent") | — | PH specifies phishing checks for "an installed web app" in a standalone window, and AH contrasts its `.izapp` apps with PWAs. No spec owns installing a web app |
| PO-12.3 | PWA uninstall through the OS | Missing | — | — | Searched "PWA", "installed web app", "uninstall" (AH-13 covers `.izapp` apps only) |
| PO-12.4 | OS-level app integration | Missing | — | — | Searched "badging", "window controls overlay", "file_handlers", "share_target", "launch_handler" (one EX row deferring extension handler keys to a non-existent "OS-integration design") |
| PO-12.5 | Enterprise force-installed web apps | Missing | — | — | Searched "WebAppInstallForceList", "force-install" with apps; AH 6.7's enterprise allowlist covers `.izapp` packages only |
| PO-12.6 | Store packaging of PWAs | Missing | — | — | Searched "PWA", "MSIX", "Store" |
| PO-12.7 | Site-specific shortcuts / taskbar tabs | Missing | — | — | Searched "site shortcut", "taskbar tab", "open in its own window" |

## 13. Embedding, automation, headless and command line

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-13.1 | Embedding API (WebView) | Deferred (Stage 2) | S1 13.2 "Embedding API" | Stage 2 | "Stage 2. Stage 1 fixes the capability endpoints and process model it will use" |
| PO-13.2 | Embedded runtime distribution and update | Deferred (Stage 2) | S1 13.2 "Embedding API" | Stage 2 | Part of the Stage 2 embedding work; shared-runtime distribution and update are not described |
| PO-13.3 | [SB] WebDriver classic | Covered | DT 6.3 (`hg-bidi`, `izcaldriver`); DT-5; S1 7.3 | Stage 1 (M2 slice, M7 full) | WebDriver HTTP server after one-time enablement, loopback only; WPT `webdriver/tests/classic/` pass rates published |
| PO-13.4 | WebDriver BiDi | Covered | DT 6.3; DT-5 | Stage 1 (M7) | All BiDi modules; user contexts map to containers |
| PO-13.5 | CDP-compatible remote debugging endpoint | Covered | D-S11, DT-D10, DT-D12 (DT 6.3) | Stage 1 (JS, DOM, CSS, Network, Page, Overlay); Stage 2 (rest) | Izcal-authored CDP profile; loopback WebSocket per session, user-enabled; pipe transport |
| PO-13.6 | Safari-style remote automation switch | Covered | DT 6.3 "WebDriver's HTTP server" | Stage 1 | "runs only after a one-time enablement in settings, after Safari" |
| PO-13.7 | [SB] Headless mode | Partial | DT 6.3 ("headless embedders" over `--automation-pipe`); H3 B7 (pixel-optional tabs) | Stage 1 / Stage 2 | Pipe launch into an automation container is specified; pixel-optional rendering is a Stage 2 agent feature. Missing: a headless mode (no windows, no shell) and its rendering and test semantics |
| PO-13.8 | Testing builds (pinned, non-updating) | Missing | — | — | Searched "Chrome for Testing", "testing build", "non-updating", "pinned build" |
| PO-13.9 | Command-line switches | Partial | SEC-20, AGT-R8; DT 6.3 (`--automation-pipe`) | Stage 1 | Rules for what flags may not do (attach to a user container). Missing: the supported switch set (profile, private window, proxy override, logging) |
| PO-13.10 | Single instance and remote control of a running browser | Missing | — | — | Searched "single instance", "running instance", "second instance", "already running". SEC-20 only refuses automation against a profile held by another instance |
| PO-13.11 | Developer tools enablement policy | Missing | — | — | Searched "DevTools" with "policy", "disable developer tools". DT has no admin switch; SH 6.9 has no DevTools row |

## 14. Localisation and the browser's own accessibility

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-14.1 | [SB] UI localisation at scale | Excluded | S1 13.2 "Text and input"; SH-D24; SH 4.4 | — (Stage 1 English-only; no later stage set) | "The shell UI is English-only in Stage 1, with strings in Fluent format". No stage or locale count for translation |
| PO-14.2 | Locale selection and language packs | Partial | SH 6.9 "Language and writing" (Stage 2); FP 6.x optional components | Stage 2 | FP: "extra UI languages for Izcal's own interface chosen in settings", not in the installer. Missing: language-pack delivery and admin-set locale |
| PO-14.3 | Right-to-left UI mirroring | Partial | SH 6.16 "Strings and bidirectional text" | Stage 1 (bidi isolation) | Runtime text isolated with `<bdi>`; pseudolocalization. Missing: a mirrored shell layout for RTL locales |
| PO-14.4 | Translation and localisation pipeline | Partial | SH-D24; SH 9.3 track T12; SH 6.16 | Stage 1 (string layer) | Fluent from the first commit, plain-language review, string budget, locale testing. Missing: translator tooling, string freeze, locale onboarding |
| PO-14.5 | Regional builds and defaults | Missing | — | — | Searched "regional default", "China", "region", "default search" |
| PO-14.6 | [SB] Browser UI accessibility conformance | Partial | SH 6.16; UI-2; S1 13.2 "Shell"; AX targets | Stage 1 | The shell passes the same checks as a web page; keyboard completeness is a ship blocker. Missing: a published conformance report (VPAT, EN 301 549, Section 508). Searched "VPAT", "Section 508", "EN 301 549", "conformance report" |
| PO-14.7 | Full keyboard access and focus management | Covered | UI-2; SH 6.16; AX 6.12 | Stage 1 | F6 region cycling; every shell action keyboard-reachable; shortcut sheet |
| PO-14.8 | Accessibility settings in the browser | Covered | SH 6.9 "Accessibility"; AX 6.11–6.12 (AX-D20, AX-D21) | Stage 1 | Caret browsing (F7), minimum font size, zoom defaults, focus indicators. Accessibility policies not specified |

## 15. Legal, regulatory and compliance

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-15.1 | [SB] Privacy notice and terms of use | Partial | S1 13.2 "Privacy and legal" | Stage 1 (before M7) | "A privacy policy … written before M7". Missing: terms of use (searched "terms of use", "terms of service": 0 hits) |
| PO-15.2 | [SB] Third-party licence notices | Covered | SH 6.14 (`about:licences`, `about:credits`); SH 9.5 | Stage 1 | "the MIT inventory for shipped code and data, generated from the build" |
| PO-15.3 | GDPR data access and export | Partial | SH 6.12 (`izcal-export` v1); S1 13.2 "Privacy and legal" | Stage 1 | Local data export is specified and there is no account. Missing: subject-access handling for crash reports and telemetry held server-side |
| PO-15.4 | Data deletion | Partial | SH 6.9, 6.12; SEC SH-R5 | Stage 1 | Local clearing and 30-day† dump deletion. Missing: deletion on request of sent telemetry and crash data |
| PO-15.5 | DMA browser choice screens | Missing | — | — | Searched "choice screen" (only the pain-points review), "DMA" with eligibility, "listing criteria" |
| PO-15.6 | DMA data portability import | Missing | — | — | Searched "data portability", "Data Portability API", "takeout" |
| PO-15.7 | Age assurance and age signals | Missing | — | — | Searched "age assurance", "age verification", "Declared Age", "Online Safety Act" |
| PO-15.8 | Encryption export classification | Covered | S1 13.2 "Privacy and legal" | Stage 1 (before first public binary) | EAR 5D002; published-source exclusion; legal review before the first public binary because of the bundled DRM path |
| PO-15.9 | [SB] EU Cyber Resilience Act | Missing | — | — | Searched "Cyber Resilience", "CRA", "ENISA", "support period". Pieces exist (SBOM in S1 7.8, disclosure policy in S1 13.2, fix bounds SEC-1) but no spec names the CRA's reporting duty (in force since 11 September 2026), support period or conformity assessment |
| PO-15.10 | European Accessibility Act exposure | Missing | — | — | Searched "European Accessibility Act", "EAA" (one pain-points mention), "Directive 2019/882" |
| PO-15.11 | Trademark and brand rules | Partial | D-S9; S1 13.2 "Privacy and legal" | Stage 1 (before first public binary) | Trademark clearance search required. Missing: a trademark and redistribution policy |
| PO-15.12 | Codec, DRM and font licences | Covered | S1 13.2 "Codecs" (legal review before M7); D-S7 licensing track; TX-D24; D-S8 note | Stage 1 | Codec exposure review; Widevine/PlayReady/castlabs enquiries; Izcal's own fonts |
| PO-15.13 | Search engine agreements and default search | Partial | D-S8 (deferred); SH 6.8 ("the chosen search engine") | — | Search revenue is one D-S8 funding option, deferred. Missing: how the default engine is chosen, the search-engine list and its policy |

## 16. Security response and operations

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-16.1 | [SB] Vulnerability disclosure policy and intake | Covered | S1 13.2 "Security response"; SEC 6.17 | Stage 1 (before first public binary) | Published policy, security contact, embargo process, machine-readable advisories |
| PO-16.2 | Bug bounty | Deferred (Stage 2) | S1 13.2; SEC-D10 | Stage 2 | "a bug bounty once funding allows"; external audit before first public binary regardless |
| PO-16.3 | CVE assignment (CNA) | Missing | — | — | Searched "CNA", "CVE Numbering", "CVE ID" (hits cite others' CVEs) |
| PO-16.4 | Public security advisories | Covered | S1 13.2; SEC 6.17 | Stage 1 | Advisories in a machine-readable format |
| PO-16.5 | [SB] Fix-time targets | Covered | SEC-D1 (JS-16), SEC-1, SEC-25 | Stage 1 | 72 h† for exploited bugs, 7 days† critical and high; published acknowledgement and fix times |
| PO-16.6 | Security severity rating scale | Partial | SEC-25 | Stage 1 | "Acknowledgement and fix times by severity, published". Missing: the scale itself. Searched "severity scale", "severity rating", "critical, high" |
| PO-16.7 | Coordination and embargoes | Covered | S1 13.2; SEC 6.17 | Stage 1 | Embargo for own fixes and coordinated disclosures, upstream fixes |
| PO-16.8 | Continuous fuzzing and sanitiser builds | Covered | S1 7.5, 7.2 (T2 nightly); SEC-24 | Stage 1 | Fuzz targets registered with OSS-Fuzz; nightly campaigns; pre-release audits |
| PO-16.9 | Release engineering and on-call | Partial | S1 7.8; SEC UPD-R3; JS-16 (24-hour on-call); SEC-D2, SEC-D4 | Stage 1 | On-call rotation, release keys, second release person before the first public binary. Missing: a rollout dashboard and halt switch (see PO-3.7) |
| PO-16.10 | Security notes for enterprise | Missing | — | — | Searched "enterprise notes", "administrator release notes", "security release notes" |

## 17. Back-end services

| # | Item | Status | Owner (spec §, decision) | Lands in | Evidence / what is missing |
|---|---|---|---|---|---|
| PO-17.1 | [SB] Update and download servers and CDN | Covered | SEC 6.16 ("served from storage Izcal does not trust (GitHub or a CDN)"); NW-D17; S1 13.2 "Updates" | Stage 1 | TUF is the only integrity root; public cacheable fetches. Hosting capacity and cost are not estimated |
| PO-17.2 | [SB] Web Push service | Deferred (Stage 2) | S1 13.2 "Service workers"; NW 6.14 | Stage 2 | Self-hosted autopush server "is an operations commitment, so push and background sync are Stage 2"; Q-SEC-12 open |
| PO-17.3 | Sync service | Deferred (Stage 3) | S1 9 ("Sync (if wanted)"); SH 6.12 (Q-SH-5) | Stage 3 | "There is no Izcal account and no sync in this version" |
| PO-17.4 | Extension store and signing service | Deferred (Stage 2) | EX-D10; S1 8.1 Stage 2 seams | Stage 2 | Directory, review, signing service and decision log |
| PO-17.5 | Safe-browsing and reputation service | Covered | PH (`hg-phish-ingest`, `hg-phish-compile`) | Stage 1 | Server-side build pipeline and governance staffed in PH 9.2 |
| PO-17.6 | Network geolocation provider | Missing | — | — | Searched "network geolocation", "Wi-Fi", "location provider", "GeoClue" |
| PO-17.7 | Speech, translation and suggestion services | Partial | ML (Stage 2 translate task); SH 6.8 (search suggestions from the chosen engine, after Enter) | Stage 2 | Local translation is a Stage 2 AI task; suggestions only from the chosen engine. No speech recognition service |
| PO-17.8 | Crash and telemetry ingestion back end | Partial | S1 7.8; D-S4 | Stage 1 | Named only ("Symbol server and crash reporting follow D-S4"). See PO-5.3 and PO-5.7 |
| PO-17.9 | Support site and community | Missing | — | — | Searched "support site", "help centre", "knowledge base", "community forum" |

---

## Summary

### Counts per status per group

| Group | Rows | Covered | Partial | Excluded | Deferred | Missing |
|---|---|---|---|---|---|---|
| 1 Packaging and installation | 17 | 2 | 6 | 0 | 1 | 8 |
| 2 Code signing and trust | 10 | 6 | 2 | 1 | 0 | 1 |
| 3 Updates and channels | 15 | 6 | 4 | 0 | 0 | 5 |
| 4 Component and data updates | 17 | 15 | 1 | 1 | 0 | 0 |
| 5 Crash, telemetry, experiments | 13 | 7 | 4 | 0 | 0 | 2 |
| 6 Enterprise policy | 20 | 3 | 5 | 0 | 0 | 12 |
| 7 Enterprise networking | 17 | 10 | 2 | 1 | 0 | 4 |
| 8 Default browser and shell | 14 | 3 | 6 | 0 | 0 | 5 |
| 9 OS services | 27 | 15 | 6 | 0 | 3 | 3 |
| 10 Graphics, power, display | 18 | 14 | 4 | 0 | 0 | 0 |
| 11 Platforms and sandboxes | 11 | 8 | 1 | 1 | 1 | 0 |
| 12 Installed web apps | 7 | 0 | 1 | 0 | 0 | 6 |
| 13 Embedding and automation | 11 | 4 | 2 | 0 | 2 | 3 |
| 14 Localisation and UI a11y | 8 | 2 | 4 | 1 | 0 | 1 |
| 15 Legal and regulatory | 13 | 3 | 5 | 0 | 0 | 5 |
| 16 Security response | 10 | 5 | 2 | 0 | 1 | 2 |
| 17 Back-end services | 9 | 2 | 2 | 0 | 3 | 2 |
| **Total** | **237** | **105** | **57** | **5** | **11** | **59** |

Ship blockers: the inventory's header says 57, but its table body tags **55** rows [SB]; this mapping uses the 55. They split 30 Covered, 13 Partial, 1 Excluded (PO-14.1), 3 Deferred (PO-9.7, PO-9.14, PO-17.2) and 8 Missing.

### Missing rows (59)

**Ship blockers (8):**
- PO-1.2 Windows enterprise MSI
- PO-1.13 Uninstall, repair and leftover cleanup
- PO-2.2 SmartScreen app reputation (launch-day lead time for a new signing identity)
- PO-6.1 Windows Group Policy (ADMX/ADML)
- PO-6.2 MDM on Windows
- PO-6.3 macOS configuration profiles
- PO-6.5 Linux policy files
- PO-15.9 EU Cyber Resilience Act (its reporting duty has applied since 11 September 2026)

PO-6.1, 6.2, 6.3 and 6.5 share one root cause: no spec defines how policy reaches the browser, although about fifteen specs consume "enterprise policy".

**Other Missing rows (51):**
- Packaging: PO-1.3 MSIX/Store, PO-1.5 Mac App Store, PO-1.11 initial preferences, PO-1.12 install-time options, PO-1.16 VDI/roaming/multi-session, PO-1.17 kiosk
- Updates: PO-3.3 delta updates, PO-3.5 release cadence, PO-3.6 extended support channel, PO-3.7 staged rollout and halt, PO-3.8 version pinning and admin rollback
- Diagnostics: PO-5.4 WER and OS hang reporting, PO-5.9 field trials / experiments
- Enterprise: PO-6.4 Apple DDM (Safari-only analogue), PO-6.6 cross-platform policy file, PO-6.7 cloud management, PO-6.10 managed bookmarks, PO-6.14 enterprise reporting, PO-6.15 DLP connectors, PO-6.17 legacy browser / IE mode, PO-6.19 admin documentation
- Enterprise networking: PO-7.3 OS SSO to cloud identity, PO-7.9 TLS version/cipher policy, PO-7.12 intranet single-word navigation, PO-7.16 OS VPN compatibility
- Shell integration: PO-8.5 enterprise default-browser assignment, PO-8.11 background mode / start at login, PO-8.12 OS scripting hooks, PO-8.13 set as desktop background, PO-8.14 cross-device handoff
- OS services: PO-9.9 share sheet, PO-9.22 speech synthesis/recognition, PO-9.27 parental controls
- Installed web apps: PO-12.1 manifest processing, PO-12.3 PWA uninstall, PO-12.4 OS-level app integration, PO-12.5 enterprise force-installed web apps, PO-12.6 Store packaging, PO-12.7 site shortcuts
- Automation: PO-13.8 testing builds, PO-13.10 single instance, PO-13.11 DevTools enablement policy
- Localisation: PO-14.5 regional builds and defaults
- Legal: PO-15.5 DMA choice screens, PO-15.6 DMA data-portability import, PO-15.7 age assurance, PO-15.10 European Accessibility Act
- Security ops: PO-16.3 CNA status, PO-16.10 enterprise security notes
- Back end: PO-17.6 network geolocation provider, PO-17.9 support site

### Ship blockers by status

| Status | [SB] rows |
|---|---|
| Covered (30) | PO-2.1, 2.3, 2.4, 3.4, 4.1, 4.2, 4.6, 5.1, 7.1, 7.5, 7.7, 7.8, 9.1, 9.2, 9.3, 9.5, 9.10, 10.1, 10.13, 11.1, 11.3, 11.4, 11.6, 11.8, 11.9, 13.3, 15.2, 16.1, 16.5, 17.1 |
| Partial (13) | PO-1.1, 1.4, 1.6, 2.6, 3.1, 8.1, 8.2, 8.3, 9.16, 10.10, 13.7, 14.6, 15.1 |
| Excluded (1) | PO-14.1 (Stage 1 English-only, no later stage set) |
| Deferred (3) | PO-9.7 (notifications, Stage 2), PO-9.14 (printing UI, Stage 2), PO-17.2 (Web Push, Stage 2) |
| Missing (8) | PO-1.2, 1.13, 2.2, 6.1, 6.2, 6.3, 6.5, 15.9 |

### Partial rows and what is missing (57)

- **PO-1.1** installer technology, stub/offline, location, repair. **PO-1.4** MDM `.pkg`. **PO-1.6** signed APT/YUM repository; package-manager vs TUF updater (SEC "Open"). **PO-1.8** Snap decision. **PO-1.10** side-by-side installs and profiles per channel. **PO-1.15** multiple profiles, picker, lock, refresh.
- **PO-2.6** OpenPGP repository/package signing. **PO-2.8** crash-driven injected-module blocklist (broker CIG measured at M5).
- **PO-3.1** update while the browser is closed (task/agent/timer), check schedule. **PO-3.9** admin update policies. **PO-3.13** security-release train. **PO-3.14** release-notes process, enterprise notes.
- **PO-4.9** extension blocklist format and client behaviour.
- **PO-5.3** crash server, symbol store, grouping, regression flags. **PO-5.7** metrics pipeline and metric set. **PO-5.8** deletion of sent data, regional defaults. **PO-5.12** general feedback form.
- **PO-6.8** policy precedence and scopes; `about:policies` vs `izcal://policy` naming. **PO-6.9** one policy catalogue and schema. **PO-6.11** force-install details (Q-EX-5). **PO-6.12** admin URL allow/block list in CB. **PO-6.16** IdP-bound managed profile.
- **PO-7.6** PKCS#11 module loading. **PO-7.17** password-reuse alerts.
- **PO-8.1/8.2/8.3** per-OS default-browser registration. **PO-8.6** registering Izcal as file/scheme handler (EX points at a non-existent "OS-integration design"). **PO-8.7** jump lists, dock menus, badges, progress. **PO-8.9** AMSI/IAttachmentExecute scan.
- **PO-9.8** SMTC/MPNowPlaying/MPRIS adapters, media keys. **PO-9.11** CTAP2 roaming authenticators (Linux); the "credential subsystem" DV names. **PO-9.16** native open/save dialogs and portal. **PO-9.20** location provider. **PO-9.23** game controllers. **PO-9.24** OS wake-lock calls.
- **PO-10.10** Per-Monitor v2 and mixed-DPI moves. **PO-10.16** Wayland decorations, Spaces, materials. **PO-10.17** swipe navigation, pen, trackpad gestures. **PO-10.18** unplugged monitors, per-monitor full screen.
- **PO-11.2** Windows on Arm as a named target.
- **PO-12.2** an owner for installed web apps (PH already assumes them).
- **PO-13.7** a headless mode. **PO-13.9** supported command-line switches.
- **PO-14.2** language packs, admin locale. **PO-14.3** mirrored RTL shell. **PO-14.4** translator tooling, string freeze. **PO-14.6** published ACR/VPAT.
- **PO-15.1** terms of use. **PO-15.3** subject access for server-held data. **PO-15.4** deletion of sent data. **PO-15.11** trademark policy. **PO-15.13** default search engine choice and list.
- **PO-16.6** severity scale. **PO-16.9** rollout dashboard and halt.
- **PO-17.7** speech services; translation is Stage 2 local. **PO-17.8** ingestion back end.

*(Partial rows by group: 6 + 2 + 4 + 1 + 4 + 5 + 2 + 6 + 6 + 4 + 1 + 1 + 2 + 4 + 5 + 2 + 2 = 57.)*

### Proposed grouping into new specs or amendments

1. **Release engineering, distribution and updates — new spec ("Izcal distribution").** PO-1.1–1.6, 1.8, 1.10–1.13, 2.2, 2.6, 3.1, 3.3, 3.5–3.9, 3.13, 3.14, 13.8, 16.9, 17.8 (release half). *Reason:* S1 7.8 and SEC 6.16 specify the trust chain (TUF, signing, attestations) but nothing between "a signed binary exists" and "it is on a machine and stays current": installer forms, Linux repositories vs the TUF updater (SEC leaves it Open), uninstall, cadence, rollout and halt, delta payloads and SmartScreen lead time.
2. **Enterprise management — new spec ("Izcal enterprise policy").** PO-1.2, 1.4 (pkg), 1.16, 1.17, 3.8, 3.9 (admin half), 6.1–6.17, 6.19, 7.9, 8.5, 12.5, 13.11, 16.10. *Reason:* about fifteen specs already name policies (NW 6.14, B10, DV, JS-D13, EX, AH, PH, SH 6.9), and S1 1 gives the broker "profile and policy", but no spec defines the schema, the per-OS sources (ADMX, MDM profiles, `/etc` JSON), precedence or the policy page; one schema should generate every template, as the inventory suggests. SEC-D14's Stage 2 per-machine trigger belongs here.
3. **OS integration (shell plumbing) — new spec or a large shell amendment ("Izcal OS integration").** PO-8.1–8.3, 8.6, 8.7, 8.9, 8.11–8.14, 9.8, 9.9, 9.16, 9.20, 9.24, 9.27, 10.10, 10.16–10.18, 11.2, 13.9, 13.10. *Reason:* EX already defers handler keys to an "OS-integration design" that does not exist; default-browser registration, handler registration, native file dialogs, jump lists, wake locks and media-control adapters are per-OS broker work that no spec owns, and the inventory's "every row is three OS integrations" holds.
4. **Installed web apps — amendment to the app host spec, or a short new spec.** PO-12.1–12.7. *Reason:* PH 6.4.9 already checks "installed web apps", and AH treats PWAs as the incumbent its `.izapp` model must beat, yet no spec owns manifest processing, install, uninstall or OS integration of web apps; the inventory's advice is to specify them once with the app host's shims.
5. **Diagnostics, crash and telemetry operations — amendment to S1 7.8 plus a short new spec.** PO-5.3, 5.4, 5.7, 5.8, 5.9, 5.12, 17.8. *Reason:* D-S4 decides consent but there is no ingestion server, symbol store, metric set or WER/hang integration; PO-5.9 needs an explicit decision (Izcal's B10/P1 posture may make "no server-side experiments" the right non-goal, but it must be recorded).
6. **Legal, regulatory and compliance — new spec ("Izcal compliance").** PO-15.1, 15.3–15.7, 15.9–15.11, 15.13, 16.3, 14.6 (ACR). *Reason:* S1 13.2's "Privacy and legal" row has three duties; the CRA reporting duty is already in force and the CRA's support period and conformity assessment touch release engineering; DMA choice-screen eligibility, the EAA and age signals need one owner.
7. **Localisation — amendment to the shell spec (SH-D24, T12).** PO-14.1–14.5. *Reason:* Fluent strings are designed, but Stage 1 is English-only with no later stage, locale count or RTL mirroring; PO-14.1 is a ship blocker for public-sector buyers, so the deferral needs a stage.
8. **Automation and headless — amendment to the DevTools spec.** PO-13.7, 13.8, 13.11. *Reason:* WebDriver, BiDi and CDP are well covered; a headless mode, pinned testing builds and an admin DevTools switch are the missing edges.
9. **Networking amendments.** PO-7.3, 7.6, 7.12, 7.16, 7.17 (password reuse may belong to PH). *Reason:* NW covers enterprise networking deeply; these five are small additions to NW 6.1/6.13/6.14 and the address-bar section of SH 6.8.
10. **Credentials and OS services amendments.** PO-9.11 (CTAP2 "credential subsystem"), 9.22 (speech), 9.23 (gamepad), 17.6 (geolocation provider), 17.9 (support site). *Reason:* each is a named dependency of an existing spec (DV, DM, media) without an owner.

### Non-MIT data sets the inventory flags, and what the specs decide

| Data set | Licence (inventory) | Decision in the specs |
|---|---|---|
| Public Suffix List | MPL-2.0 | **Shipped** under the standards-data exception (H2 M-1 2.3.2, OD-L6; S1 13.2 "Shipped data"); a static global of the network system service (NW-D7) |
| Unicode CLDR, UCD, emoji data, tz | Unicode licence / public domain | **Shipped as standards data** under the same exception; Izcal writes its own Unicode algorithms (`hg-ucd`, `hg-unicode`), no ICU code |
| Hunspell-format dictionaries | Mixed (LGPL, GPL, MPL…) | **Excluded** (ED M-1 note: "Hunspell and its dictionaries … are therefore out"); Izcal's own lexicons shipped in the engine image (ED-D7, ED-D22), interim OS spelling tier (ED-D8) |
| Vendor root store lists (Mozilla/NSS, CCADB, Chrome) | Vendor terms | **Not shipped.** Izcal runs its own root program (NW-D5); CCADB records inform but are "not shipped or copied as a data set"; OS stores are a development-build interim and a labelled first-release fallback (NW-D24) |
| HSTS preload list | Chromium BSD (unchecked in inventory) | **Not used.** Izcal builds its own list from operator requests and CT (NW-D25) |
| CT log list | Not checked in inventory | **Shipped as standards data** (H2 M-1 2.3.2 lists "the CT log list"), filtered by Izcal's log policy (NW 6.5.1) |
| Safe-browsing lists (Google Safe Browsing) | Vendor terms | **Not used.** Izcal's own bundle; Phishunt (CC0) ships; CERT Polska and URLAbuse only if the MIT audit allows; PhishTank, URLhaus and others test-only; malware URLs a documented gap (SEC-D7, PH-D14) |
| Tracking/filter lists (EasyList, EasyPrivacy) | GPL-3.0 / CC BY-SA 3.0 | **Not shipped.** Izcal's own lists (NW-D26, CB-D13) |
| Fonts and emoji | Mixed (OFL etc.) | Izcal's own font set (TX-D24) as one signed package; emoji compiled from Microsoft Fluent Emoji (MIT) plus Izcal artwork; Noto Color Emoji not chosen |
| On-device ML models and training data | Mixed | Every shipped weight and data set MIT-compatible, tracked in a data ledger; a weight without a ledger row is rejected at build (ML-20, ML-23) |
| GPU driver blocklist | (vendor lists) | Built from Izcal's own device lab and vendor advisories (GR-D19) |
| DRM CDMs (Widevine, PlayReady, FairPlay) | Commercial | Used only as **OS-supplied or licensed components** (protected media mode, Windows PlayReady); never shipped as Izcal data; Linux Clear Key only until licensed (D-S7) |
| H.264/AAC/HEVC decoders | Patent pools | **Not shipped**; OS decoders only; no OpenH264 (S1 13.2 "Codecs") |

### Contradictions and loose ends found

- **Policy page name:** SH 6.9/6.14 call it `about:policies`; NW 6.14 calls it `izcal://policy`.
- **Linux updates:** S1 13.2 ships `.deb`/`.rpm` (normally updated by the system package manager) while SEC-D14 and UPD-R2 assume Izcal's own per-user TUF updater writes the install directory; SEC 6.5 lists "distribution packages" as Open.
- **A phantom spec:** EX's manifest table defers `protocol_handlers` and file-handler keys to "Stage 3, with the OS-integration design"; no such design exists.
- **A phantom subsystem:** DV says "FIDO/CTAP remains in the credential subsystem (WebAuthn)"; no spec defines CTAP2 transport, which Linux passkeys need.
- **Installed web apps** are assumed by PH 6.4.9 but owned by no spec.
- **Inventory self-count:** the inventory's header says 57 ship blockers; its table body tags 55 rows [SB].
