# Izcal enterprise policy

*First-principles specification of how enterprise policy reaches Izcal. It covers one policy catalogue from which every template and reference page is generated; typed, versioned policies with a published total precedence; delivery through Windows Group Policy and MDM, macOS configuration profiles and Linux policy files, read by a sandboxed policy adapter and sent to the broker as typed snapshots; an optional signed remote policy document in place of a vendor console; the `about:policies` page and an exact management disclosure; and the policy-driven product areas: extensions and apps, managed bookmarks, networking, DevTools and automation, kiosk mode, legacy-browser switching, reporting, DLP, single sign-on, installs and updates. It also covers how policy enters the deterministic simulation and the recordings. Date: 24 October 2026. Status: draft 0.4. Research notes: `research_notes/Izcal enterprise policy/` (`history_postmortems.md`, `os_delivery_formats.md`, `landscape_admin_gaps.md`; phase 8: `factcheck.md`, `critique.md`). Work item: **FI-3** of the [feature inventory audit][FI], which closes 29 ledger rows. Relates to: HGBA-2 [M-1, P1, P2, I-1, I-2]; HGBA-3 [B1, B2, B9, B10, D13]; Stage 1 design [§1 process map, 2.1–2.2, §3 substrate, 3.4, §4 IPC, 8.1, 8.3, 13.2]; security scope [ATK-7, IPC-R2, IPC-R15, REC-R2, SEC-D14, Q-SEC-18]; shell [6.9, 6.11, 6.14, 6.17]; networking [6.1, 6.13, 6.14, 6.16, NW-D20]; extensions [6.6, 6.11, 6.12, Q-EX-5]; app host [6.7, A19]; devices [6.10, 6.15]; JavaScript engine [JS-D13]; ML service [ML-D4]; phishing protection [6.13, Q-PH-15]; content blocking [6.10, 6.12]; DevTools [automation gateway]; storage [6.x managed areas]; resource footprint [FP-1, FP-18, 8.2]; WebCrypto [WC-D7]; the audit's FI-D9, FI-D11, FI-D12, FI-D14, FI-D17 and §8 points 1–2.*

*Revision 0.1 (24 October 2026): first draft, written by the synthesis author from the three research notes and the existing specs. No other report is edited; the amendments in 9.5 are proposals.*

*Revision 0.4 (25 October 2026): **the owner accepted EP-D1 to EP-D19 as recommended** (EP-D2, EP-D4, EP-D8 and EP-D10 reviewed explicitly), and 9.3 marks each one. The owner also set OI-D18 (OS scripting) to "full scripting behind consent and debug mode", so this spec adds **EP-D20**: a fourth, separate automation policy, `OsScriptingAvailability`, which can force OS scripting off or restrict it to an allow-list of calling apps, and can never force it on. It is added to the catalogue with its `risk_flag` (6.2, 6.14, 6.21), with a slice to its enforcer (6.6), target EP-27 and amendment A29. G240–G249 are unchanged.*

*Revision 0.3 (25 October 2026): the owner's decisions of 25 October 2026 are recorded.*
- ***OS inputs, "both, split by source".*** *Policy keeps this spec's pattern: it is parsed in the sandboxed adapter and delivered as typed `.hgidl` snapshots with the 32-byte `PolicyDigest`, with no substrate trait for policy. Separately, the OS integration spec's broker-only `OsPort` trait (OI-D1) carries what the OS delivers to the broker itself (launches, default status, settings, displays, power) and **no longer carries policy**. 6.21 now says so.*
- ***The shared header.*** *It has one definition, owned by FI-11, and 6.21 lists the union of this spec's, the OS integration spec's and the compliance spec's (§9.4) fields. Crash reports carry **coarse policy flags**, not the digest, as the compliance spec decided.*
- ***Distribution's cadence.*** *Stable ships every 4 weeks (later 2 weeks), and public signed Nightlies come at least 3 months before Stable once an EU legal entity exists. Deprecation is now measured in time, not releases, and the Nightly policy preview depends on public Nightlies.*
- *G240–G249 are unchanged.*

*Revision 0.2 (24 October 2026): phase 8 applied.*
- *Fact-check (`factcheck.md`): 67 claims, 41 correct, 17 partly correct, 6 wrong, 3 unverifiable. Every wrong and partly-correct row is fixed, and the unverifiable rows stay graded Medium. The link check passed.*
- *Design critique (`critique.md`): 24 findings (5 High, 13 Medium, 6 Low), all accepted. Dispositions are in Appendix A.*
- *The reserved `policy.hgidl` now passes M1's merged `hg-ipc-codegen` checker, run as a library on this revision's text (6.6).*
- *The coordinator accepted the M1-exit direction on 24 October 2026: IPC snapshots from an OS-adapter process, a 32-byte `PolicyDigest` in the shared header, and a reserved `policy.hgidl`. The coordinator also assigned this spec G-rules **G240–G249**.*

About fifteen Izcal specs consume "enterprise policy". The shell marks rows "Set by your organisation", networking lists ten policies, devices define six keys and a "signed policy" pre-grant, B10 lets an administrator force AI off, and S1 gives the broker "profile and policy" and a crate called `hg-policy`. **None of them says how a policy reaches the browser.** This spec does. The central idea is that **policy is data with one schema and one meaning on every OS, derived fresh from the sources present now, never written into the person's settings, and visible where it applies**. Four decisions matter most. **(1) One catalogue** (EP-D1) generates the ADMX and ADML templates, the macOS profile manifest, the JSON Schema, the admin reference, the `about:policies` metadata and the Rust types. A CI check binds it to the shell's settings registry, so a policy cannot exist without a documented setting or a documented reason. **(2) Only locations an administrator alone can write are policy sources** (EP-D3), and Izcal checks that when it opens them. Adware with elevated installers got around Chrome's extension rules by writing the force-install policy keys under HKLM and re-adding them every four hours ([Malwarebytes, 2022](https://www.malwarebytes.com/blog/news/2022/06/forced-chrome-extensions-keep-reappearing); [HP 3.5][HP]). That attacker is an administrator and out of scope (SEC 6.1). Same-user malware (ATK-7) is in scope and could do the same through any user-writable location, so a per-user install directory or a user-writable Flatpak path is never a source. **(3) A sandboxed policy adapter, launched on demand, reads and parses every source and sends the broker typed snapshots over `.hgidl`** (EP-D7). The broker parses nothing (IPC-R15) and merges in a pure function. Each process records the policy it receives like any other IPC input. That is also the **M1-exit input** (6.21): a 32-byte `PolicyDigest` in the shared recording and crash-report header, and no new substrate trait. **(4) Open protocols first** (FI-D14): Izcal operates no management console and speaks no vendor's cloud protocol. Where admins want one, an organisation publishes a **signed remote policy document** on any web server, under a protocol Izcal publishes in prose (EP-D8).

## 1. The problem and who has it

| Who | Task | What goes wrong today |
|---|---|---|
| **IT administrators** (enterprise, education, public sector) | "Configure every browser in the organisation the same way, prove it is configured, and find out why it is not" | Policy that silently does not apply is the recurring complaint. The highest source wins and "all other policies are ignored". Policies are blocked by device state or profile type, and ADMX template versions drift; each shows up only as "Ignored" or "Error" on an internal page ([LG 1.1–1.5][LG]) |
| **People using a managed browser** | "Know what my organisation controls and what it can see" | Chrome discloses with one phrase, "Managed by your organization"; the fields its reports send are listed only in admin docs ([LG 3.1, 3.4][LG]) |
| **Procurement and compliance** | "Show that the browser meets our baseline" | BSI's Mindeststandard requires that a base configuration can be applied centrally and that cloud sync can be disabled centrally (MUSS). Telemetry and DoH must be disableable, and it says central enforcement of the telemetry setting and a centrally managed extension allow-list should exist (SOLLTE) ([LG 10.1][LG]; factcheck row 39) |
| **Izcal engineers** | "Reproduce a bug from a managed machine" | Policy is an input from outside the simulation. If a recording does not say which policy was in force, a crash on a managed machine cannot be reproduced (audit §8 point 1) |
| **Every Izcal spec that names a policy** | "Rely on a delivery path" | No spec defines one (FI-3) |

**What the existing specs already decide or constrain:**
- **SH 6.9.** Each registry row records "whether it can be set by policy". Policy-set rows are "visible, marked 'Set by your organisation', and cannot be changed", with "the policy's source and the administrator's contact" shown. **"A policy can set only registry settings, never an arbitrary preference."** There is no `about:config`.
- **SH 6.14.** `about:policies` lists "enterprise policies in force and their sources"; the page is local and unlisted.
- **H3 B10, D13, ML-D4.** Administrators may **force off and lock** `ai.enabled` or `accessibility.model`, **never force on**.
- **NW 6.14.** Ten policies: DoH mode and resolver; QUIC; ECH; proxy configuration and lock; the ambient-authentication allowlist (NW-D20); LNA defaults; added trust anchors and client-certificate auto-selection; HTTPS-First exceptions; the content-blocking list set; Web Push allowed. These are "applied by the broker per container" and shown on `izcal://policy`, which is contradiction C1.
- **DV 6.10, 6.18.** Six device-family keys, plus `DevicePreGrantEnabled` and pre-grants by "signed policy", shown as "Set by your administrator" and still revocable by the person.
- **JS-D13, AH A19.** Per-URL JIT ceilings and app ceilings are policy inputs.
- **EX 6.6, 6.12.** `storage.managed` is "enterprise policy read by the broker's configuration reader". Enterprise packages come from the organisation's "own signed source… labelled 'installed by your organization'", but enterprise installs also "verify the platform signature".
- **AH 6.7.** Enterprise policy is "a signed policy that allowlists apps and pins versions".
- **PH 6.13.** A policy may add blocks but never switch the check off (Q-PH-15).
- **WC-D7.** WebCrypto has no enterprise policy.
- **SEC-D14.** Per-user installs only in Stage 1, with "Enterprise deployment (Stage 2)" as the reversal trigger.
- **S1 §1.** The broker owns "profile and policy". An **OS-adapter process** "reads OS documentation-sourced network configuration"; NW 6.1 says the broker reads proxy and resolver settings "as today". These conflict (Section 11.3). FP 8.2 budgets the OS adapter as **on demand**: ≤ 2 MiB† idle, exiting after 10 s†. FP-1 has 1 MiB† left.
- **SEC 6.1.** An attacker with administrator rights is out of scope. **ATK-7, same-user malware, is in scope.**
- **FI-D9.** This spec is written before M5. Per-machine installs, ADMX, MDM, configuration profiles and Linux policy files ship in **Stage 2**.

**Success, measurably:**
- Every policy any spec names is in one catalogue, and CI fails on a policy that is used but not catalogued (EP-1).
- The same effective policy and the same digest result from equivalent sources on all three OSes (EP-3).
- An administrator can find why a policy did not apply from `about:policies` alone (EP-9, EP-10).
- No policy is taken from a location the policed user can write (EP-6).
- Every recording names the policy in force, and every crash report carries coarse policy flags (EP-15).

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1996 | **Windows NT 4.0 system policies** (`.pol`, ADM) | Wrote registry values anywhere | One machine-wide registry, no policy namespace | **Tattooing**: a policy "persists until the value is intentionally reversed"; removing the policy file did nothing ([HP 1.1–1.2][HP]) |
| 2000 | **Windows 2000 Group Policy** | Confined policy to four ACL-protected `…\Policies` trees, "wiped clean and the new policies are rewritten" on every change; policy and preference kept in separate keys | To make policy removable, and to restore the person's preference when the policy goes ([HP 1.3–1.5][HP]) | Survives today as the contract every browser reads |
| 2002–2011 | **Apple MCX → configuration profiles** | Managed preferences with Once/Often/Always, then `.mobileconfig` profiles from OS X Lion | Directory-based management gave way to device management | Profiles survive; the per-key Once/Often semantics were partly lost ([HP 5.1–5.2][HP], Low) |
| 2006–2008 | **ADMX/ADML** (Vista, Server 2008) | XML definitions, language files apart, one central store | Per-policy-object copies of ADM files bloated SYSVOL replication ([HP 1.7–1.8][HP]) | Survives; mixing template generations causes known errors ([HP 1.9][HP]) |
| 2010 | **Chrome for Business** | MSI, Group Policy, ADM/ADMX templates | Enterprise deployment (inferred; [HP 3.1][HP], Medium) | The model every browser copied |
| 2013 | **MS-MDM** (OMA-DM subset) | Windows MDM over SyncML for enrolled devices; later the Policy CSP and ADMX ingestion | Management of enrolled devices without a domain (inferred; [HP 6.1][HP]) | Survives; the registry key stays the contract, carried over a new transport ([HP 6.3][HP]) |
| 2014 | **MS14-025** | Group Policy Preferences stored passwords encrypted with a key Microsoft's own specification prints, in a folder every domain user can read | Not stated by the sources | Exploited in targeted attacks; the fix "does not remove any existing GPOs" holding passwords ([MS14-025](https://learn.microsoft.com/en-us/security-updates/securitybulletins/2014/ms14-025); [HP 1.10–1.12][HP]) |
| 2014–2015 | **Chrome's off-store extension block** | From May 2014 Windows Chrome installed only store extensions, **with enterprise policy exempt** | Malware silently installed ad-injecting extensions; support requests about them fell 75% ([HP 3.2–3.3][HP], Medium) | The exemption became the attack: installers with administrator rights wrote the force-install policy under HKLM and re-applied it every 4 hours ([HP 3.5][HP]). Edge now honours off-store force-install only on AD-joined Windows machines and on MDM- or MCX-managed Macs; Chrome's wording was not read ([HP 3.4][HP]) |
| 2014–2022 | **Enterprise Mode → IE mode** | An admin site list sent named sites to IE8-compatible, then Trident-in-Edge rendering | Enterprises had 1,678 legacy apps on average; ActiveX ([HP 2.1–2.4][HP]) | The IE11 desktop app retired on 15 June 2022 on certain Windows 10 versions; IE mode is supported "through at least 2029" ([Microsoft lifecycle FAQ](https://learn.microsoft.com/en-us/lifecycle/faq/internet-explorer-microsoft-edge); [HP 2.4–2.5][HP]). The site list outlived the engine it served |
| 2018 | **Firefox policy engine** (Firefox 60 ESR) | Declarative `policies.json`, GPO and macOS support, replacing AutoConfig (JavaScript) and CCK2 | Enterprises asked for Group Policy, and AutoConfig was to be sandboxed ([HP 4.1–4.2][HP]) | Survives. Until Firefox 87 (March 2021) GPO **replaced** `policies.json`; now both merge, with GPO winning ([Firefox 87 release notes](https://support.mozilla.org/en-US/kb/firefox-enterprise-87-release-notes); [OD 6.7][OD]) |
| 2019 | **Chrome Browser Cloud Management** → Chrome Enterprise Core (2024) | A vendor console with enrolment tokens; no domain needed | Cross-OS management for organisations without AD ([HP 3.7–3.8][HP], Medium; [LG 2.1–2.3][LG]) | Free, proprietary; the Edge management service and, from October 2026, Firefox Enterprise's self-hosted console followed ([LG 2.4–2.5][LG]) |
| 2021– | **Apple declarative device management** | Declared state, atomic activations, a status channel, no polling | Apple called MDM "imperative and reactive" ([HP 5.3–5.4][HP]) | Growing; an app-config declaration exists on macOS 26, but its reader framework lists no macOS version (inferred from availability metadata, [OD 4.11–4.12][OD], Medium) |
| 2022 | **Kiosk escapes** | NVISO left Edge kiosk mode through an `ftp://` app prompt, file dialogs and a renamed `powershell.exe` | Kiosk modes hide the UI over a general browser | Microsoft says open and save dialogs "aren't automatically locked down" ([NVISO](https://blog.nviso.eu/2022/05/24/breaking-out-of-windows-kiosks-using-only-microsoft-edge/); [Edge kiosk docs](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-configure-kiosk-mode); [HP 7.3–7.4][HP]) |
| 2025 | **Firefox `ContentAnalysis`** (138, per Mozilla Hacks; the policy reference's compatibility line is inconsistent) | Implements Google's DLP agent SDK protocol so one agent serves both browsers | DLP vendors otherwise inject DLLs, which causes crashes ([LG 4.2, 4.5][LG]) | The protocol is defined only by the SDK's code ([LG 4.4][LG], Medium) |

### 2.2 Technology mismatches: assumptions that no longer hold

| Inherited assumption | Why it was true | Why it fails for Izcal |
|---|---|---|
| A local, admin-writable key means "managed" | In 2000, few home machines had malware with admin installers | Adware runs elevated installers and writes HKLM ([HP 3.5][HP]). Administrators are out of scope (SEC 6.1). Same-user malware (ATK-7) is in scope, and per-user install directories are user-writable, so a user-writable policy location would hand it the same channel without elevation |
| Policy can be anything a preference can be | Firefox's `Preferences` policy and `mozilla.cfg` set arbitrary prefs ([OD 5.9][OD]; [HP 4.1][HP]) | Izcal has no arbitrary preferences (SH 6.9) and no `about:config` |
| The browser reads policy in its main process | Chrome's and Firefox's architectures | Izcal's broker parses no web-format bytes (IPC-R15), and only `hg-substrate-os` may make ambient OS calls (G3) |
| Polling refreshes policy | Group Policy refreshes every 90 minutes, Edge's cloud every 90 minutes or 24 hours ([OD 1.12][OD]; [LG 2.4][LG]) | Izcal's own code may run no periodic timers, and the whole browser gets at most 1 idle wake-up per second (FP-18); FP-M0 cut an empty sandboxed process from 2 wake-ups per second to 0. Refresh must ride OS notifications or an existing wake grid |
| Precedence can be "undefined" | Chrome documents a same-level file conflict as "undefined" ([OD 5.2][OD]) | Izcal's runs must be deterministic, and a recording must replay |
| A cloud console is the vendor's | Chrome, Edge and Firefox all ship one ([LG 2.7][LG]) | Izcal has no accounts or servers for this, and FI-D14 puts open protocols first |
| Kiosk mode means hiding UI | Every incumbent | The escapes are OS hand-offs, and Izcal's broker already mediates every one of them |

## 3. Today: the landscape and its gaps

**No open standard delivers browser policy.** The open, documented layers are the OS channels: Windows Group Policy and the MDM Policy CSP with ADMX ingestion, Apple's configuration profiles, and files on disk. Every cloud console is proprietary and documented only by its admin-facing behaviour ([LG 2.7–2.8][LG]). Policy shapes converge on Chromium's de facto: Firefox models `ManagedBookmarks` on Chrome's, and Edge mirrors Chromium's names ([LG 2.9][LG]).

**What admins cannot see:**
- **Why a value was ignored.** Precedence, device-state gates and profile-type gates surface as a bare "Ignored" or "This policy is blocked" ([LG 1.1–1.4][LG]).
- **Whether two OSes mean the same thing.** Chrome's FAQ maps mandatory to HKLM and recommended to HKCU, which conflicts with the `Recommended`-subkey convention Edge documents. Firefox has no recommended level and uses per-policy `Locked` flags ([OD 1.7–1.8, 1.10][OD]).
- **What the user was told.** Reporting is detailed: per-extension permissions, machine serials and security events, every 3–24 h. The user's disclosure is one phrase ([LG 3.1–3.4][LG]).
- **What changed.** Chrome's and Edge's per-policy pages carry version and refresh flags, but no browser explains a change in force to the person or the admin at the moment it happens.

**What Izcal can do that incumbents cannot:**
- **Explain every value.** The broker computes effective policy in one pure function, so every displaced value has a recorded rule ("overridden by machine mandatory", "blocked: source user-writable", "invalid: 3 of 40 entries dropped").
- **Lock by capability.** Kiosk, DevTools and DLP decisions sit on broker chokepoints (file grants, print, clipboard, downloads, external protocols), not on hidden UI.
- **Replay a managed session.** Each process receives its policy as recorded IPC input, so a recorded tab or shell session replays under the policy it saw. A broker crash is matched to its policy by digest.
- **Disclose exactly.** One catalogue generates both the reporter and the disclosure page, so the two cannot disagree.

## 4. First principles

### 4.1 Users' real tasks

1. **Set** a value for every browser on a set of machines or users, either fixed (mandatory) or as a starting default (recommended).
2. **Verify** the value is in force, and see **why not** when it is not.
3. **Restrict** what can run: extensions, apps, DevTools, automation, sites.
4. **Connect** the browser to the organisation's network: proxy, Kerberos, certificates, DNS, single sign-on.
5. **Know**, as the person at the keyboard, what the organisation controls and sees.
6. **Remove** management and get the person's own settings back.

### 4.2 Platform primitives

- **Windows:** the registry under `HKLM`/`HKCU\Software\Policies\<vendor>`, written by Group Policy, by MDM ADMX ingestion or by scripts. Change notification comes through `RegisterGPNotification` and `RegNotifyChangeKeyValue` ([OD 1.12–1.14, 3.1–3.3][OD]).
- **macOS:** managed preferences for the app's domain, delivered by a `com.apple.ManagedClient.preferences` profile on the device or user channel. `CFPreferencesAppValueIsForced` tells forced from default ([OD 4.1–4.3][OD]).
- **Linux:** root-owned JSON files under `/etc` ([OD 5.1–5.3][OD]).
- **HTTPS and signatures:** any web server can serve a signed document.
- **Izcal's own:** typed `.hgidl` endpoints with broker-bound identity (`CallerIdentity`, IPC-R2), the substrate's recorded IPC inputs (S1 3.4), the settings registry (SH 6.9), and capability chokepoints in the broker.

### 4.3 Izcal's principles that bind this design

M-1 (no third-party source as evidence; Izcal's own implementation); P1 and I-2 (no ambient authority; the broker parses no untrusted bytes, IPC-R15); B1 (every input from outside the simulation is recorded); B10 (AI off by default; force-off only); D-S4 and FI-D12 (usage telemetry opt-in; crash reports opt-out); FI-D14 (open protocols first); FI-D17 (profiles share one broker per OS user); FP-1 and FP-18 (the always-on budget; no periodic timers in Izcal's own code).

### 4.4 Assumptions kept and broken

| Assumption | Keep or break | Why |
|---|---|---|
| Policy lives in the OS's admin channel (registry, managed preferences, `/etc`) | **Keep** | Every MDM, GPO and configuration tool already writes there ([LG 2.7][LG]) |
| Policy and preference are separate; removing policy restores the preference | **Keep** (non-tattooing) | The 2000 design was right ([HP 1.5][HP]) |
| Mandatory and recommended levels | **Keep** | Admins use both; Edge and Chrome expose both ([OD 1.8–1.9][OD]) |
| Machine scope above user scope | **Keep** | The machine owner's intent outranks a per-user assignment ([OD 6.1–6.3][OD]) |
| Admin-writable local data is trustworthy | **Break**: check writability when opening, and disclose powerful policies | Abuse of the policy channel is documented ([HP 3.5][HP]); ATK-7 would use any user-writable location |
| A policy file next to the binary | **Break** for per-user installs | That directory is user-writable (SEC-D14) ([OD 7.2][OD]) |
| Arbitrary preferences can be policy | **Break** | Only catalogued policies exist (SH 6.9) |
| Policy may be script | **Break** | Mozilla's own move away from AutoConfig ([HP 4.2][HP]) |
| Secrets in policy | **Break**: no secret-typed value exists | MS14-025 ([HP 1.10–1.12][HP]) |
| Undefined conflict resolution | **Break**: one published total order | Determinism, explainability |
| Vendor cloud console | **Break**: a signed document anyone can host | FI-D14; no open standard exists ([LG 2.7][LG]) |
| Kiosk mode means hidden UI | **Break**: a capability-reduced profile | The escapes come through OS hand-offs ([HP 7.4][HP]) |
| Policy can switch anything on | **Break** for AI, Izcal-bound telemetry and phishing protection | B10, D-S4 and PH 6.13 allow only the protective direction. Policy may weaken some other protections that NW 6.14 lists for enterprise needs (HTTPS-First exceptions, ECH, QUIC, DoH mode, tracking-protection level, GPC, LNA defaults). Each is shown on `about:policies` and in the site panel as "weakened by your organisation". Partitioning and D-S1 have no catalogue entry (CB 6.12) |

## 5. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| NT 4.0 policies | Tattooed the registry ([HP 1.1][HP]) | Policy must be removable state | Effective policy is recomputed from live sources and never written into settings (EP-5) |
| Windows 2000 GP | A separate namespace, wiped and rewritten on change ([HP 1.3–1.5][HP]) | Separate layers plus a full recompute work | Every source change triggers a full, pure re-merge (6.4) |
| MS14-025 | Passwords leaked through a published key; old data stayed deployed ([HP 1.10–1.12][HP]) | No secrets in policy; a fix must purge | No secret type; credentials come from OS stores or a consent path (6.1) |
| ADM → ADMX | Definitions moved out of the values and strings out of the definitions; mixed generations break ([HP 1.7–1.9][HP]) | Generate templates once, versioned | One catalogue generates every template, with a versioned namespace (6.23) |
| Chrome 2014–2022 | The enterprise exemption became the malware channel ([HP 3.2–3.6][HP]) | Writable by an admin is not the same as managed | Admin-only sources checked at read time; high-impact policies disclosed at the moment they change (EP-D3, EP-D4) |
| AutoConfig → `policies.json` | Executable config replaced by declarative data ([HP 4.2][HP]) | Policy is data | No script, no `Preferences` escape (6.1) |
| Firefox 87 | Only after 87 did `policies.json` merge with GPO instead of being replaced by it ([OD 6.7][OD]) | Precedence must be stated, not discovered | A published total order (6.4) |
| Chrome "undefined" | Same-level file conflicts were left undefined ([OD 5.2][OD]) | Leaving a conflict undefined is an anti-pattern | Lexical order and a reported conflict (6.4) |
| Edge "blocked" policies | Device-state gating misclassified editions ([LG 1.3][LG]) | Gates fail silently | Every gate has a named reason on the page (6.9) |
| Apple DDM | Desired state, atomic activations, a status channel ([HP 5.3–5.4][HP]) | Atomic apply plus status | Snapshots apply atomically; per-policy status is exported (6.4, 6.9) |
| ADMX ingestion | The same registry contract over a new transport, with path rules ([HP 6.3][HP]; [OD 3.3][OD]) | Keep the schema independent of the transport | One schema, many sources, one parser per format (6.3) |
| Intune ADMX import | ≤ 1 MB, en-US only, no combo box, a new namespace per version ([OD 3.5][OD]) | Templates must fit the importers | Generated ADMX meets these limits (EP-2) |
| IE mode | The site list outlived the engine ([HP 2.5][HP]) | Compatibility lists last for decades | Site switching to another browser, from the published v.2 schema (6.16) |
| Edge kiosk | Escapes via dialogs and protocols ([HP 7.3–7.4][HP]) | Hiding UI is not a boundary | The kiosk profile removes the hand-offs (6.15) |
| DLL-injected DLP | Crashes and bypassed protections ([LG 4.5][LG]) | No supported interface means injection | Broker verdict hooks are reserved; injected modules are refused (6.18) |
| Chrome reporting | Rich data, one disclosure phrase ([LG 3.1–3.4][LG]) | Disclosure must be exact | Disclosure is generated from the reporter's schema (6.17) |

## 6. Design

### 6.0 Architecture at a glance

```
 OS sources (admin-only)                       organisation endpoints (Stage 2, optional)
  Windows HKLM/HKCU\Software\Policies\Izcal\Browser   remote document + .sig, site list,
  macOS managed prefs dev.izcal.browser (device, user)  reporting upload
  Linux /etc/izcal/policies/{managed,recommended}      via the organisation container (6.10)
            |                                              |
  broker: hg-substrate-os::policy watch API                | bytes routed by the broker
  (existence + change notifications only, parses nothing)  | (IPC-R16 entries, 6.10)
            | launches on demand                           v
  +--------------------------------------------------------------+
  | Policy adapter (hg-os-adapter-main, Policy configuration)     |
  |  hg-substrate-os::policy reads: registry, CFPreferences, /etc |
  |  ownership and ACL checks on the opened object                |
  |  hg-policy-format: JSON, registry JSON, site-list XML (via    |
  |  hg-xml-syntax), Ed25519 verification of the remote document  |
  |  validates against hg-policy-catalogue; sends snapshots;      |
  |  exits 10 s† after the last request                           |
  +-----------------------------|--------------------------------+
                                | policy.hgidl (bound endpoint, CallerIdentity)
                                v
  +--------------------------------------------------------------+
  | Broker: hg-policy (pure merge) -> EffectivePolicy + status   |
  |  canonical encoding -> PolicyDigest -> recording/crash header|
  |  slices from the catalogue's enforced_in map (6.6)           |
  +-----------------------------|--------------------------------+
        typed slice messages on existing endpoints, recorded at each receiver:
        settings service, network system/container/partition services, renderers,
        device arbiter, extension capability service, crash handler, update client
        and updater, storage service, shell (about:policies)
```

New crates (6.24): `hg-policy-catalogue` (the catalogue, its canonical encoding and generated types; L2), `hg-policy` (merge, status, digest, slices; broker), `hg-policy-format` (the format parsers; L2p, policy adapter only), `hg-policy-adapter` (the adapter's L5 logic, run by `hg-os-adapter-main`), `hg-policy-gen` (tool: templates, docs, `.hgidl` cases), and the `policy` backend module in `hg-substrate-os`.

### 6.1 The policy model (EP-D1, EP-D2)

**A policy** is a catalogue entry with these fields:
- a stable **name** in CamelCase, never reused;
- a **type**;
- **bounds**, including a **maximum encoded size** from which the generator sizes its IPC messages;
- **scopes** allowed (machine, user);
- **levels** allowed (mandatory, recommended);
- **direction**: unrestricted, or `restrict-only` (only the protective value can be set);
- the **settings-registry row** it sets, or "non-setting";
- a **merge rule**;
- an **atomic group**, if any;
- **apply** timing (live, or next launch);
- **sensitivity** (public, or internal-hostnames, which are masked in exports and sent only to processes that enforce the policy);
- **`enforced_in`**: the process kinds that enforce it, which generates the slice map (6.6);
- **`remote`**: `allowed`, `machine-opt-in` or `never`, deciding whether the remote document may set it (6.10);
- **`risk_flag`**: the coarse crash-report flag the policy sets when present, if any (6.21);
- **platforms**;
- **stage**;
- **since**, **deprecated** and **removed** versions;
- the **owning spec**;
- **documentation** text in English (localised later through FI-24's string pipeline).

**Types.**
- Bool; integer with a range; enum (string values); bounded string; URL; bounded **path** (absolute, no `..`, no device paths, OS-validated).
- **URL-pattern list.** The grammar is defined by this spec in the catalogue (scheme, host with a leading-`*.` wildcard, optional port, path prefix), with a published mapping into content blocking's filter syntax wherever a list is compiled for CB's matcher. CB has a filter syntax, not a URL-pattern grammar, so one grammar is defined once here.
- String list.
- **Structured** values with a catalogue schema: records and lists of records, such as `ExtensionSettings`. A tree such as `ManagedBookmarks` is **flattened** into records with a parent index and a depth cap of 8†, because the IPC checker refuses recursive types.
- **There is no secret type** and **no float type.** A password, token or private key cannot be expressed (G244). Credentials come from OS stores (NW 6.1) or the consent path. A remote-document key is a **public** key.

**What a policy cannot be:** an arbitrary preference (SH 6.9), a script, or a value that switches on AI (B10) or Izcal-bound telemetry (D-S4, FI-D12), or switches off phishing protection (PH 6.13). Those entries are `restrict-only`: the catalogue accepts only the protective value, and a non-protective value is rejected as `invalid: direction` (EP-12, EP-13). Other protections that enterprises legitimately weaken (4.4) are unrestricted. Each is shown, when weakened, as "weakened by your organisation".

**Versioning.** The catalogue has a monotonically increasing **catalogue version** (u32), bumped by any change. An entry is never renamed. A changed meaning means a new name, with the old one deprecated. A deprecated policy still applies for at least **6 months†** after the release that deprecates it, and shows "deprecated, use X". At distribution's 4-week Stable cadence that is about six releases, and more once the cadence moves to 2 weeks. It is measured in time because admins plan in quarters, not releases. A removed policy shows "unknown (removed in N)". Templates are cumulative, and **the ADMX namespace changes with every catalogue version that changes the ADMX output**. At a 4-week cadence that can be monthly, so policy additions are batched into the template release at most once per Stable release. The namespace must change because Intune refuses to re-upload changed settings into an existing namespace unless the admin deletes and re-imports ([Intune ADMX import](https://learn.microsoft.com/en-us/intune/device-configuration/settings-catalog/import-custom-admx-templates); [OD 3.5][OD]). The generator therefore embeds the catalogue version in the namespace.

**Names and shapes (EP-D2).** Izcal uses its own names. Where Chromium-family browsers have a well-known policy with **exactly** the same semantics, Izcal uses that name and shape so admins' scripts carry over (`ManagedBookmarks`, `ExtensionSettings`'s `installation_mode` values, `DeveloperToolsAvailability`, `AuthServerAllowlist`'s comma-list form). Where the semantics differ, Izcal uses a different name, so a copied value fails visibly instead of meaning something else. The admin docs publish the mapping table. Izcal never claims parity: DV already defines its own device keys for this reason (DV §11, flagged claim 4).

### 6.2 The catalogue (inventory of every policy the corpus defines, plus FI-3's)

"Owner" is the spec whose behaviour the policy controls; that spec owns the semantics and this spec owns delivery. **S** is the stage in which a source can set the policy (Stage 1 builds the plumbing, 6.25). Scopes: M = machine, U = user. Levels: Ma = mandatory, Re = recommended. Unless noted, `remote` is `allowed`.

| Group | Policy | Type | Scope / level | Owner | S | Notes |
|---|---|---|---|---|---|---|
| **Management** | `ManagementDisplayName`, `ManagementContact` | string, URL or mailto | M / Ma | EP | 2 | Shown in the disclosure and on locked rows (SH 6.9 "administrator's contact") |
| | `RemotePolicyDocument` | record {url, keys[1..4], level, allowed_classes} | M / Ma; remote `never` | EP | 2 | 6.10 |
| | `HighImpactChangeNotice` | enum `show` (default) / `quiet-after-first` | M / Ma | EP | 2 | EP-D4 |
| **Settings registry** | one entry for **each** row marked policy-settable, e.g. `HttpsFirstMode`, `GlobalPrivacyControl`, `TrackingProtectionLevel`, `PasswordManagerEnabled`, `DownloadDirectory`, `DownloadPromptForLocation`, `AutoplayDefault`, `SessionRestoreMode` | per row | M, U / Ma, Re | SH 6.9 | 2 | Generated from the registry; CI binds both ways (EP-1) |
| **AI (B10)** | `AiEnabled`, `AccessibilityModelEnabled` | bool, restrict-only (false) | M, U / Ma | H3 B10, ML-D4 | 2 | Force off and lock, never on |
| **Telemetry** | `UsageTelemetryEnabled`, `CrashReportsEnabled` | bool, restrict-only (false) | M, U / Ma | D-S4, FI-D12 | 2 | Policy can only remove Izcal-bound reporting; crash reports are opt-out under D-S4, so the policy turns them off; enforced in the crash handler |
| **Updates** (PO-3.9) | `UpdatesEnabled`, `UpdateChannel`, `TargetVersionPrefix`, `UpdatesSuppressedWindow`, `RelaunchNotificationPeriod` | bool, enum, string, record, int (days) | M / Ma | SH 6.11, FI-2 | 2 | `UpdatesEnabled=false` is honoured only where an admin-run update channel exists (per-machine or package-managed installs; FI-2); enforced in the update client and updater |
| **Default browser** (PO-8.5) | `DefaultBrowserPromptEnabled` | bool | M, U / Ma | SH 6.11, FI-4 | 2 | OS default associations are the OS's (DefaultAssociations XML, [LG 8.12][LG]) |
| **Start and search** | `HomepageUrl`, `StartupPages`, `DefaultSearchProvider` | URL, URL list, record | M, U / Ma, Re | SH | 2 | High-impact (EP-D4) |
| **Bookmarks** (PO-6.10) | `ManagedBookmarks` | flattened record tree (`toplevel_name`, `name`, `url`, parent) | M, U / Ma | SH 6.7 | 2 | 6.13 |
| **Networking** (NW 6.14) | `DnsOverHttpsMode`, `DnsOverHttpsResolver`, `QuicAllowed`, `EncryptedClientHelloAllowed`, `ProxySettings` (atomic group), `AuthServerAllowlist` (NW-D20), `AuthDelegationAllowlist` (remote `never`), `LocalNetworkAccessDefaults`, `TrustAnchors` (DER certificates, public; remote `machine-opt-in`), `ClientCertificateAutoSelect`, `HttpsFirstExceptions`, `WebPushAllowed` | various | M / Ma (proxy and auth: M, U) | NW | 2 | Per-container keys in Stage 2 with SH's container editing |
| **TLS** (PO-7.9) | `TlsMinimumVersion` | enum `1.2` / `1.3` | M / Ma | NW | 2 | EP-D10; no knob below 1.2, no switch to turn post-quantum key agreement off |
| **URL filtering** (PO-6.12) | `UrlBlocklist`, `UrlAllowlist` | URL-pattern lists | M, U / Ma | EP (navigation), CB (requests) | 2 | Enforced as a navigation decision, and optionally as an enterprise CB domain that user rules cannot cancel (6.8); restrictive lists merge by union (6.4) |
| **Content blocking** | `ContentBlockingLists`, `ContentBlockingSiteExceptions` | list set, URL patterns | M / Ma | CB | 2 | Exceptions cannot cancel Izcal's Security entries; the entry is proposed to CB (9.5) |
| **Phishing** | — | — | — | PH 6.13 | — | **Non-entry**: no policy may disable the check; a blocked or stale state shows on `about:policies` (Q-PH-15) |
| **WebCrypto** | — | — | — | WC-D7 | — | **Non-entry** by decision |
| **Devices** (DV 6.10) | `DeviceUsbPolicy`, `DeviceHidPolicy`, `DeviceSerialPolicy`, `DeviceMidiPolicy`, `DeviceBluetoothPolicy`, `DeviceNfcPolicy`, `DevicePreGrantEnabled`, `DevicePreGrants`, `ExperimentalDeviceApisAllowed` | enum, bool, records | M / Ma (family policies also U) | DV | 2 | "Signed policy" means a machine-scope mandatory source (6.22) |
| **JavaScript** | `ScriptJitCeilingRules` (URL pattern → Full, Baseline or Jitless), `AppJitCeiling` | records, enum | M / Ma | JS-D13, AH A19 | 2 | |
| **Extensions** (PO-6.11) | `ExtensionSettings`, `ExtensionOrganisationKeys` (remote `machine-opt-in`), `ExtensionManagedStorage` | records | M / Ma (force-install: M only) | EX | 2 | 6.11 |
| **Native messaging** | `NativeMessagingHostsAllowlist`, `NativeMessagingUserLevelHosts` | ID list, bool | M / Ma | EX 6.11 | 2 | |
| **Apps** (PO-12.5) | `AppInstallForceList` (`.izapp`), `WebAppInstallForceList` | records | M / Ma | AH 6.7, FI-9 | 2 | |
| **DevTools and automation** (PO-13.11) | `DeveloperToolsAvailability`, `DeveloperToolsBlockedUrls`, `RemoteAutomationAvailability`, `TabRecordingAvailability` | enums, URL patterns | M, U / Ma | DT | 2 | 6.14 |
| **OS scripting** (OI-D18) | `OsScriptingAvailability` (`user-choice` default / `allowlist-only` / `blocked`); `OsScriptingAllowedCallers` (caller identities) | enum, restrict-only (no value enables scripting); record list | M, U / Ma | OI | 2 | 6.14, EP-D20; `risk_flag`: **os-scripting-restricted** |
| **Kiosk** (PO-1.17, BF-20.2) | `KioskMode` | record (6.15) | M / Ma | EP, SH | 2 | 6.15 |
| **Legacy switching** (PO-6.17) | `SiteSwitchingRules`, `SiteSwitchingSiteListUrl`, `SiteSwitchingBrowser` | records, URL (`https`, `file` or UNC), record {path, args[]} | M / Ma | EP | 2 | 6.16 |
| **Reporting** (PO-6.14) | `EnterpriseReporting` | record {endpoint, interval, fields[]} | M / Ma | EP | 2 | 6.17, EP-D14 |
| **DLP** (PO-6.15) | reserved: `ContentAnalysisAgent` | — | — | EP | 3 | 6.18, EP-D15 |
| **Single sign-on** (PO-7.3) | `OsSingleSignOn` | enum `off` / `on` | M / Ma | NW, EP | 2 | 6.19 |
| **Profiles and VDI** (PO-1.16, PO-6.16) | `ProfileDirectory`, `EphemeralProfile`, `ProfileCreationAllowed` | path, bool, bool | M, U / Ma | SH, FI-10 | 2 | 6.20 |

### 6.3 Sources per OS, and the admin-only rule (EP-D3)

| OS | Mandatory sources | Recommended sources | Admin-only check |
|---|---|---|---|
| **Windows** | `HKLM\Software\Policies\Izcal\Browser` (machine), `HKCU\Software\Policies\Izcal\Browser` (user) | the same keys' `Recommended` subkey | The opened key's security descriptor must grant write only to Administrators, SYSTEM or TrustedInstaller. A key a standard user can write is rejected as `blocked: source writable by user`. Registry links are refused. Only an archived Microsoft page states that the policy trees "cannot be modified by a user who is not an administrator" ([HP 1.3][HP]); no current page documents the ACL, so the default is measured (EP-M1, [OD 1.11][OD]) |
| **macOS** | Forced values (`CFPreferencesAppValueIsForced` = true) in the domain `dev.izcal.browser`, split into **device channel** and **user channel** | Managed, non-forced values on the same channels | Only managed values count. A plain `defaults write` value is ignored (Chrome treats it as recommended, [OD 4.6][OD]; Izcal has no arbitrary preferences). How the adapter tells the device channel from the user channel is measured at EP-M1 (Q-EP-3). **Until then every macOS forced value is treated as user scope**, so machine-only policies such as force-install fail safe on macOS |
| **Linux** | `/etc/izcal/policies/managed/*.json` | `/etc/izcal/policies/recommended/*.json` | Each file and every parent directory must be owned by root and not group- or world-writable. The check runs on the **opened object**: walk from `/` with directory descriptors, open with `O_NOFOLLOW`, refuse symlinks, and check owner and mode with `fstat`. Anything else is rejected and reported. Chrome only advises this ([OD 5.3][OD]) |
| **Linux, Flatpak** (Stage 2 packaging, S1 13.2) | `/var/lib/flatpak/extension/dev.izcal.browser.Policy/<arch>/<branch>/policies/{managed,recommended}` | same | The user-writable `$XDG_DATA_HOME` extension path is **never** a source ([OD 5.6][OD]) |
| **All** | **Never**: the install directory of a per-user install, `$XDG_DATA_HOME`, `%LOCALAPPDATA%`, the profile, command-line flags | — | A file in the install directory is honoured only for a per-machine install whose directory is admin-only, and only as **recommended** (FI-2) |

Registry encoding follows the conventions administrators' tools already emit ([OD 1.6, 1.8][OD]):
- a bool is a DWORD 0 or 1;
- an int is a DWORD, or a QWORD where the range needs it;
- a string is REG_SZ;
- a list is a subkey with values named `1`, `2` and so on, numbered from 1;
- a structured value is REG_SZ holding JSON. A JSON value is parsed in the adapter and never in the broker.

The ADMX generator emits these forms.

**What the check buys, in SEC's terms.** The documented abuse of incumbents' policy channels came from installers with administrator rights writing HKLM ([HP 3.5][HP]), which is out of scope (SEC 6.1). Same-user malware (ATK-7) can write anywhere the user can, so a user-writable policy location would give it the same durable channel without elevation. The admin-only check closes that **durable, injection-free route**: a written value that survives reboots and looks legitimate. ATK-7 keeps what SEC 6.1 already grants it: injecting into the broker or the adapter per run, relaunching with flags, and (under SEC-D14's per-user installs) replacing user-writable binaries. SEC's other mitigations make those costlier and more visible. Per-machine installs (EP-D17) remove the binary-replacement route.

**No management-relationship gate (EP-D4).** Chromium-family browsers also accept some policies only on domain-joined or MDM-enrolled machines ([OD 1.2, 1.4][OD]; [HP 3.4][HP]). Izcal does not adopt that gate:
- it has no Linux equivalent;
- it defends against an administrator-level attacker, who is out of scope;
- it fails silently when the device state is misread, as Edge's Pro Education bug showed ([LG 1.3][LG], Medium).

Izcal instead makes **high-impact policies visible when they change**. Force-installed extensions or apps, proxy, trust anchors, organisation keys, homepage or search, device pre-grants and reporting each show a one-time, non-modal notice the next time a window opens: "Your organisation changed: <list>. See what your organisation controls." The change is also written to the prompt ledger **in the storage service**, so a later session still shows it even if the broker of the session that applied it was tampered with.

### 6.4 Precedence and merging (EP-D5)

**`EffectivePolicy` covers the policy layers only.** The person's own setting is resolved downstream, in the settings service (6.8). **One total order** for the policy layers, highest first:
1. **machine mandatory** (OS);
2. **remote document mandatory** (6.10);
3. **user mandatory** (OS);
4. **machine recommended** (OS);
5. **remote document recommended**;
6. **user recommended** (OS);

and then, for setting-backed policies, mandatory > the person's setting > recommended > Izcal's default (6.8).

**Why the remote document ranks above OS user policy.** Its URL and keys can come only from OS machine policy, so it speaks with the machine owner's authority, delegated. It still ranks below OS machine policy, which can therefore always correct it. Its reach is limited per policy by the catalogue's `remote` field (6.10).

**Conflicts within a level.** On Linux, files apply in byte-wise lexical order of their names. A policy set in two files at one level is a **conflict**: the lexically first file wins and the conflict is reported. On Windows and macOS, one level has one source, so no conflict can occur there.

**Merge rules are fixed in the catalogue, not configurable by admins:**
- **highest wins** is the default;
- **union** applies only to restrictive lists, across mandatory levels (`UrlBlocklist`, a block-list entry of `ExtensionSettings`, `DeviceXPolicy` blocks), because a union can only restrict further;
- an allow-list never unions, because a union of allow-lists widens access;
- an **atomic group** (`ProxySettings`; `RemotePolicyDocument`; a `KioskMode` record) is taken whole from the highest source that sets any member.

Chrome makes merging an admin opt-in ([Chrome policy precedence](https://support.google.com/chrome/a/answer/9037717); [OD 6.3][OD]). Izcal does not, because a merge that can widen access should not be a switch.

**The merge is a pure function.** `merge(catalogue, [SourceSnapshot]) -> (EffectivePolicy, [PolicyStatus])` is total, deterministic, order-independent across sources (each snapshot carries its level), idempotent, and **monotone for restrict-only and union entries**: adding a source can never relax a restriction. Property tests check all of this (EP-4). A source change replaces that source's whole snapshot and re-merges everything, as Windows 2000 did ([HP 1.3][HP]). Effective policy is per OS user: one broker per OS user serves all of that user's profiles (FI-D17).

**Non-tattooing.** Effective policy is held only in the broker's memory and in the recordings of the processes that received it. It is never written into the settings store. When a policy disappears, the setting reads the person's own value again, or the default (EP-5). Managed bookmarks live in a policy-provenance folder that is rebuilt from policy, never copied into the person's bookmarks (6.13).

### 6.5 Validation and failure (EP-D6)

- **Per policy, never all or nothing.** An invalid value rejects that policy only (`invalid: <reason>`). An invalid list entry is dropped individually and counted (`3 of 40 entries invalid`). An unknown name is shown and ignored. A deprecated name applies and is flagged. These follow Firefox's granularity ([OD 6.9][OD]).
- **Wrong scope or level** (for example, force-install from a user source) is `blocked: not allowed at user scope`.
- **Direction violation** (for example, `AiEnabled=true`) is `invalid: restrict-only`.
- **A whole OS source that fails** (unreadable, unparsable, rejected ownership) is shown as a source error. Its **last snapshot applied in this run** is kept and marked **stale**, so a half-written file cannot drop every restriction in the middle of a session. At launch with no earlier snapshot, the source contributes nothing and the error is shown. An OS-source snapshot is never persisted: a persisted cache would sit in a user-writable location (ATK-7) and would need its own integrity key. The OS source itself is the durable copy.
- **The remote document is different, because it is signed.** Its persistence is in 6.10.
- **Every status has a reason code** from a closed enum, shown on `about:policies` and exported in policy-health reporting (6.17). That makes "why didn't my policy apply" answerable (EP-9).

### 6.6 Processes and IPC (EP-D7)

**On demand, not always on.** FP 8.2 budgets the OS-adapter process kind as on demand (≤ 2 MiB† idle, exiting after 10 s†), and FP-1 has 1 MiB† left. The policy adapter follows that row.
- **The broker watches.** At launch the broker calls `hg-substrate-os::policy`'s **watch API**: does a source exist, and register for its changes (`RegisterGPNotification` and `RegNotifyChangeKeyValue` on Windows; inotify on the `/etc` directories and their parents on Linux; app activation on macOS, 6.7). That API returns events, never values, so the broker parses nothing.
- **The broker launches the adapter** when a source exists at launch, when a watched source changes, on "Reload policies", and when organisation bytes arrive (6.10, 6.16).
- **The adapter** runs as `hg-os-adapter-main` with a typed **Policy configuration** in its launch message (one `ProcessKind::OsAdapter`, configured, rather than a new kind). It:
  - reads the sources through `hg-substrate-os::policy`'s **read API** (registry keys, the managed-preference domain through CFPreferences, the `/etc` files), with the admin-only check on each opened object (6.3);
  - parses with `hg-policy-format` (JSON, registry-embedded JSON, and the site-list XML through `hg-xml-syntax`, one parser per format);
  - verifies the remote document's signature;
  - validates every value against `hg-policy-catalogue`;
  - sends the broker one snapshot per source;
  - exits 10 s† after its last request.
- **Count.** There is one adapter per OS-user broker, because brokers are per OS user (FI-D17). Each reads the machine sources plus its own user's sources.
- **Machines with no source run no adapter.** That covers unmanaged machines and every Stage 1 release build (EP-D19), so EP-8's cost is zero there.
- **The adapter holds no profile data, no network access and no page input.** Its sandbox grants read access to the policy locations alone. Whether each OS's sandbox can grant exactly that is measured at EP-M1. If one cannot, the broker opens the key or directory handle through the read API and passes it as a handle, still with no parsing in the broker. That needs registry and directory handle kinds in the IPC schema.
- **Only two callers may use `hg-substrate-os::policy`** (G248): `hg-policy` may use its watch API and `hg-policy-adapter` its read API, enforced by the dependency allowlist as devices' G210 does for device effects.

**Why not the broker.** The broker must parse no untrusted bytes (IPC-R15). A JSON file that same-user malware cannot write is still a complex format; the parser surface belongs in a sandbox, as it does for content-blocking list text (CB 6.10) and for trust anchors, which go to `hg-pki` as opaque DER (NW 6.1).

**Why not the per-container credential adapters.** NW 6.1's OS adapters handle SSPI, GSSAPI and client-key signing per container. Policy reads in the same process would widen a credential process's sandbox to the policy locations.

**The broker side** is `hg-policy`, a pure crate:
- it receives snapshots on a **bound** endpoint and accepts them only from the adapter it launched (`CallerIdentity`, IPC-R2);
- it re-checks restrict-only entries and scope rules in the merge (6.4), and computes the digest (6.21);
- it derives **slices** from the catalogue's `enforced_in` map and sends each on the receiver's existing endpoint. A process receives only the fields it enforces, so an `internal-hostnames` field never reaches a process that does not enforce it (EP-20). The map at this draft:

| Slice | Receiver | Carries |
|---|---|---|
| `SettingsPolicy` | Settings service in the broker | Locks and recommended defaults for registry rows |
| `NetworkSystemPolicy` | Network system service | Proxy for Izcal's own traffic, trust anchors, DoH |
| `NetworkContainerPolicy` | Each container service | Proxy and auth allowlists, content-blocking list set, LNA defaults, Web Push |
| `PartitionNetworkPolicy` | Each partition service (per container) | TLS floor, QUIC, ECH, HTTPS-First exceptions, the compiled `UrlBlocklist` request rules |
| `NavigationPolicy` | `hg-navigation` (broker) | `UrlBlocklist`/`UrlAllowlist` navigation decisions, site switching |
| `RendererPolicy` | Each renderer, at launch and on change | Its JIT ceiling only |
| `DevicePolicy` | Device arbiter | Family policies, pre-grants |
| `ExtensionPolicy` | The broker's extension capability service (`hg-ext-capability`) | Install modes, capability caps, host lists, organisation keys. Each component host receives only its own extension's `storage.managed` value (EX G171: the component host holds no grant truth) |
| `CrashPolicy` | Crash handler | `CrashReportsEnabled`, and the **coarse policy flags** for the shared header (6.21) |
| `UpdatePolicy` | Update client and updater | Update policies (6.20) |
| `StoragePolicy` | Storage service | `ProfileDirectory`, `EphemeralProfile` |
| `OsScriptingPolicy` | The broker's OS-scripting gate (`hg-os-broker`, OI 6.14) | `OsScriptingAvailability`, `OsScriptingAllowedCallers` |
| `ShellPolicy` | Shell renderer | Locks, disclosure, the `about:policies` model |

Every slice carries the policy **epoch** (a u64 counter in the broker) and the digest. The AI grants reach the ML service and the agent host through their settings rows, as B10 already routes them.

**`policy.hgidl` (reserved shape for M1's exit).** The text below passes M1's merged `hg-ipc-codegen` checker; it was run as a library on 24 October 2026. The two `Policy*` unions are emitted by `hg-policy-gen`, one case per catalogue entry. The stand-in cases show the largest shapes the generator must size. Because the checker sizes a message by its maximum encoding, each list case is sized by the generator from the catalogue's encoded bounds so that one `list_part` fits `hg_substrate::ipc::MAX_MESSAGE_BYTES`, 64 KiB† (S1 4.1). A list larger than one message crosses as several parts of the same policy, in order.

```
/// Policy adapter to broker (enterprise policy spec 6.6, EP-D7). Reserved
/// at M1's exit; the `Policy*` unions are emitted by hg-policy-gen from the
/// catalogue, and the stand-in cases here show the largest shapes.
protocol policy;

/// Why the adapter could not serve a request. Closed (CS-ER-6).
error PolicyError {
    /// The source does not exist on this machine.
    NoSource,
    /// The adapter is shutting down.
    ShuttingDown,
}

/// Which source a snapshot came from. Closed; the order is not precedence.
enum PolicySource {
    /// HKLM policy key, mandatory.
    WindowsMachine,
    /// HKLM policy key, Recommended subkey.
    WindowsMachineRecommended,
    /// HKCU policy key, mandatory.
    WindowsUser,
    /// HKCU policy key, Recommended subkey.
    WindowsUserRecommended,
    /// macOS device-channel forced values.
    MacDeviceForced,
    /// macOS device-channel managed defaults.
    MacDeviceDefault,
    /// macOS user-channel forced values.
    MacUserForced,
    /// macOS user-channel managed defaults.
    MacUserDefault,
    /// Linux managed directory.
    LinuxManaged,
    /// Linux recommended directory.
    LinuxRecommended,
    /// Flatpak host extension, managed.
    FlatpakManaged,
    /// Flatpak host extension, recommended.
    FlatpakRecommended,
    /// Signed remote document, mandatory part.
    RemoteMandatory,
    /// Signed remote document, recommended part.
    RemoteRecommended,
}

/// Why a whole source did not apply. Closed.
enum SourceStatus {
    /// Read and parsed.
    Read,
    /// Writable by a non-administrator: refused (EP-D3).
    WritableByUser,
    /// Could not be read.
    Unreadable,
    /// Could not be parsed.
    Unparsable,
    /// The remote signature did not verify.
    SignatureInvalid,
    /// The remote document is past its expiry.
    Expired,
    /// The remote document is older than one already accepted.
    RolledBack,
}

/// Why the broker asks for a re-read. Closed.
enum RereadReason {
    /// The person pressed Reload policies.
    UserReload,
    /// An OS change notification fired.
    OsChange,
    /// A remote document or site list arrived.
    RemoteArrived,
    /// The broker started.
    Startup,
}

/// Stand-in for a generated scalar or small-record case.
struct UrlEntry {
    /// One URL or URL pattern, in the catalogue's bound.
    url: string<2048>,
}

/// Stand-in for a flattened tree node (ManagedBookmarks).
struct BookmarkNode {
    /// Index of the parent node; 0 is the root folder.
    parent: u32,
    /// The display name.
    name: string<256>,
    /// The URL, absent for a folder.
    url: option<string<2048>>,
}

/// One policy's small value (generated; at most 4 KiB per case).
union PolicyScalar {
    /// Stand-in: a boolean policy.
    AiEnabled {
        /// The value.
        value: bool,
    },
    /// Stand-in: a URL policy.
    HomepageUrl {
        /// The value.
        value: UrlEntry,
    },
}

/// A part of one list-valued policy (generated; each case sized to fit).
union PolicyListPart {
    /// Stand-in: part of a URL-pattern list.
    UrlBlocklist {
        /// Up to 30 entries.
        items: vec<UrlEntry, 30>,
    },
    /// Stand-in: part of a flattened bookmark tree.
    ManagedBookmarks {
        /// Up to 20 nodes.
        items: vec<BookmarkNode, 20>,
    },
}

/// Adapter to broker. Bound: served only to the adapter the broker launched.
interface PolicyHost bound capacity 64 overflow terminate {
    /// Starts one source's snapshot.
    begin(
        /// The source.
        source: PolicySource,
        /// The catalogue version the adapter validated against.
        catalogue: u32,
        /// The snapshot's sequence number.
        seq: u64,
    );
    /// One small value.
    scalar(
        /// The source.
        source: PolicySource,
        /// The snapshot.
        seq: u64,
        /// The value.
        value: PolicyScalar,
    );
    /// Part of one list value; parts of one policy arrive in order.
    list_part(
        /// The source.
        source: PolicySource,
        /// The snapshot.
        seq: u64,
        /// The part.
        part: PolicyListPart,
    );
    /// Ends the snapshot; the broker applies it atomically only here.
    end(
        /// The source.
        source: PolicySource,
        /// The snapshot.
        seq: u64,
        /// How many messages the snapshot held, for a completeness check.
        count: u32,
    );
    /// The source's state, sent with every snapshot and on failure.
    status(
        /// The source.
        source: PolicySource,
        /// Its state.
        state: SourceStatus,
    );
}

/// Broker to adapter.
interface PolicyControl bound capacity 16 overflow terminate {
    /// Re-read every source.
    reread(
        /// Why.
        reason: RereadReason,
    );
    /// Part of a fetched remote document (IPC-R16 entry; at most 1 MiB in all).
    remote_part(
        /// Byte offset of this part.
        offset: u32,
        /// The bytes.
        chunk: bytes<60000>,
        /// Whether this is the last part.
        last: bool,
    );
    /// The remote document's detached Ed25519 signature.
    remote_signature(
        /// Exactly 64 bytes; the adapter refuses another length.
        sig: bytes<64>,
    );
    /// Part of a fetched or file-read site list (IPC-R16 entry; at most 4 MiB).
    site_list_part(
        /// Byte offset of this part.
        offset: u32,
        /// The bytes.
        chunk: bytes<60000>,
        /// Whether this is the last part.
        last: bool,
    );
}
```

The snapshot applies **atomically** on `end`. A partial snapshot, or one whose `count` disagrees, is discarded, as in Apple's activations ([HP 5.4][HP]). Every value is a generated, typed `.hgidl` case, so the broker decodes with generated decoders and parses no self-describing payload (IPC-R3). The remote document and site-list bytes travel broker → adapter as opaque bytes. They are the only payloads outside `.hgidl` here, each registered as an IPC-R16 entry (A23), with the adapter as receiver and `hg-policy-format`'s fuzz targets. Shared memory is not needed: M1 has no shared-memory handle kind (`HandleKind` holds only channel ends), and chunking fits the existing rules. M1-exit work checks this file in as a golden `.hgidl` in the CS-ST-17 sample pattern, so a generator change that breaks it fails CI.

### 6.7 Refresh and change notification

| OS | Trigger (in the broker's watch API) | Fallback |
|---|---|---|
| Windows | `RegisterGPNotification` for machine and user policy, plus a re-armed, thread-agnostic `RegNotifyChangeKeyValue` on the two policy keys, because MDM and script writes may not raise the GP event ([OD 1.13–1.14][OD]) | None needed if both fire; measured at EP-M1 |
| macOS | No documented notification for forced-value changes was found ([OD 4.10][OD]) | Launch the adapter at app activation if the last read is older than 15 min†, and on the existing network wake grid (no new timer); EP-M1 measures whether `NSUserDefaultsDidChangeNotification` arrives |
| Linux | inotify on the policy directories and their parents | None |
| Remote document | The organisation container's fetch schedule (6.10) | — |

Changes are **debounced** for 2 s† to coalesce a GPO refresh that writes many values. A policy marked `apply: next launch` shows `pending restart`. Everything else applies live: the broker sends new slices, and each consumer applies them at its next task boundary. **Policy adds no periodic timer to Izcal's code** (FP-18; EP-8). The watch API sleeps in OS waits, and the adapter runs only when there is work.

### 6.8 How values reach behaviour

- **Settings rows.** The effective value is: mandatory policy if set (the row is locked); otherwise the person's value if set; otherwise recommended policy if set (shown as "Default from your organisation", changeable); otherwise Izcal's default. The person's own value is kept while a mandatory policy is in force and returns when the policy goes (non-tattooing).
- **Lock display.** One label family, owned by SH 6.9, is used everywhere, devices and extensions included:
  - "**Set by your organisation**" for a mandatory value;
  - "**Default from your organisation**" for a recommended value;
  - "**Installed by your organisation**" for force-installed code;
  - "**Allowed by your organisation**" for a policy grant the person can still revoke (DV 6.18's device pre-grants, policy-granted extension hosts). Revocability is unchanged.

  Each label links to the policy's row on `about:policies`.
- **URL blocking.** `UrlBlocklist` and `UrlAllowlist` are a **navigation decision** in `hg-navigation`, made before a renderer commits. The decision is recorded and explained on the error page with the policy's name. For subresources, the compiled rules go to the partition service's CB matcher as an **enterprise trust domain that the person's own rules and CB 6.12's per-site controls cannot cancel**; CB is amended to allow that (A18), because CB's user rules otherwise cancel blocks in every domain.
- **Network, devices, JS, extensions, apps.** Each owner applies its slice by its own rules. Per-container policy (NW's "per container") becomes a catalogue scope when SH's Stage 2 container editing ships. Until then, each network policy applies to every container.
- **Profiles (FI-D17).** OS policy is per OS user and applies to every profile under that user, because all of that user's profiles share one broker. Profile-bound (account-bound) policy waits for FI-35's accounts and is not designed here.

### 6.9 `about:policies` and the management disclosure

**C1 is resolved: the page is `about:policies`.** The shell owns about pages (SH 6.14), and NW 6.14 adopts the name (amendment 9.5). The page is a shell document, local, unlisted, never linked from content, and fully keyboard and screen-reader accessible under SH 6.16.

**Contents:**
- **Management summary:** "This browser is managed by <ManagementDisplayName>", the contact, the number of policies in force, the sources present, and **what your organisation can see**. With reporting off, that reads "nothing is reported". With reporting on, it lists each field, the destination and the interval, generated from `EnterpriseReporting`'s schema (6.17).
- **One row per policy that any source sets, or that has a status:**
  - name, effective value (internal hostnames masked unless "Show values" is pressed) and level;
  - source, as a plain-language name with its technical location, for example `HKLM\Software\Policies\Izcal\Browser`;
  - **status with reason**, for example `Overridden by machine mandatory`, `Blocked: source writable by user`, `Invalid: 3 of 40 entries` (expandable), `Deprecated: use X`, `Pending restart`;
  - **values that lost**, with their sources;
  - the setting the policy locks, linked to the settings row;
  - the policy's reference page in the offline docs.
- **Source health:** each source's last read time, its sequence number and any error, including "stale since <time>".
- **Actions:** "Reload policies" (`reread`); **export as JSON**, with masked values unless the person chooses otherwise, for an admin's ticket; and copying the **digest**, so support can match it to a recording.
- **Weakened protections:** each unrestricted policy that weakens a protection (4.4) is listed as "weakened by your organisation", and the site panel shows the same where it applies.

"What changed" (EP-D4) appears as the notice and as ledger entries, which `about:policies` lists for 90 days†.

Nothing on the page is fetched from the network. Its data is the `ShellPolicy` slice.

### 6.10 Cloud management: the signed remote policy document (EP-D8)

No open standard exists ([LG 2.7][LG]), so FI-D14's rule leaves two options: deliver through the OS channels any MDM already speaks, or publish an Izcal protocol in prose. Izcal does both and operates **no console**.

- **Enrolment** is a machine-mandatory OS policy, `RemotePolicyDocument = {url, keys, level, allowed_classes}`. The URL is `https`. `keys` holds one to four Ed25519 public keys, so keys can rotate with overlap. `level` is mandatory or recommended. `allowed_classes` opts in, by class, the policies the catalogue marks `machine-opt-in`. Because the trust root arrives through the OS channel, no token, account or Izcal server is involved.
- **Reach.** The catalogue's `remote` field limits what the document may set:
  - `never`: `RemotePolicyDocument` itself (the document cannot re-point its own URL or keys), `AuthDelegationAllowlist`, and the proxy used to fetch the document;
  - `machine-opt-in`: `TrustAnchors` (TLS interception), `ExtensionOrganisationKeys` (unreviewed code), `OsSingleSignOn`. These apply only if OS machine policy names their class in `allowed_classes`;
  - `allowed`: everything else.

  The document's signing key typically lives in a web-publishing workflow rather than the domain's administrative tier, so the most powerful classes stay with the OS channel unless the admin chooses otherwise.
- **The document** is UTF-8 JSON in the Linux file format, wrapped in an envelope `{version, issued, expires, organisation, policies}` and signed detached (`<url>.sig`, Ed25519 over the exact bytes). `version` is a u64 that must increase. `expires` is at most 30 days† after `issued`.
- **Who fetches: the organisation container.** NW-D17 reserves the network system service for Izcal's own traffic, on a published endpoint list checked by NW-36. Organisation traffic is a different class: the remote document, the site list (6.16) and reporting uploads (6.17). It runs in a **built-in, non-user-visible organisation container** that exists only when machine policy names an organisation endpoint. The container has its own container and partition services (NW-D3) and uses the OS proxy or OS-policy `ProxySettings`, never a proxy the remote document sets. It holds the policy trust anchors and uses NW-D20's allowlist, so intranet hosts and Negotiate work, and it never shares state with the person's containers. An unmanaged machine makes **0** extra requests (NW-36 unchanged for it).
- **Schedule.** The fetch runs at start, then every 6 h† on the existing wake grid, and on `reread`. The bytes reach the adapter through the broker as opaque bytes. That flow is registered as an **IPC-R16 entry** (A23): the document is capped at 1 MiB†, the signature at 64 bytes, the receiver is the adapter's `hg-policy-format`, and the entry has its own fuzz target and SEC-6 approval.
- **Verification** happens in the adapter: signature, monotonic version (rollback refused: `document rolled back`), and expiry.
- **Persistence (the document is signed, so the signature is its integrity key).** The adapter asks the broker to store **the last accepted document bytes, its signature and the highest accepted `(version, issued)`** per enrolment URL and key set, in the OS user's policy cache (per OS user, not per profile: FI-D17). At launch the adapter re-verifies the cached document and applies it as `stale` until the first fetch succeeds. A blocked fetch at start therefore leaves the last policy in force, and an older valid document served after a restart is refused as `rolled back`. An expired document stays in force as stale for 7 days† after `expires` and is then dropped; a freeze longer than that shows on the page and in policy-health reporting. **Residual:** ATK-7 can delete the cache or reset the counter. That returns "unmanaged by the remote source until the next fetch", never a forged policy, and the deletion is visible on `about:policies` as a missing cache.
- **Protocol publication.** Izcal publishes the document format and fetch rules as a prose specification. Any organisation can host the document on a static web server, and any MDM or console vendor can generate one. A self-hostable reference console is a Stage 3 option (reversal trigger in EP-D8).

### 6.11 Extensions, native messaging and apps (EP-D9)

**`ExtensionSettings`** takes a per-ID record, plus a `*` default that may carry only `installation_mode` (allowed or blocked) and `allowed_types`:
- `installation_mode`: allowed, blocked, force_installed, normal_installed or removed;
- `blocked_capabilities`: EX 6.5's capability classes, not Chrome's permission strings;
- `runtime_blocked_hosts` and `runtime_allowed_hosts`, as URL patterns;
- `minimum_version`;
- `update_source`;
- `toolbar_pin`.

The shape follows the documented incumbents where the semantics match ([LG 8.8–8.9][LG]).

**Force-install is honoured only from machine-scope mandatory sources** (OS machine policy, or the remote document, whose keys come from machine policy). A user-scope force-install is `blocked: not allowed at user scope`.

**Policy-granted host capabilities appear in the same UI as user grants**, labelled with their source (EX 6.5).

**The organisation trust tier.** This resolves the conflict between EX 6.12 and AH 6.7.
- `ExtensionOrganisationKeys` (machine mandatory) names one to eight public signing keys.
- A package signed by one of those keys installs from the organisation's `update_source` **without Izcal's platform signature**. It is labelled "Installed by your organisation" and kept in a trust tier that never mixes with the directory's (EX 6.12's G84 discipline).
- A directory package that is force-installed still needs Izcal's platform signature, as EX 6.12 says.

An organisation can therefore deploy in-house extensions without Izcal reviewing them, and the person can see that it did. Writing an organisation key durably needs administrator rights (6.3). The remote document can add one only if OS machine policy opts that class in (6.10). Q-EX-5's "force-install surface" is answered here; managed-device attestation stays open.

**`ExtensionManagedStorage`** maps an extension ID to a structured value.
- The adapter checks only the catalogue's generic structure and bounds. It holds no profile or extension data.
- Validation against the extension's declared `storage.managed_schema` runs in EX's **package utility** (`hg-ext-package-main`, strictest tier), which already parses the package (EX 6.12). The broker's extension capability service hands it the generic value and the installed package's schema, per profile.
- The schema evaluator is a bounded subset (types, required keys, enums, lengths) with its own fuzz target, listed with EX's parsers.
- The validated value goes to that extension's component host only, served read-only as `storage.managed` (EX 6.6). This replaces EX's "configuration reader".

**Native messaging.** `NativeMessagingHostsAllowlist` limits which host names may be connected, and EX 6.11's connect-time grant still applies. `NativeMessagingUserLevelHosts=false` ignores host manifests in user-writable locations.

**Apps.** `AppInstallForceList` (`.izapp` packages, signed by the platform or an organisation key, with version pins, AH 6.7) and `WebAppInstallForceList` (FI-9's installed web apps, records shaped like [LG 8.11][LG]) are force-installs, so the same machine-scope rule applies. Force-installed web apps run in the profile's ordinary (site, container) keys (FI-D17).

### 6.12 Networking, TLS and FIPS (EP-D10)

NW 6.14's list becomes catalogue entries (6.2). Points this spec settles:
- **Trust anchors.** `TrustAnchors` carries public DER certificates. The adapter validates the encoding only, and NW passes the certificates to `hg-pki` as opaque DER. Anchors that an enterprise adds to the OS store remain honoured as NW 6.5 says, so the policy is for organisations that do not manage the OS store.
- **Ambient authentication.** `AuthServerAllowlist` is NW-D20's allowlist. The default stays empty, and there is no intranet heuristic, unlike Chromium's fallback when the policy is unset ([LG 5.7][LG]).
- **TLS minimum version.** `TlsMinimumVersion` is `1.2` (the default) or `1.3`. There is no value below 1.2, matching Edge's removal of lower floors ([LG 8.3][LG]).
- **Post-quantum key agreement cannot be disabled by policy.** Edge removed that switch in Edge 147 ([Edge policy page](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-browser-policies/postquantumkeyagreementenabled); [LG 8.4][LG]). Firefox still has one, described as "a temporary workaround" for broken middleboxes ([LG 8.4][LG]), which shows how such switches outlive their reason. The reversal trigger is a measured middlebox breakage on the managed-environment corpus (NW 10).
- **FIPS (EP-D10): refused.** FIPS 140 validates a cryptographic **module**, and Windows' FIPS mode governs only its own primitives libraries ([LG 8.6][LG]). Izcal's `hg-crypto` is its own implementation, so a "FIPS mode" switch would be a label without a validation behind it. Izcal does not ship one. The reversal trigger is a public-sector buyer that requires a FIPS 140-3 validated module: that means commissioning validation of `hg-crypto`'s TLS subset, a cost and a decision, never a switch to another vendor's module.

### 6.13 Managed bookmarks, homepage, startup and search

`ManagedBookmarks` builds a read-only folder (the default name is the organisation's display name) in the bookmark store. SH 6.7's provenance vocabulary is closed and has no policy value, so this spec proposes to the shell (which owns the schema, Q-SG-6) a **14th provenance value `policy`** and a folder attribute **`managed_by_policy`**, with SH-19's schema test updated (A19). It is rebuilt from the effective policy whenever the policy changes, never merged into the person's bookmarks, and deleted when the policy goes. The search index treats its entries as bookmarks.

`HomepageUrl`, `StartupPages` and `DefaultSearchProvider` are setting-backed and high-impact (EP-D4). A recommended value is a starting default, and a mandatory value locks the row.

### 6.14 DevTools, automation and recordings (EP-D11)

**`DeveloperToolsAvailability`:**
- `allowed` is the default;
- `blocked-for-managed` blocks DevTools on force-installed extensions and apps and on pages matching `DeveloperToolsBlockedUrls`;
- `blocked` blocks DevTools, View Source and the pane renderers entirely.

The policy governs the DevTools frontend and its panes. URL lists take precedence over the value, as Edge documents ([LG 8.7][LG]).

**`RemoteAutomationAvailability`** has two values. `user-choice` (the default) leaves the DevTools automation gateway (CDP, BiDi, MCP, DAP sessions) as the person enables it today. `blocked` makes the gateway refuse to start, and the launch flags that would enable it do nothing. This is separate from DevTools, because infostealers abuse remote debugging ports (SEC ATK-7 examples), so an organisation may block automation and still allow DevTools.

**`TabRecordingAvailability`:**
- `allowed` is the default;
- `local-only` allows "Reload to record" but disables export;
- `blocked` disables the affordance.

A recording holds page content (S1 3.4), so an organisation can stop it leaving the machine.

Agents are governed by `AiEnabled` (B10).

**`OsScriptingAvailability` (EP-D20)** governs the full OS scripting surface the owner decided for OI-D18 (AppleScript on macOS, COM automation on Windows, a D-Bus interface on Linux). That surface exists only when the person turns on a debug mode and grants consent to each calling app. The policy's values:
- `user-choice` (the default) leaves OI-D18's model as the person uses it.
- `allowlist-only` lets the person grant consent only to callers in `OsScriptingAllowedCallers`. Each caller is named by its OS identity: a code-signing team and bundle ID on macOS, a publisher and path on Windows, an executable path and owner on Linux. Consent is still per app and still the person's. The list narrows who may be asked; it grants nothing.
- `blocked` disables the debug-mode switch and the scripting surface. Existing consents are revoked and shown as "Blocked by your organisation".

**No value turns scripting on.** Policy cannot enable debug mode, pre-grant a caller's consent or expose the surface without the person's actions. The entry is `restrict-only` (G245). The reason is the one AGT-R8 records: infostealers relaunch browsers to drive them. A forced-on scripting surface would hand same-user malware a page-reading channel under the organisation's name, and an OS consent prompt checks nothing after the first grant (OI 6.14). The standard open and activate events (Apple Events `GURL` and `odoc`, launches through the launcher) are not scripting and stay under OI's inbound allowlist.

**Why a fourth policy.** EP-D11's reasoning applies. OS scripting is a different principal: local apps over OS IPC, not a DevTools client and not a remote-debugging socket. It meets a different attacker (ATK-7 driving the browser from another process). An organisation may want DevTools and BiDi automation while blocking OS scripting, or the reverse. One switch would force a choice it does not need.

### 6.15 Kiosk mode (EP-D12)

**Kiosk is a capability-reduced profile, not hidden UI.** `KioskMode` is an atomic record with these fields:
- `start_urls`;
- `navigation_allowlist`, URL patterns; everything else is blocked;
- `layout`: `fullscreen-single` or `windowed-tabs`;
- `idle_reset_minutes`, 1–1440†;
- `print`: off, or a printer allowlist;
- `downloads`: off, or `non-executable-only`;
- `keyboard`: the shortcuts that remain. This can never remove SH 6.16's navigation set (focus movement, F6 and the keys screen readers rely on).

In the kiosk profile the broker **does not have** these capabilities, so there is nothing to hide:
- OS file pickers (no open or save dialog: the escape class in [HP 7.3–7.4][HP]);
- external protocol launch (the `ftp://` route);
- writing executable files;
- `about:`, `view-source:` and `file:` navigation;
- DevTools and the automation gateway;
- extension installs other than force-installs;
- the settings and diagnostics pages;
- new windows outside `layout`.

**Accessibility is not a capability the kiosk removes.** Assistive technology through the UI host's adapters (UIA/IA2, AX, AT-SPI), page and UI zoom, forced colours, text size and the accessibility-model grant all keep working. A kiosk-safe accessibility panel replaces the settings page, with no file or protocol hand-offs. Settings a person changes there last for their session and reset with it. The kiosk suite also passes SH 6.16's AT walkthrough (EP-14).

Each session runs in an **ephemeral kiosk container** that is destroyed on idle reset and on exit, and the reset re-creates every renderer. SH-D26's Stage 1 containers are fixed, so the shell amendment adds this container kind (A6).

Kiosk is configured by machine mandatory policy. `--kiosk <url>` starts the same profile for testing, labelled "Kiosk preview (not locked)". The OS must keep the browser running: on Windows, Assigned Access auto-restarts only UWP apps and Edge, so Izcal documents Shell Launcher or the restricted multi-app experience ([LG 6.3][LG]); on macOS, an OS single-app mode; on Linux, the session manager.

**Escape suite.** A published suite covers the NVISO and pentest classes ([HP 7.4][HP]; [LG 6.5][LG]): dialogs, protocols, downloads, `file://`, print-to-file, keyboard shortcuts, context menus, drag and drop, crash and restart, and new-window paths. The OS accessibility shortcut stays enabled. Its escape routes (help links and file dialogs inside OS accessibility settings) belong to the OS kiosk configuration, which the deployment guide covers. The target is 0 escapes (EP-14).

### 6.16 Legacy-browser switching (EP-D13)

Izcal **does not host a legacy engine**: Trident is proprietary and Windows-only, and D-S10 forbids another browser's code. It **switches**:
- `SiteSwitchingRules` (URL patterns → open in `SiteSwitchingBrowser`, the path and arguments of an admin-named browser, with a return-to-Izcal list);
- `SiteSwitchingSiteListUrl`, which reads the Enterprise Mode Site List **v.2 XML from Microsoft's published prose schema** ([LG 7.3][LG]). Sites with `open-in` IE11 or MSEdge are handed to Edge, which renders them in IE mode. An `https` list is fetched by the organisation container (6.10). A `file:` or UNC list is read by the broker through a read grant, as opaque bytes. Either way, the bytes reach the adapter through `site_list_part`, an IPC-R16 entry capped at 4 MiB† (A23).

The switch is a navigation decision in `hg-navigation`, made before a renderer commits, and recorded like any other navigation decision. Launching the other browser is a broker OS call with a fixed argument vector, never a shell string. The XML is parsed in the adapter with `hg-xml-syntax`. Demand runs at least to Microsoft's 2029 floor ([LG 7.1][LG]). The Windows site list matters only on Windows; generic rules work on every OS.

### 6.17 Enterprise reporting under FI-D12 (EP-D14)

FI-D12 refuses remote experiments and keeps **usage telemetry to Izcal** opt-in. Enterprise reporting is a different flow: from the organisation's browser to the organisation's endpoint. It never reaches Izcal, and the organisation, not Izcal, is the controller. Recommended:
- **Stage 2: inventory and policy health only.** Browser version and channel, OS, extension and app inventory (ID, version, install mode, granted capability classes), policy status codes per policy, and the digest. **No URLs, no history and no browsing events.**
- **Configuration.** `EnterpriseReporting = {endpoint (https), interval (6–24 h†, default 24 h), fields (subset of the catalogue's field list)}` is machine mandatory and **off when unset**.
- **Transport.** The organisation container (6.10) sends JSON over HTTPS, in a format Izcal publishes in prose, on the wake grid. OHTTP does not apply: the organisation is the controller and knows its own devices. NW-D17's Izcal-bound rule is unchanged.
- **Disclosure.** `about:policies` shows every field, the endpoint and the interval, generated from the same schema the reporter serialises from (EP-17). A high-impact notice appears when reporting starts.
- **Security-event reporting** (blocked malware downloads, phishing blocks, password reuse) is **Stage 3**. The default would be domain-level events only, and the design must first pass a data-protection review. The ICO requires notice and the least intrusive means, and German works councils co-determine monitoring tools ([LG 9.1–9.2][LG]).

### 6.18 DLP and content-analysis connectors (EP-D15)

Recommended: **deferred to Stage 3, with the hooks reserved now.**
- The interception points are already broker chokepoints: upload (file grants), paste (clipboard), drag and drop, print, and download completion. Each gains a typed, reserved **verdict hook** (allow, block or warn, with a timeout and a default result) that is a no-op in Stage 1 and Stage 2, so the design costs no rework.
- **The agent protocol in use today is defined only by Google's SDK code** ([Mozilla Hacks, 2025](https://hacks.mozilla.org/2025/03/improving-firefox-stability-in-the-enterprise-by-reducing-dll-injection/); [Firefox ContentAnalysis](https://firefox-admin-docs.mozilla.org/reference/policies/contentanalysis/); [LG 4.4][LG], Medium). M-1 part 3 forbids reading that code.
- **FI-D14's order, argued.** No open protocol exists for browser-to-DLP-agent verdicts, and no agent vendor implements one. An open protocol Izcal designs alone therefore does not "work" for buyers, whose agents speak only the SDK's protocol. Izcal does two things in parallel:
  - it **publishes its own open protocol** and offers it to agent vendors;
  - it writes a **black-box specification** of the SDK protocol by observing a real agent at the pipe boundary (M-1 part 2), or adopts a vendor's prose specification if one is published. A legal review of the SDK's terms comes first (Q-EP-6).
- **DLL injection is not a supported route.** Injected modules are detected and listed on `about:diagnostics`, and a crash-driven blocklist refuses them (PO-2.8). Mozilla describes injection as the outcome when no supported interface exists ([LG 4.5][LG]), which is why the hooks are reserved.
- The reversal trigger to bring DLP into Stage 2 is a published prose agent protocol, or a black-box specification passing interoperability tests with two agents.

### 6.19 OS single sign-on and Conditional Access (EP-D16)

- **Kerberos and Negotiate** already exist through NW-D20's allowlist (`AuthServerAllowlist`).
- **Entra and other cloud identity.** Device-based Conditional Access accepts only browsers on Microsoft's list, and on Linux desktop only Edge qualifies ([Entra Conditional Access conditions](https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-conditional-access-conditions); [LG 5.1][LG]). In Stage 2, `OsSingleSignOn=on` (default off) makes Izcal use the platform's documented SSO interfaces: on Windows, the OS's account-broker interface for attaching device credentials to sign-in requests; on macOS, Apple's Extensible Enterprise SSO interfaces. Both run in the per-container credential OS-adapter (NW 6.1), and only for hosts on `AuthServerAllowlist` or the identity provider's documented hosts (EP-25). `OsSingleSignOn` is `machine-opt-in` for the remote document.
- These are proprietary OS interfaces used through their documentation, which FI-D14 allows where no open route works. Getting onto Entra's supported-browser list is a business action with Microsoft (Q-EP-4); without it, tenants that require compliant devices block Izcal.

### 6.20 Installs, updates, VDI and profiles (EP-D17, EP-D18)

**Per-machine installs (EP-D17)** are FI-2's to build: an MSI with documented silent switches and a `.pkg` for MDM. They trigger SEC-D14's reversal in Stage 2, which needs the elevated updater's own threat model (Q-SEC-18). This spec fixes only the policy side:
- `UpdatesEnabled`, `UpdateChannel`, `TargetVersionPrefix` (pinning), `UpdatesSuppressedWindow` and `RelaunchNotificationPeriod` are machine mandatory;
- a pinned or suppressed update shows on `about:policies` and in SH 6.11's update row;
- SH 6.11's rule stands: no silent forced relaunch window. A policy may shorten the "Later" period to a minimum of 1 day†; it may not schedule a silent restart.

**VDI and multi-session (EP-D18).** Both incumbents roam a single profile file and break under concurrent sessions ([LG 8.1–8.2][LG]).
- `ProfileDirectory` places the profile, for example on a per-session disk.
- `EphemeralProfile` gives a profile that is deleted at exit, for non-persistent VDI.
- Concurrent use of one profile directory must be refused, including from a second session host on a shared disk. No spec defines that profile lock yet, so it is handed to FI-10 (profiles) and FI-4 (single instance) to design (A24).
- Izcal does not roam a profile file. State that should follow a person across sessions follows through FI-35's sync, which is end-to-end encrypted.
- Windows multi-session and Citrix/AVD support is a measurement row at EP-M4: footprint per session and the shared broker per OS user (FI-D17).

**Profile separation (PO-6.16).** Policy applies per OS user to all of that user's profiles. `ProfileCreationAllowed=false` limits a managed user to the profiles the organisation expects. Work-versus-personal separation inside one profile is SH-D26's containers.

### 6.21 Determinism, recording and the M1-exit input

**The pattern (recommended and accepted): OS-sourced inputs enter as typed IPC snapshots from an OS-adapter process. There is no new substrate trait.**

Why this pattern and not a `PolicySource` substrate trait:
1. **Recording is already done where recordings exist.** S1 3.4 records each recorded process's IPC inputs. A renderer's JIT ceiling or the shell's `ShellPolicy` arrives as a slice message and is in that process's recording with no new mechanism. The broker is unsandboxed and has **no production recording**. Broker-side policy replay therefore exists in DST, where every `PolicyHost` message is a scheduled input. In production, a broker crash is matched to its policy by the digest in the crash header.
2. **No ambient authority.** Substrate traits are effects every engine crate needs (`Clock`, `Entropy`, `Executor`, `Memory`, `Ipc`). A policy trait would put an OS-configuration capability into every process's substrate, renderers included, which I-2 forbids. Policy is the broker's alone, and G248 limits who may call the OS backend.
3. **No parsing in the broker** (IPC-R15). A trait implemented in the broker would parse there.
4. **Simulation for free.** In `hg-substrate-sim` the adapter is a scripted peer under the seeded scheduler. DST scenarios inject policy changes, malformed snapshots, adapter crashes and flapping sources as faults.
5. **No change to M1 as built for policy.** The traits and the wire format stay as merged. FI-4's launch requests, OS settings and the rest go through OI-D1's `OsPort`, not this pattern (see below).

**Agreement with the OS integration spec (owner decision of 25 October 2026, "both, split by source").** OS inputs enter by two routes, split by source:
- **Policy** uses this spec's pattern: a sandboxed adapter parses it and the broker receives typed `.hgidl` snapshots (6.6). No substrate trait carries policy.
- **What the OS delivers to the broker itself** (launches, default-browser status, settings, locale, time zone, hardware, displays, power, media keys, notification activations) goes through the broker-only **`OsPort`** substrate trait of the OS integration spec (OI-D1), recorded at that boundary in simulation.

`OsPort` therefore no longer carries policy, and the `Policy` row of OI 6.1's input inventory is removed (A27). The broker's change notifications for policy sources (6.6's watch API) carry no values and are not `OsPort` inputs. In simulation they are scripted triggers beside the scripted adapter. The two routes differ because policy sources are complex text formats that must be parsed outside the broker (IPC-R15), while `OsPort`'s inputs are OS-structured values the broker already receives.

**What M1's exit adopts (three items):**
1. **The shared recording and crash-report header** has **one definition, `RecordingHeader`, owned by FI-11** and placed in `hg-types` (L1/L2, crypto-free). This spec, the OS integration spec (OI-D26) and the compliance spec (§9.4) each contribute fields to it rather than defining their own. The union:

   | Field | In recordings | In crash reports | From |
   |---|---|---|---|
   | Header schema version | Yes | Yes | Compliance §9.4 |
   | Engine build ID (keys the published, signed **build manifest**: version, release line, **channel**, platform and ISA, tier ceiling, SBOM digest) | Yes | Yes | S1 M1 exit; compliance §9.4 |
   | Data versions | Yes | Yes | S1 M1 exit |
   | Decision-point log (reference) | Yes | — | S1 M1 exit |
   | OS version | Yes | Yes | Compliance §9.4 |
   | Process kind | Yes | Yes | Audit §8 point 1; compliance §9.4 |
   | Sandbox tier | Yes | Yes | Compliance §9.4 |
   | **Policy**: `PolicyDigest` (32 bytes) in recordings; **coarse policy flags** in crash reports | Digest | Flags | This spec; compliance §9.4 |
   | Profile identity: a per-installation random value, replaced by a fresh random value on every export or upload | Yes, re-randomised on export | — | Audit §8 point 1; compliance §9.4 |

   Channel is not a separate field, because the build manifest gives it. The OS integration spec's recording scope (OI-D26) adds no header field: a tab recording carries only the OS-derived values the tab consumed, inside its stream.

   **The policy field.**
   - **Recordings** carry `PolicyDigest`: 32 opaque bytes. In `.hgidl`, which has no fixed-array type, it crosses as `bytes<32>` with an exact-length check in the receiver.
   - **Crash reports carry coarse policy flags instead of the digest**, as the compliance spec decided: a digest of an organisation's policy set is close to unique to that organisation. The flags are a u64 bitset generated from a catalogue field `risk_flag`. Each bit says whether one class of risk-relevant policy is set: added trust anchors, organisation code-signing keys, force-installed extensions or apps, a raised JIT ceiling, a TLS floor, kiosk, remote automation blocked, DevTools blocked, OS scripting restricted, weakened protections. The crash handler receives them in `CrashPolicy`.
   - **What the digest hashes.** `PolicyDigest = SHA-256(canonical_policy_encoding)`, SHA-256 per FIPS 180-4, computed by `hg-policy` with `hg-crypto` in the broker. The header crate depends on no crypto crate.
   - **The canonical policy encoding** (`izcal-policy-v1`) is **owned by `hg-policy-catalogue`**, versioned and published with test vectors. It is not the IPC wire format, which is same-build only (VER-1). It encodes:
     - the tag `izcal-policy-v1`;
     - the catalogue version (u32 little-endian);
     - the effective entries **sorted by stable policy name**, each as (name, **level**, value). The level matters: a mandatory and a recommended value behave differently;
     - lists whose merge rule is a set, sorted and de-duplicated;
     - URLs and hosts in `hg-url`'s canonical form (case, IDNA, no trailing dot);
     - record keys in catalogue order;
     - integers in fixed little-endian widths, with no floats (G244).

     **`EffectivePolicy` excludes the person's settings** (6.4), so one digest holds for all of an OS user's profiles and does not change when a person flips a recommended row.
   - **`NO_POLICY`** is the digest of the encoding of the empty entry set, **with the catalogue-version field fixed at 0**, so an unmanaged machine's header always reads the same published constant on every build. With no policy, all the coarse flags are zero.
2. **The recorded-input rule.**
   - Every slice message carries `(epoch, digest)`, so a process's recording marks each policy change in-stream.
   - A recording's header digest is the one in force at its first input.
   - A saved bundle contains **only the slices its process received**, which are already in its stream. Only a shell UI recording carries `ShellPolicy`, and its `internal-hostnames` fields are masked under SH 6.17's origin-only redaction default.
   - No bundle carries the organisation's whole effective policy.
   - In DST, the broker's `PolicyHost` inputs are recorded and replayed like every scheduled message.
3. **The reserved `policy.hgidl`** of 6.6, checked in as a golden file that passes `hg-ipc-codegen`. The adapter runs as `ProcessKind::OsAdapter` with a Policy configuration. That case is added to `process.hgidl`'s closed enum when the adapter is built (M5 or later), and the enum is same-build only, so no versioning is needed.

**What M1 does not need:** no trait for policy (`OsPort` is OI-D1's, for other inputs), no change to `.hgidl`'s wire format, no change to `process.hgidl` before the adapter exists, no black-box measurements (those move to EP-M1), and no change to the merged phases A to D2. M1 cannot compute a real digest without `hg-crypto`. Until `hg-policy` exists, every M1 recording carries `NO_POLICY`, which is correct: M1 has no policy sources.

### 6.22 Security, privacy and consent

**Threat model for the policy adapter** (to add to SEC 6.5 and SEC 6.3a's OS-adapter row):

| Item | Content |
|---|---|
| Assets | Read access to the policy locations; the catalogue; no profile data, no keys, no network |
| Entry points | Registry values, managed-preference values and `/etc` files written by administrators; remote-document and site-list bytes routed by the broker (IPC-R16 entries) |
| Attackers | ATK-7 trying to plant policy in user-writable places (refused by the admin-only check on the opened object, 6.3); ATK-5 serving a remote document or site list (refused by signature, version, persisted highest version and expiry); ATK-2 from the broker side; an administrator is out of scope (SEC 6.1) |
| Mitigations | Parsers in `hg-policy-format` only, tier P, fuzzed (EP-11); strictest sandbox tier with read-only grants; on demand, exiting when idle; typed output validated against the catalogue in the adapter **and** decoded by generated `.hgidl` decoders in the broker; `CallerIdentity` binding; `panic = "abort"` (IPC-R5) |
| Residual | A compromised adapter can send any **catalogue-valid** policy. It cannot force AI on or switch phishing off (restrict-only is enforced again in the broker's merge), cannot send a value the catalogue cannot express, and cannot see profile data. Every value it sends is shown on `about:policies`. **ATK-7 residual:** same-user code can inject into the broker or the adapter per run, or replace per-user binaries (SEC-D14), and change effective policy that way. The admin-only check closes only the durable source route (6.3). The change ledger in the storage service leaves evidence |

**"Signed policy" (DV 6.10, AH 6.7) means:** a value from a **machine-scope mandatory** source, which is either an admin-only OS location (6.3) or a remote document verified with keys from such a location. OS policy channels carry no signatures; what the two specs need is a source the person and same-user malware cannot write durably. Amendment 9.5 replaces the term.

**Privacy.**
- Policy never makes Izcal send anything to Izcal.
- Organisation traffic runs in the organisation container, separate from the person's containers (6.10).
- Reporting goes to the organisation, off by default, and is disclosed exactly (6.17).
- The remote-document fetch carries no identifiers beyond what the organisation's own authentication requires.
- The person can always see what is managed, and can see the change notice (EP-D4).
- Exports and recordings mask internal hostnames by default.

**Consent.** A policy is not consent. Prompts that a policy pre-answers (device pre-grants, client-certificate auto-selection) show the indicator their owner spec requires (DV 6.10, NW 6.14), and the prompt ledger records "decided by policy".

### 6.23 Admin documentation and generated artefacts

`hg-policy-gen` (tier T) emits, from `hg-policy-catalogue`, reproducibly and in CI:
- **ADMX/ADML.** A namespace that changes with each catalogue version that changes the ADMX output, en-US first, a separate "Default settings (users can override)" category tree for recommended policies, no combo box, ≤ 1 MB, so Intune's import accepts it ([OD 2.x, 3.5][OD]).
- **The macOS preference manifest** and example `.mobileconfig` profiles for `dev.izcal.browser`, on both channels.
- **A JSON Schema** for the Linux, Flatpak and remote-document formats.
- **The `.hgidl` cases** of `PolicyScalar` and `PolicyListPart`, sized from the catalogue's encoded bounds.
- **The reference.** One page per policy, with type, scopes and levels, `remote` reach, apply timing, since and deprecated versions, owner, examples per OS channel (registry, OMA-URI, plist, JSON) and the Chromium-name mapping (EP-D2).
- **The per-release policy changelog** (new, changed, deprecated, removed), in the shape Firefox publishes ([LG 10.3][LG]).
- **The deployment guide** (per OS, VDI, kiosk including the OS kiosk configuration, troubleshooting with `about:policies`) and a **BSI Mindeststandard conformance table** ([LG 10.1–10.2][LG]).

The docs ship offline inside the browser, so `about:policies` links resolve without a network, and are also published on the website. These are generated from Izcal's catalogue alone; no other vendor's ADMX or schema is an input (M-1).

### 6.24 Crates and rules (G240–G249)

This spec's range is **G240–G249**, assigned by the coordinator on 24 October 2026 and recorded in S1 2.2's registry when the specs merge. **G270 onward goes to the next specification.**

| Crate | Layer and tier | Role |
|---|---|---|
| `hg-policy-catalogue` | L2 contracts (generated, beside `hg-ipc-gen`), tier P | The catalogue, its canonical encoding, generated types and test vectors; no I/O |
| `hg-policy-format` | L2p parsers, tier P (policy adapter only) | JSON and registry-JSON parsers; the site-list reader over `hg-xml-syntax`; remote-document verification |
| `hg-policy` | L5 services, tier P (broker) | Merge, status, digest, slices; pure |
| `hg-policy-adapter` | L5 services, tier P (run by `hg-os-adapter-main` in its Policy configuration) | Source reads through the OS backend, validation, snapshots |
| `hg-substrate-os::policy` | L1, tier O | Watch API (existence, notifications) and read API (registry, CFPreferences, `/etc`, with the admin-only check on the opened object) (G3) |
| `hg-policy-gen` | tier T | ADMX/ADML, manifest, JSON Schema, `.hgidl` cases, docs, changelog |

| Rule | Statement |
|---|---|
| **G240** | Only `hg-policy` computes effective policy; consumers receive slices, never snapshots or sources |
| **G241** | `hg-policy-format` is linked only by `hg-os-adapter-main` through `hg-policy-adapter`; the broker links no policy parser (IPC-R15) |
| **G242** | `hg-policy` is a pure function of its inputs: no clock, entropy or I/O; it is property-tested (EP-4) |
| **G243** | Every catalogue entry has an owner spec, an `enforced_in` set, a `remote` class, a `risk_flag` (possibly none), an encoded-size bound, a documentation text and a test; every settings-registry row marked policy-settable has exactly one entry (EP-1) |
| **G244** | No catalogue type can carry a secret or a float; the generator refuses a field named or typed as a password, token or private key |
| **G245** | Restrict-only entries are enforced in the catalogue validator, in the adapter **and** in the merge |
| **G246** | No code writes a policy value into the settings store, or into the bookmark store outside the `managed_by_policy` folder (non-tattooing) |
| **G247** | Templates, docs and the `Policy*` `.hgidl` cases are generated; a hand-edited one fails CI |
| **G248** | Only `hg-policy` (watch API) and `hg-policy-adapter` (read API) may call `hg-substrate-os::policy`, enforced by the dependency allowlist as devices' G210 does; the read API returns a value only after the admin-only check of 6.3 on the opened object |
| **G249** | A policy reaches a process only through the slice generated for it from `enforced_in`, which carries `(epoch, digest)`; a slice carries no field its receiver does not enforce (EP-20) |

### 6.25 Staging (EP-D19) and alternatives

**EP-D19: Stage 1 builds the pipeline; Stage 2 ships supported delivery.** FI-D9 puts delivery in Stage 2, but Stage 1 already shows "Set by your organisation" and B10's lock, and recordings need the digest.
- **Stage 1** builds the catalogue, generator, `hg-policy`, the adapter with all three OS backends, `about:policies`, the slices to every consumer, and the DST scenarios.
- **Release builds in Stage 1 read no OS source.** Their watch API is compiled out, so they launch no adapter and pay nothing. Development and Nightly builds read OS sources behind a launch flag, labelled "policy preview, unsupported", for lab testing (EP-M3). The preview reaches outside testers only once distribution's public signed Nightlies exist, which come at least 3 months before Stable and after an EU legal entity exists. Until then the preview runs in the lab.
- **Stage 2** turns sources on in every build, with the generated templates, the documentation, per-machine installers (FI-2) and a support commitment.
- The reversal trigger is an early enterprise adopter in Stage 1. The pipeline already exists, so enabling it is one release decision.

| Alternative | Recommendation | Cost | Reversal trigger |
|---|---|---|---|
| A `PolicySource` substrate trait (audit §8 point 2, first option) | **No**: 6.21 | A recording mechanism for free | A measured need to read policy outside the broker |
| Parse in the broker, read raw OS values | **No**: IPC-R15 | One process fewer | — |
| An always-on adapter holding the OS registrations | **No** (6.6) | Simpler watch logic | EP-M1 shows a source whose notification only the reader can hold: take its measured always-on cost to the owner as an FP-1 decision |
| Put policy reads in NW's per-container credential adapters | **No** (6.6) | One process kind fewer | — |
| Chromium-compatible names everywhere | **No**: EP-D2 | Admin familiarity | — |
| Domain or MDM gate for high-impact policies | **No**: EP-D4 | Silent failures, no Linux equivalent | Observed abuse of admin-only sources by non-administrator code |
| Izcal-operated cloud console | **No**: EP-D8 | A server and accounts | Three or more enterprise deployments request it: build a **self-hostable** reference console in Stage 3 |
| In-browser legacy engine | **No**: D-S10 | — | — |

## 7. M-1 compliance

- **Dependencies, unmodified:** OS interfaces only, called as their documentation describes: the Win32 registry and Group Policy notification APIs, CFPreferences, POSIX file and inotify interfaces. Ed25519 and SHA-256 come from `hg-crypto`, Izcal's own. **No third-party policy library, template or schema** is used or adapted.
- **Built new:** the catalogue and its generator; the merge; the adapter; the parsers (`hg-policy-format`); the remote-document protocol; `about:policies`; the kiosk profile; site switching; the reporting format.
- **Anchored to:** Microsoft's ADMX schema documentation and ADMX-ingestion and Policy CSP pages ([OD 2.1–3.3][OD]); Apple's ManagedPreferences payload and `CFPreferencesAppValueIsForced` pages ([OD 4.1, 4.3][OD]); RFC 8259 (JSON), RFC 8032 (Ed25519), FIPS 180-4 (SHA-256); Microsoft's Enterprise Mode Site List v.2 prose schema ([LG 7.3][LG]); Izcal's own specs for every behaviour a policy controls.
- **Evidence rule (M-1 part 3):** the research notes read rendered vendor documentation, maintainer and security posts, and forums. No third-party source code, header, commit, code review or in-repository data file (ADMX files, JSON schemas, SDK code) was opened or cited. The notes record where only code holds a fact: the DLP agent protocol ([LG 4.4][LG]), and a Chromium change reported from a code review, which was excluded ([HP, unverified list][HP]). Values only code states become black-box measurements (EP-M1 list, 11.2).

## 8. Targets

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **EP-1** | Catalogue completeness | **100%** of policies named in any spec are catalogued; **100%** of registry rows marked policy-settable have exactly one entry and vice versa | CI cross-check (G243) and a corpus scan at each spec revision | Build fails |
| **EP-2** | Generated templates are accepted | ADMX/ADML load in the Group Policy editor with **0** errors and import into Intune (≤ 1 MB, en-US, no combo box); generated profiles install on the two latest macOS releases; examples validate against the JSON Schema: **100%** | T2 lab on rig253 (Windows) and a Mac; CI schema check | Release blocker for Stage 2 |
| **EP-3** | One name, one meaning | Equivalent sources on Windows, macOS and Linux give identical `EffectivePolicy` and **identical digests** in **100%** of ≥ 200† conformance scenarios | Cross-OS conformance suite in CI | Fix the backend that differs |
| **EP-4** | Merge laws | Total, deterministic, source-order-independent, idempotent, monotone for restrict-only and union entries: **0** failures over 10⁶† generated cases per release | Property tests (G242) | Merge blocked |
| **EP-5** | Non-tattooing | Removing every source restores the person's prior value or the default for **100%** of setting-backed policies, and leaves **0** policy values in the settings and bookmark stores | Scenario suite; store diff | Release blocker |
| **EP-6** | Admin-only sources | **0** policies applied from user-writable locations in an injection suite (per-user install dir, `%LOCALAPPDATA%`, `$XDG_DATA_HOME` Flatpak path, a user-writable HKCU policy key, world-writable and user-owned `/etc` files, symlinks and check-then-swap races under `/etc`, registry links, plain `defaults write`) | ATK-7 lab suite on all three OSes | Release blocker |
| **EP-7** | Change latency | An OS-notified change applies (slice delivered) in **≤ 10 s†** p95 on Windows and Linux, including adapter launch; on macOS, within 10 s† of app activation | Lab timing; EP-M1 measures notification behaviour | Revisit 6.7 |
| **EP-8** | No idle cost | **0** periodic timers added by policy (FP-18); **0** adapter processes on a machine with no source and in Stage 1 release builds; the adapter exits ≤ 10 s† after its last request and stays ≤ 2 MiB† while alive (FP 8.2's OS-adapter row); broker policy state **≤ 256 KiB†** for 500 policies | FP lab meters | FP review (6.25 alternative) |
| **EP-9** | `about:policies` is right | On ≥ 200† scenarios: **100%** of displayed values, sources, statuses and reasons match the merge's ground truth | Scenario suite with oracle | Release blocker |
| **EP-10** | Admins can diagnose | ≥ **80%†** of administrators (n ≥ 12) identify why a planted policy did not apply in ≤ **3 min†** using `about:policies` alone | Moderated study at EP-M4 | Redesign the page |
| **EP-11** | Parsers hold | Each `hg-policy-format` parser is fuzzed **≥ 24 h†** per release cycle; **0** open crashes; **0** parsing code linked into the broker | Fuzz farm; G241 graph lint | Release blocker |
| **EP-12** | AI and telemetry cannot be forced on | **0** of all generated attempts (every source, level and encoding) yields `ai.enabled`, `accessibility.model`, usage telemetry or crash reports **on** through policy | Property test plus lab suite | Release blocker |
| **EP-13** | Phishing protection cannot be switched off | **0** catalogue paths disable PH's check | Catalogue lint plus suite | Release blocker |
| **EP-14** | Kiosk holds, and stays accessible | **0** escapes on the published kiosk suite (≥ 50† techniques, 6.15) on each OS, **and** the kiosk passes SH 6.16's AT walkthrough with NVDA, VoiceOver and Orca | Red-team suite and AT lab each release | Kiosk not offered on that OS |
| **EP-15** | Recordings name the policy | **100%** of recordings carry `PolicyDigest` and **100%** of crash reports carry the coarse policy flags (never the digest); a DST run with ≥ 3† mid-run policy changes replays byte-identically in **100/100** runs; the canonical encoding matches its published test vectors on all three OSes | DST farm; header audit; vector test | M1-exit item open |
| **EP-16** | Remote document is safe | **0** forged, rolled-back or expired-beyond-grace documents accepted on the attack suite, **including across a relaunch** (blocked fetch at start; an older valid document served after restart) | Adversarial suite (ATK-5) | Release blocker |
| **EP-17** | Disclosure is exact | **100%** of reported fields are listed on `about:policies`; **0** fields sent outside the configured set | Lab traffic capture against the generated list | Release blocker |
| **EP-18** | Docs complete | **100%** of entries have a generated reference page; **100%** of catalogue changes appear in that release's changelog | CI diff | Build fails |
| **EP-19** | Procurement baseline | The BSI Mindeststandard's central-configuration clauses (WB.2.1.03 d, 2.1.06 b–e, 2.3.03), MUSS and SOLLTE alike, are met **100%** | Conformance table review at Stage 2 | Close the gap before Stage 2 GA |
| **EP-20** | Slices match the enforcement map | For each process kind, the catalogue fields it receives equal the generated `enforced_in` map; **0** `internal-hostnames` fields reach a process that does not enforce them | CI check over generated slices (G249) | Build fails |
| **EP-21** | The change notice works | **100%** of high-impact changes in the scenario suite show the notice once, list every change and write a ledger entry that survives a restart | Scenario suite | Release blocker for EP-D4 |
| **EP-22** | Organisation code stays in its tier | **0** organisation-signed packages enter the directory trust tier; **0** force-installs honoured from a user-scope source | EX lab suite | Release blocker |
| **EP-23** | Site switching is exact | **100%** of a ≥ 200† case v.2 site-list and rule corpus switch correctly; the launch uses a fixed argument vector in **100%** of cases (no shell string) | Corpus test; launch audit | Fix before EP-M5 |
| **EP-24** | The adapter can fail safely | Adapter crash, restart and malformed-snapshot faults in DST leave effective policy unchanged or stale-marked in **100%** of runs; **0** partial snapshots applied | DST fault scenarios | Fix the broker's apply path |
| **EP-25** | SSO stays scoped | **0** device-credential attachments to hosts outside `AuthServerAllowlist` and the identity provider's documented hosts | Lab traffic capture | Release blocker for EP-D16 |
| **EP-27** | OS scripting cannot be forced on | With `blocked`: **0** scripting requests accepted on AppleScript, COM and D-Bus across the suite, and the debug-mode switch unavailable; with `allowlist-only`: **0** consents grantable to callers outside the list; **0** policy paths that enable debug mode, pre-grant consent or expose the surface | Lab suite per OS with scripted callers; catalogue lint | Release blocker for EP-D20 |
| **EP-26** | Reserved hooks cost nothing | The DLP verdict hooks add ≤ 1%† to upload, paste, print and download latency while no-op | Performance lab | Remove the overhead |

## 9. Milestones and staffing

### 9.1 Milestones

| ID | Aligns | Work | Exit criterion |
|---|---|---|---|
| **EP-M0** | M1 exit | `PolicyDigest` field in FI-11's `RecordingHeader`; the `NO_POLICY` constant; the reserved `policy.hgidl` checked in as a golden file that passes `hg-ipc-codegen` | M1 recordings carry `NO_POLICY`; the golden file passes CI |
| **EP-M1** | M5 | Black-box measurements: HKCU policy-key ACL; GP and registry notification behaviour for gpupdate, MDM and `reg add`; macOS forced-value notifications, the Managed Preferences layout and device-versus-user channel detection; sandbox read access to each OS source (rig253, a Mac, Linux). Catalogue v0 with every corpus policy; canonical encoding with test vectors; generator emitting Rust types, `.hgidl` cases and docs; `hg-policy` merge with property tests; DST scenarios with a scripted adapter | The measurement list in 11.2 has results; EP-1, EP-4, EP-20, EP-24 pass; EP-15's DST half passes |
| **EP-M2** | M5 (SH-M4) | Policy adapter on all three OSes (development builds); `about:policies` and the settings locks in SH-M4 | EP-3, EP-5, EP-6, EP-9, EP-11 pass in the lab |
| **EP-M3** | M7 | Slices to every enforcing process; Nightly policy preview (EP-D19) | EP-7, EP-8, EP-12, EP-13 pass |
| **EP-M4** | Stage 2 start | Supported delivery: ADMX/ADML, profiles, JSON Schema, Intune import, per-machine MSI and `.pkg` (FI-2), docs, changelog, BSI table; the organisation container; admin study | EP-2, EP-10, EP-18, EP-19, EP-21 pass |
| **EP-M5** | Stage 2 | Extensions and apps (organisation tier), managed bookmarks, site switching, kiosk, remote document, reporting, OS SSO, VDI rows | EP-14, EP-16, EP-17, EP-22, EP-23, EP-25, EP-26 pass |

### 9.2 Staffing

S1 8.3's shell row names "UI, profile, policy" within thirteen sub-tracks (168–243 engineer-months†). This spec's work is too large to carve out of that row, so it is an **additive enterprise policy sub-track**, with an S1 8.3 amendment row (A25):
- **Stage 1 (EP-M0 to EP-M3):** **1.5–2†** engineers, about 70–100 engineer-months† from M1 to M7. Catalogue, encoding and generator 0.5; merge, slices and DST 0.5; adapter and OS backends 0.5–1. A shell engineer from SH's settings sub-track builds `about:policies` within the shell row.
- **Stage 2 (EP-M4 to EP-M5):** **4–6†** engineers plus **1†** technical writer. Delivery and templates 1; kiosk 1; extensions, apps and site switching 1; remote document, organisation container and reporting 1; SSO and VDI 0.5–1; a security reviewer 0.5 shared with SEC.
- This is additive to FI-2's installer work.

### 9.3 Decisions for the owner

A bare "yes" accepts every recommendation.

| ID | Decision | Options | Recommended | Why | Reversal trigger | Status |
|---|---|---|---|---|---|---|
| **EP-D1** | Catalogue | (a) one Izcal catalogue generating every artefact, CI-bound to SH's registry; (b) hand-written templates per OS | **(a)** | No drift between OSes, docs and code ([HP 1.9][HP]) | — | **Accepted by the owner, 25 October 2026** |
| **EP-D2** | Names and shapes | (a) Izcal names, with Chromium names and shapes only where semantics match exactly, mapping published; (b) Chromium-compatible everywhere; (c) all new | **(a)** | Familiar where safe; a copied value that means something else fails visibly | Admin study shows the mapping confuses more than it helps | **Accepted by the owner, 25 October 2026** (reviewed explicitly) |
| **EP-D3** | Sources | (a) admin-only locations, checked on the opened object; never per-user install dirs or user-writable paths; (b) trust any documented location | **(a)** | ATK-7 is in scope and would use any user-writable location durably; the documented abuse used HKLM with administrator rights ([HP 3.5][HP]) | — | **Accepted by the owner, 25 October 2026** |
| **EP-D4** | Management gate | (a) no domain or MDM gate; high-impact changes are disclosed when they take effect and written to a persistent ledger; (b) Chromium-style join gate | **(a)** | No Linux equivalent; the gate defends against an out-of-scope attacker and fails silently when device state is misread ([LG 1.3][LG], Medium) | Abuse by non-administrator code is observed | **Accepted by the owner, 25 October 2026** (reviewed explicitly) |
| **EP-D5** | Precedence and merge | (a) 6.4's total order; merge rules fixed per policy (union only for restrictive lists; atomic groups); (b) Chromium's admin-configurable merge | **(a)** | Determinism; no merge that widens access | — | **Accepted by the owner, 25 October 2026** |
| **EP-D6** | Failure handling | (a) per-policy rejection; a failed OS source keeps its last snapshot for this run, marked stale, never persisted; the signed remote document is persisted with its highest version (6.10); (b) all-or-nothing; (c) a persisted last-good cache for OS sources | **(a)** | Visible, local failures; no unsigned user-writable cache; the signature makes the remote cache safe | Admin reports of outages caused by a corrupted file at launch | **Accepted by the owner, 25 October 2026** |
| **EP-D7** | Process and IPC | (a) an on-demand policy adapter (OS-adapter Policy configuration) reads, parses and sends typed snapshots; the broker watches through the OS backend, merges and slices by `enforced_in`; (b) a substrate trait; (c) broker reads and parses | **(a)** (accepted direction) | IPC-R15, I-2, FP 8.2, recording for free where recordings exist (6.21) | A sandbox cannot reach a source: pass handles from the broker | **Accepted by the owner, 25 October 2026** |
| **EP-D8** | Cloud management | (a) no Izcal console; a signed remote document under a published prose protocol, any host, fetched by an organisation container, with reach limited per policy; (b) an Izcal-run console; (c) adopt a vendor protocol | **(a)** | FI-D14; no open standard exists ([LG 2.7][LG]) | Three or more deployments ask for a console: a self-hostable reference console in Stage 3 | **Accepted by the owner, 25 October 2026** (reviewed explicitly) |
| **EP-D9** | Organisation trust tier | (a) organisation keys from machine policy; org-signed packages need no Izcal platform signature; force-install from machine scope only; (b) every enterprise package needs Izcal's signature | **(a)** | In-house extensions without Izcal review, visible to the person; resolves EX vs AH | — | **Accepted by the owner, 25 October 2026** |
| **EP-D10** | FIPS and crypto knobs | (a) refuse a FIPS mode; TLS floor 1.2 or 1.3 only; post-quantum not disableable; (b) a FIPS label on Izcal's own crypto | **(a)** | A FIPS label without a validated module is false ([LG 8.6][LG]) | A buyer requires FIPS 140-3: commission validation of `hg-crypto`'s TLS subset | **Accepted by the owner, 25 October 2026** (reviewed explicitly) |
| **EP-D11** | DevTools and automation | (a) three separate policies: DevTools, remote automation, tab recording (OS scripting is a fourth, EP-D20); (b) one switch | **(a)** | Automation is the infostealer route; recordings hold content | — | **Accepted by the owner, 25 October 2026** |
| **EP-D12** | Kiosk | (a) Stage 2 capability-reduced profile in an ephemeral kiosk container, machine policy only, accessibility kept, published escape suite; (b) UI lockdown over the normal profile | **(a)** | Escapes are OS hand-offs ([HP 7.4][HP]) | — | **Accepted by the owner, 25 October 2026** |
| **EP-D13** | Legacy browsers | (a) switching to an admin-named browser, reading the v.2 site list from its prose schema; (b) none; (c) host MSHTML | **(a)** | Demand to ≥ 2029; D-S10 forbids (c) | Microsoft ends IE mode: retire the site-list reader | **Accepted by the owner, 25 October 2026** |
| **EP-D14** | Enterprise reporting under FI-D12 | (a) Stage 2 inventory and policy health to an organisation endpoint only, off by default, no URLs, exact disclosure; security events Stage 3, domain-level by default; (b) refuse all reporting; (c) Chrome-level reporting | **(a)** | FI-D12 governs data to Izcal, not to the organisation; procurement expects inventory; the ICO and BetrVG constrain monitoring ([LG 9][LG]) | A data-protection review rejects the inventory fields | **Accepted by the owner, 25 October 2026** |
| **EP-D15** | DLP connectors | (a) defer to Stage 3, reserve broker verdict hooks now, publish Izcal's own open protocol and, after a legal review, interoperate through a black-box or published prose specification of the SDK protocol; (b) Stage 2 with Izcal's own protocol only; (c) never | **(a)** | The protocol in use is code-only ([LG 4.4][LG]) and no agent speaks an open one; hooks cost nothing now (EP-26) | A prose protocol is published, or a black-box spec passes with two agents | **Accepted by the owner, 25 October 2026** |
| **EP-D16** | OS SSO | (a) Stage 2, off by default, through documented OS interfaces in the credential adapter; seek Entra listing; (b) none | **(a)** | Entra device CA blocks unlisted browsers ([LG 5.1][LG]) | — | **Accepted by the owner, 25 October 2026** |
| **EP-D17** | Per-machine installs and update policy | (a) FI-2 builds MSI and `.pkg` in Stage 2, triggering SEC-D14 with its own threat model; update policies as 6.20; no silent forced restart | **(a)** | FI-D9; SH 6.11 | — | **Accepted by the owner, 25 October 2026** |
| **EP-D18** | VDI and profiles | (a) profile location and ephemeral profiles; a profile lock designed by FI-10 and FI-4; no profile-file roaming; state follows through FI-35; (b) roam a profile file | **(a)** | Both incumbents break under concurrent sessions ([LG 8.1–8.2][LG]) | — | **Accepted by the owner, 25 October 2026** |
| **EP-D19** | Stage 1 pipeline vs Stage 2 sources | (a) Stage 1 builds everything; release builds compile the watch API out and read no OS source; Nightly preview; Stage 2 ships support; (b) Stage 1 builds nothing; (c) Stage 1 ships supported sources | **(a)** | Respects FI-D9 while making the Stage 1 locks and digest real | An early enterprise adopter in Stage 1 | **Accepted by the owner, 25 October 2026** |
| **EP-D20** | OS scripting policy (OI-D18) | (a) a fourth separate policy, `OsScriptingAvailability`: `user-choice`, `allowlist-only` (with `OsScriptingAllowedCallers`) or `blocked`; never force-on; (b) fold it into `RemoteAutomationAvailability`; (c) also allow force-on with pre-granted consent | **(a)** | A different principal and attacker from DevTools and BiDi (EP-D11's reasoning); force-on would give same-user malware a page-reading channel (AGT-R8) | A managed workflow that needs scripting without per-app consent: design a supervised model meeting AGT-R8 first, never a policy switch | Proposed, 25 October 2026 |

### 9.4 What this spec closes

28 of the 29 FI-3 rows, with PO-6.4 deferred:
- **BF-20.1** (6.9); **BF-20.2** and **PO-1.17** (6.15);
- **PO-1.2** (EP-D17 with FI-2); **PO-1.16** (6.20); **PO-3.9** (6.20);
- **PO-6.1–6.3, PO-6.5** (6.3, 6.7); **PO-6.4** (Apple DDM) is **deferred**: Safari's declarations target Safari only, and DDM's app-configuration declaration has no macOS reader yet (Q-EP-5); **PO-6.6**: the Linux JSON format is the cross-platform format and the remote document reuses it; a file beside the binary is refused for per-user installs (6.3);
- **PO-6.7** (6.10); **PO-6.8** (6.4, 6.9); **PO-6.9** (6.2); **PO-6.10** (6.13); **PO-6.11** (6.11); **PO-6.12** (6.2); **PO-6.13** (6.1, 6.8);
- **PO-6.14** (6.17); **PO-6.15** (6.18); **PO-6.16** (6.20); **PO-6.17** (6.16); **PO-6.19** (6.23);
- **PO-7.3** (6.19); **PO-7.9** (6.12); **PO-8.5** (6.2); **PO-12.5** (6.11); **PO-13.11** (6.14).

### 9.5 Amendments to other specs (proposed, not applied)

| # | Spec | Section | Amendment |
|---|---|---|---|
| A1 | Networking | 6.14 | `izcal://policy` → **`about:policies`** (C1). The ten policies take this catalogue's names (6.2). "Applied by the broker per container" applies to every container until SH's Stage 2 container editing adds a container scope. Policy values arrive through the `NetworkSystemPolicy`, `NetworkContainerPolicy` and `PartitionNetworkPolicy` slices |
| A2 | Networking, S1 | NW 6.1; S1 §1 | One answer to who reads OS network configuration: an OS-adapter configuration through `hg-substrate-os`, as S1 says (G3); NW 6.1's "read by the broker as today" is amended to match. The policy adapter is another configuration of the same process kind |
| A3 | Networking | NW-D17, NW-36, NW-D3 | Add an **organisation traffic class**: a built-in, non-user-visible organisation container that exists only when machine policy names an organisation endpoint, carrying the remote document, site lists and reporting uploads, with the OS proxy, policy trust anchors and NW-D20's allowlist. NW-36 is unchanged for unmanaged machines (0 extra requests); for managed machines its endpoint list is the published list plus the configured organisation endpoints. OHTTP does not apply to reporting, because the organisation is the controller |
| A4 | Stage 1 design | §1 | The OS-adapter row gains the **Policy configuration**: per OS-user broker, on demand, strictest tier, read-only grants to the policy locations, no profile data and no network, exits 10 s† after its last request |
| A5 | Stage 1 design | 2.1–2.2 | Crates `hg-policy-catalogue` (L2), `hg-policy-format` (L2p), `hg-policy`, `hg-policy-adapter` (L5), `hg-policy-gen` (T), and `hg-substrate-os::policy`; register **G240–G249** for this spec, with G270 onward for the next |
| A6 | Stage 1 design, shell | 3.4, M1 exit; SH-D26 | One `RecordingHeader`, owned by FI-11, with the union of fields in 6.21: `PolicyDigest` in recordings, coarse policy flags in crash reports; the recorded-input rule of 6.21. SH-D26 gains an **ephemeral kiosk container** kind (6.15) |
| A7 | Shell | 6.9 | The registry row's "can be set by policy" becomes the catalogue name, levels and direction. One label family: "Set by your organisation", "Default from your organisation", "Installed by your organisation", "Allowed by your organisation" (revocable). Recommended values are changeable defaults. Weakened protections are labelled |
| A8 | Shell | 6.14 | `about:policies` contents as in 6.9 of this spec; the management summary and the high-impact change notice (EP-D4) |
| A9 | Shell | 6.11 | Update policies (6.20); a policy may shorten "Later" to no less than 1 day† and never schedule a silent restart |
| A10 | Devices | 6.10, 6.18 | "Signed policy" → "machine-scope mandatory policy" (6.22); "Set by your administrator" → "Allowed by your organisation", with revocability unchanged |
| A11 | Extensions | 6.6, 6.12, G171, Q-EX-5 | `storage.managed`: the package utility validates the managed value against the schema, and each component host receives only its own value; the "configuration reader" wording goes. The organisation trust tier (EP-D9). Install modes and caps go to `hg-ext-capability` in the broker (G171 unchanged). Force-install from machine scope only. Q-EX-5 partly answered |
| A12 | App host | 6.7 | Enterprise installs: organisation keys **or** the platform signature (EP-D9); "signed policy" as in A10 |
| A13 | Security scope | 6.1, 6.3a, 6.5 | The policy-adapter threat model (6.22) and SEC 6.3a's OS-adapter row ("Resolver fallback, SSPI/GSSAPI and client-key signing only") gain the Policy configuration; ATK-7's residual for policy (injection per run, binary replacement). SEC-D14's trigger is met in Stage 2 by EP-D17, which requires Q-SEC-18's answer first |
| A14 | Security scope | REC-R2, REC-R1 | Saved bundles carry only the slices their process received; `internal-hostnames` fields are masked |
| A15 | DevTools | Gateway, recordings | `DeveloperToolsAvailability`, `DeveloperToolsBlockedUrls`, `RemoteAutomationAvailability`, `TabRecordingAvailability` (6.14); the gateway refuses to start when blocked |
| A16 | H3 B10, ML service | B10, 6.x | Catalogue names `AiEnabled` and `AccessibilityModelEnabled`, restrict-only |
| A17 | JavaScript engine | JS-D13 | `ScriptJitCeilingRules` and `AppJitCeiling` names and types |
| A18 | Content blocking | 6.2 rule 3, 6.10, 6.12, CB-D19 | An **enterprise trust domain** for `UrlBlocklist` request rules that user rules and per-site controls cannot cancel; `ContentBlockingSiteExceptions` (proposed), unable to cancel Security entries; restrictive lists merge by union; this spec's URL-pattern grammar maps into CB filter syntax |
| A19 | Shell | 6.7, SH-D9, SH-19 | A 14th provenance value `policy` and a folder attribute `managed_by_policy`, both owned by the shell, with SH-19's schema test updated (G246) |
| A20 | Resource footprint | FP-1, 8.2 | FP 8.2's OS-adapter row gains the Policy configuration (on demand, ≤ 2 MiB†, exits after 10 s†, per OS-user broker); no FP-1 line is added, because nothing is always on |
| A21 | Coding standard | Tier table | The new crates' tiers; G240–G249 join the lint set |
| A22 | Feature inventory audit | §8 point 2, FI-2, FI-9, FI-11 | §8 point 2's choice is made (IPC, 6.21); FI-2 owns the MSI and `.pkg`; FI-9 uses `WebAppInstallForceList`; FI-11 owns the one `RecordingHeader` and its field union (6.21) |
| A23 | Security scope | IPC-R16 (via SEC-6) | Two new entries: the remote document and its signature (≤ 1 MiB† and 64 bytes) and the site list (≤ 4 MiB†), both broker → policy adapter, received by `hg-policy-format`, each with a fuzz target; owner: this spec |
| A24 | FI-10, FI-4 | Profiles, single instance | Design the profile-directory lock, including a second session host on a shared disk (VDI) |
| A25 | Stage 1 design | 8.3 | An additive enterprise-policy row: 1.5–2† engineers in Stage 1 (M1–M7) and 4–6† plus 1† writer in Stage 2 |
| A26 | Crash handler (S1 §1, D-S4) | — | The crash handler consumes `CrashPolicy` (`CrashReportsEnabled` and the coarse policy flags) |
| A27 | OS integration | 6.1 (`OsInput` inventory), OI-D1 | Remove the `Policy` row: policy is not an `OsPort` input (owner decision of 25 October 2026); `hg-policy` receives it from the policy adapter over `policy.hgidl` (6.6, 6.21) |
| A29 | OS integration | 6.14, OI-D18 | Policy hook: the scripting gate in `hg-os-broker` consumes `OsScriptingPolicy` (`user-choice`, `allowlist-only`, `blocked`); `blocked` disables the debug-mode switch and revokes consents; no policy enables scripting (EP-D20) |
| A28 | Compliance and security response | §9.4 | The header table points to FI-11's single `RecordingHeader`; the policy row's flag set is this catalogue's `risk_flag` bitset (6.21) |

## 10. Risks

| Risk | Likelihood | Impact | Mitigation | Trigger to act |
|---|---|---|---|---|
| A sandbox cannot read a policy source (macOS cfprefsd, Windows AppContainer registry access) | Medium | Adapter design | Broker-passed handles (6.6) | EP-M1 measurement |
| macOS gives no change notification | Medium | Latency | Re-read on activation and on the wake grid (6.7) | EP-M1 |
| macOS device and user channels cannot be told apart | Medium | Force-install unavailable on macOS | Treat forced values as user scope (fail safe, 6.3) | EP-M1 |
| HKCU policy keys turn out to be user-writable by default | Low (an archived Microsoft page says they are not, [HP 1.3][HP]) | User-scope policy unusable on unmanaged Windows | Reject as `source writable by user` (correct, visible); machine scope still works | EP-M1 ACL measurement |
| A notification only a long-lived process can hold | Low | Adapter must be always on | Take the measured cost to the owner as an FP-1 decision (6.25) | EP-M1 |
| Admins expect Chromium names | Medium | Friction | EP-D2 mapping table | EP-10 study |
| Entra listing unavailable | High | Blocked in compliant-device tenants | Q-EP-4; documented OS SSO interfaces | Stage 2 enterprise pilots |
| DLP buyers refuse a browser without connectors | Medium | Lost regulated buyers | Hooks reserved; EP-D15 trigger | Procurement feedback |
| Kiosk escape through an unlisted OS hand-off | Medium | Kiosk compromise | Capability removal, not UI hiding; suite grows with each finding | Any escape report |
| A catalogue-valid but harmful value from a compromised adapter | Low | Wrong policy | Restrict-only re-checked in the merge; everything visible | Adapter compromise in fuzzing |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-EP-1.** What is the default ACL of `HKCU\Software\Policies` for a standard user on Windows 11 24H2 and 25H2? Only an archived Microsoft page says the policy trees "cannot be modified by a user who is not an administrator" ([HP 1.3][HP]); no current page documents it ([OD 1.11][OD]). Measured at EP-M1 on rig253.
- **Q-EP-2.** Does `RegisterGPNotification` fire for MDM (Policy CSP) writes and direct registry writes? ([OD 1.13][OD]). EP-M1.
- **Q-EP-3.** Does a running macOS app learn of a forced-preference change, what does `/Library/Managed Preferences/` hold for device, user and manual profiles, and how can a reader tell the device channel from the user channel? ([OD 4.5, 4.10][OD]). EP-M1.
- **Q-EP-4.** Can Izcal be added to Entra Conditional Access's supported-browser list, and on what terms? Business action, before EP-M5.
- **Q-EP-5.** When DDM's `AppManaged` declaration gains a macOS reader, should it become a third macOS source? ([OD 4.11–4.12][OD]). Watch at each macOS release.
- **Q-EP-6.** Is there a prose specification of the DLP agent protocol Google's SDK implements ([LG 4.4][LG], absence after limited search), and do the SDK's terms permit a black-box reimplementation? Needs a legal review. Before EP-D15's Stage 3 decision.
- **Q-EP-7.** What is the Flatpak policy extension ID and host path under Izcal's Stage 2 Flatpak packaging? With FI-2.
- **Q-EP-8.** What fields may enterprise inventory reporting carry under a data-protection review (EU, UK, German works councils)? Before EP-M5.

### 11.2 Claims that are single-source, unverified or conflicting

- Chrome's registry policy on non-domain machines: chromium.org says it is not loaded, while a support.google.com page lists non-domain machines as a registry use case ([OD 1.2–1.3][OD]). Izcal's design does not depend on either.
- The Group Policy refresh interval (90 min plus 0–30 min) was read only from a search snippet of an archived page ([OD 1.12][OD]).
- The date Chrome first gated off-store force-install on managed machines, and Google's stated reason, are unverified; only Edge's wording was read ([HP 3.4][HP]).
- The 2014 and 2015 Chrome blog posts were read through feed summaries ([HP 3.2–3.3][HP], Medium).
- That Mozilla's 2018 policy engine made GPO replace `policies.json` rests on a snippet; the Firefox 87 release note that changed it is High ([HP 4.6][HP]; [OD 6.7][OD]).
- ADMX ingestion's first release (Windows 10 1703) is unconfirmed ([HP 6.5][HP]).
- That the DLP agent protocol has no prose specification is an absence claim ([LG 4.4][LG]).
- Chrome's Extensible SSO support on macOS: a Jamf blog post conflicts with Entra's page ([LG 5.5][LG]).
- Enterprise Mode for IE11's original announcement and IE mode's Build 2019 post were read through press coverage ([HP 2.1–2.2][HP], Medium).
- The Chrome 2014 store-only rule, the 75% figure and the 2010 and 2019 Chrome management history could not be re-read by the fact-checker and stay Medium (factcheck rows marked Unverifiable).
- Firefox Enterprise's announcement is dated 13 October 2026 in [LG 2.5] and 12 October on the blog page the fact-checker saw. The spec does not depend on the day.
- Black-box measurements in place of unverifiable values: [OD]'s eight questions, of which Q-EP-1 to Q-EP-3 are the ones this design depends on.

### 11.3 Contradictions found between existing specs

| # | Contradiction | Where | Resolution proposed |
|---|---|---|---|
| EP-C1 | The policy page is `about:policies` vs `izcal://policy` (audit C1) | SH 6.9, 6.14 vs NW 6.14 | `about:policies` (A1) |
| EP-C2 | Three labels for related facts, one of them revocable | SH 6.9 vs DV 6.18 vs EX 6.12 | One family owned by SH, with a revocable label (A7, A10) |
| EP-C3 | "Signed policy" is undefined; OS channels are unsigned | DV 6.10, AH 6.7 | Machine-scope mandatory source (6.22, A10, A12) |
| EP-C4 | Enterprise packages need Izcal's platform signature vs come from the organisation's own signed source | EX 6.12 (both sentences) and AH 6.7 | Organisation keys (EP-D9, A11, A12) |
| EP-C5 | Who reads OS network configuration | S1 §1 (OS-adapter) vs NW 6.1 (broker) vs audit §8 point 2 | The OS-adapter, under G3 (A2) |
| EP-C6 | FP 8.2 budgets the OS adapter as on demand per container; S1 §1 lists it as "Always" | FP 8.2 vs S1 §1 | On demand for the Policy configuration (A4, A20); the network configuration's own count is NW's to settle |

## References

[FI]: Izcal%20feature%20inventory%20audit.md
[HP]: ../research_notes/Izcal%20enterprise%20policy/history_postmortems.md
[OD]: ../research_notes/Izcal%20enterprise%20policy/os_delivery_formats.md
[LG]: ../research_notes/Izcal%20enterprise%20policy/landscape_admin_gaps.md

- **[HP]** `research_notes/Izcal enterprise policy/history_postmortems.md`: history and postmortems.
- **[OD]** `research_notes/Izcal enterprise policy/os_delivery_formats.md`: OS delivery mechanics, formats and precedence.
- **[LG]** `research_notes/Izcal enterprise policy/landscape_admin_gaps.md`: today's landscape and admin gaps.

**Primary sources that decisions rest on** (rendered vendor documentation and posts; no source code):
- Group Policy trees and tattooing: [Microsoft, Windows NT 4.0 system policies](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc728021(v=ws.10)); [Implementing registry-based Group Policy](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/bb742499(v=technet.10)).
- ADMX ingestion and its forbidden paths: [Win32 app ADMX policy ingestion](https://learn.microsoft.com/en-us/windows/client-management/win32-and-centennial-app-policy-configuration).
- Change notification: [RegisterGPNotification](https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-registergpnotification); [RegNotifyChangeKeyValue](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regnotifychangekeyvalue).
- macOS: [ManagedPreferences payload](https://developer.apple.com/documentation/devicemanagement/managedpreferences); [CFPreferencesAppValueIsForced](https://developer.apple.com/documentation/corefoundation/cfpreferencesappvalueisforced(_:_:)); [AppManaged declaration](https://developer.apple.com/documentation/devicemanagement/appmanaged); [WWDC21: Meet declarative device management](https://developer.apple.com/videos/play/wwdc2021/10131/).
- Linux and Flatpak: [chromium.org Linux quick start](https://www.chromium.org/administrators/linux-quick-start/); [Firefox: configuring policies](https://firefox-admin-docs.mozilla.org/guides/policies-configuration/); [Flatpak extensions](https://docs.flatpak.org/en/latest/extension.html).
- Precedence and cloud: [Chrome policy precedence](https://support.google.com/chrome/a/answer/9037717); [Edge management service](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-management-service); [Firefox Enterprise](https://firefox-admin-docs.mozilla.org/guides/firefox-enterprise/).
- Reporting and privacy: [Chrome browser reporting](https://support.google.com/chrome/a/answer/9301421); [ICO: monitoring workers](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/employment/monitoring-workers/data-protection-and-monitoring-workers/).
- Legacy switching: [Enterprise Mode schema v.2](https://learn.microsoft.com/en-us/internet-explorer/ie11-deploy-guide/enterprise-mode-schema-version-2-guidance); [Chrome Legacy Browser Support](https://support.google.com/chrome/a/answer/9270076).
- Procurement: [BSI Mindeststandard Webbrowser](https://www.bsi.bund.de/DE/Themen/Oeffentliche-Verwaltung/Mindeststandards/Webbrowser/Webbrowser_node.html).

## Appendix A. Review disposition (revision 0.2)

**Fact-check** (`research_notes/Izcal enterprise policy/factcheck.md`): 67 claims checked (41 correct, 17 partly correct, 6 wrong, 3 unverifiable); the link check passed. All six wrong rows are fixed:

| Row | Finding | Disposition |
|---|---|---|
| 9–10 | The documented policy abuse was attributed to ATK-7; it was elevated installers writing HKLM (out of scope), and ChromeLoader loaded at launch, not through policy | Fixed: summary, 2.2, 4.4, 6.3 and EP-D3 now name the elevated installer and state ATK-7's route as the user-writable location |
| 39 | BSI's normative levels were overstated | Fixed: §1 separates MUSS from SOLLTE |
| 56 | The G-rule range needed the coordinator's assignment | Fixed: the coordinator assigned **G240–G249**; G270 onward goes to the next spec |
| 58 | `bytes<65536>` could never be sent | Fixed: 6.6's IDL chunks at `bytes<60000>` and passes the checker |
| 61 | FP-17 was cited for the idle budget, and FP-M0's result was misdescribed | Fixed: FP-18, and FP-M0 as an empty sandboxed process |

Every partly-correct row is applied as the note proposes:
- 3: the ACL wording;
- 11: Edge's gate, now cited to HP 3.4;
- 13: IE11 "on certain Windows 10 versions";
- 19: Medium grading;
- 22: the ADMX namespace changes with each ADMX change;
- 31: the post-quantum wording;
- 36: Firefox 138 "per Mozilla";
- 44: inferred motives marked;
- 45: the hive mapping;
- 46: the gate's failure, as Medium;
- 49–50: DV citations;
- 57: the `MAX_MESSAGE_BYTES` citation;
- 59: message sizing;
- 63: PO-6.4 deferred;
- 64: "audit §8 point N";
- 66: crash reports are opt-out.

**Design critique** (`research_notes/Izcal enterprise policy/critique.md`): 24 findings (5 High, 13 Medium, 6 Low), all accepted:

| # | Severity | Finding | Disposition |
|---|---|---|---|
| C1 | High | `policy.hgidl` fails the generator and cannot carry the catalogue's types | Rewritten: closed enums, `///` docs, no reserved names, generated scalar and list-part unions sized from encoded bounds, a flattened tree, chunked remote and site-list bytes with a signature field; **passes `hg-ipc-codegen`** (6.6); golden file at EP-M0 |
| C2 | High | The remote document's rollback and grace state lives only in memory | The signed document, its signature and the highest version are persisted per OS user and re-verified at launch; ATK-7's residual stated (6.10); EP-16 covers relaunch |
| C3 | High | An always-on adapter breaks FP-1 | The adapter is on demand; the broker watches through the OS backend; zero adapters on unmanaged machines and in Stage 1 releases (6.6, EP-8, A4, A20) |
| C4 | High | Organisation traffic contradicts NW-D17 and skips IPC-R16 | An organisation container (6.10, A3); IPC-R16 entries (A23); `file:` and UNC site lists through a broker read grant (6.16) |
| C5 | High | Slices went to the component host and missed enforcing processes | `enforced_in` field; slice map with partition, crash, update, storage and extension-capability receivers (6.6); EP-20; G249 |
| C6 | Medium | Crate layers and the backend's callers | Catalogue at L2, format at L2p over `hg-xml-syntax`, `hg-policy-adapter` named, G248, one `ProcessKind::OsAdapter` (6.24) |
| C7 | Medium | Per-user brokers; the macOS channel split | One adapter per OS-user broker; macOS device and user sources split; fail-safe user scope until measured (6.3, 6.6) |
| C8 | Medium | The digest's scope and canonical form | `EffectivePolicy` excludes settings; a versioned canonical encoding with levels, sorting, URL canonicalisation and test vectors; `NO_POLICY` fixed (6.21) |
| C9 | Medium | Overstated recording claims; whole-policy bundles | Broker replay only in DST; bundles carry their own slices, masked (3, 6.21, A14) |
| C10 | Medium | URL blocklist cancellable; non-existent pattern syntax | The grammar is defined here; a navigation decision plus an enterprise CB domain users cannot cancel (6.1, 6.8, A18) |
| C11 | Medium | The admin-only check overstated against ATK-7 | Restated in SEC's terms with the residual; the ledger persisted in storage (6.3, 6.22) |
| C12 | Medium | The remote document's reach | `remote` field (`never`, `machine-opt-in`, `allowed`), `allowed_classes`, the rank argued, no self-set fetch proxy (6.4, 6.10) |
| C13 | Medium | Kiosk versus accessibility | Accessibility is kept; the OS shortcut stays enabled; AT walkthrough added to EP-14; ephemeral kiosk container (6.15, A6) |
| C14 | Medium | Bookmark provenance and profile lock | A 14th provenance value proposed to the shell (A19); the lock handed to FI-10 and FI-4 (A24) |
| C15 | Medium | Labels erased revocability | "Allowed by your organisation" for revocable grants (6.8, A10) |
| C16 | Medium | Staffing called a carve-out; EP-M0 overloaded | Additive with an S1 8.3 row (A25); measurements moved to EP-M1 |
| C17 | Medium | Managed-storage validation in the adapter | Validated in EX's package utility; the adapter checks structure only (6.11, A11) |
| C18 | Medium | Missing amendments | A3, A13, A18–A26 added |
| C19 | Low | "Protections" too broad | Narrowed; weakenable protections listed and labelled (4.4, 6.1, 6.9) |
| C20 | Low | Catalogue gaps | `DeveloperToolsBlockedUrls`, a path type and record schemas added (6.1, 6.2) |
| C21 | Low | Check-then-open races | Checks on the opened object, `O_NOFOLLOW`, registry links refused; EP-6 cases (6.3) |
| C22 | Low | DLP inverted FI-D14 | The argument is stated; Izcal's open protocol is published in parallel; legal review (6.18, Q-EP-6) |
| C23 | Low | Decisions without targets | EP-20 to EP-26 |
| C24 | Low | The header crate was unnamed | `RecordingHeader` in `hg-types`, owned by FI-11; `bytes<32>` with an exact-length check (6.21) |

**IDL check.** The `policy.hgidl` text in 6.6 was compiled with `hg_ipc_codegen::generate_unformatted` (the merged M1 generator, used as a library from a scratch crate outside the repository) on 24 October 2026. It passed with no errors. One earlier variant name, `Ok`, was reserved and was renamed `Read`.

**Link check (after the fixes).** 31 external links were added, inline and in the References list. 30 return HTTP 200. `hacks.mozilla.org` returns 403 to automated clients, but the fact-checker read that page in this session. The four relative links (`[FI]`, `[HP]`, `[OD]`, `[LG]`) resolve. No link points to third-party source code (M-1 part 3).
