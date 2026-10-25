# Izcal distribution and updates

*First-principles specification of how a signed Izcal build reaches a machine and stays current: the install layout that lets binaries change under a running multi-process browser, installers and packages per OS, code signing and reputation, channels, cadence, staged rollout, halt and revert, delta updates, background and metered behaviour, the Cyber Resilience Act's update duties, uninstall and leftover cleanup, first-run inputs, platform targets and supported-OS policy. Date: 24 October 2026. Status: draft 0.4, **every decision (DU-D1 to DU-D19) accepted by the owner, 25 October 2026**. Research notes: `research_notes/Izcal distribution and updates/` (`history_postmortems.md`, `platform_packaging_signing.md`, `update_mechanics_release.md`, `lifecycle_ux_regulation.md`; phase 8: `factcheck.md`, `critique.md`). Relates to: feature inventory audit [FI-2, C12, FI-D2, FI-D9, FI-D10, FI-D11, FI-D12, FI-D14, FI-D17, Section 8 point 3][FI]; HGBA-2 [M-1, OD-7, 2.3.2, P1–P6][H2]; Stage 1 design [Section 1 (broker, Updater, Linux spawner, crash handler), 2.1, 2.2, 3.1, 7.1, 7.8, 8.1 (M1, M5, M8), 8.3, 13.2 (Updates, Packaging, Profiles), D-S4, D-S8][S1]; security scope [6.5 updater threat model, 6.7, 6.16, UPD-R1–UPD-R5, SEC-1, SEC-8, SEC-13–SEC-17, SEC-34, SEC-D4, SEC-D11, SEC-D14][SEC]; shell [6.9, 6.11, 6.14, SH-8, SH-24, SH-D21, SH-D29][SH]; phishing [PH-D12, PH-6][PH]; networking [NW-D7, NW-D17, 6.13, 6.17][NW]; storage [SG-D2][SG]; graphics [GR-D21][GR]; resource footprint [FP-25, FP-D1, 6.8][FP]; JavaScript engine [JS-16, JS-D22][JS]; coding standard [Q-CS-1, CS-ST-18, A.4][CS]; M1 code (`crates/hg-ipc-gen/idl/process.hgidl`: `BuildId`, `ExitReason::BuildMismatch`).*

*Revision 0.4.1 (25 October 2026): G255 and OS integration's G231 named different link sets for `hg-launch-main`; both now name the same set, the union (`hg-substrate-os`'s launch entry, `hg-install`, `hg-launch-wire`, `hg-xdg`, `hg-os-journal`). No decision, target or threshold changed.*

*Revision 0.4 (25 October 2026): owner decisions recorded. **The owner accepted every decision in 6.18 as recommended (DU-D1 to DU-D19)**, and reviewed DU-D1, DU-D5, DU-D8, DU-D14 and DU-D15 explicitly; each row is marked "accepted by the owner, 25 October 2026". DU-D15 was reviewed together with the compliance spec's **CO-D5 (c)**, so its support statement is now the same rule: **the support end stated for a release on each platform is fixed at release as the earlier of five years and that OS version's vendor end of security support** (Izcal's conservative estimate where the vendor publishes none, as for macOS), with a security-only branch only if the OS loses support before that stated date, and only until it (6.11, DU-D15, DU-16). This replaces 0.2's "product support period ≥ 5 years separate from per-OS availability" and its open-ended Art. 13(10) branch. The crate rules stay G250–G255. No target threshold changed.*

*Revision 0.3 (25 October 2026): owner decisions and alignment with the OS integration spec. **DU-D4 accepted as recommended**: Stable ships a feature release every 4 weeks with security-only point releases between them, and moves to 2 weeks after the full pipeline runs in under 3 working days for two quarters. **DU-D10 accepted**: an EU-established legal entity first (with the compliance spec's CO-D1), then signed public Nightlies, as CRA Art. 4(3) testing builds, at least 3 months before the first Stable; the first public binary is therefore the first public Nightly (compliance CO-D3). **OI-D3 of `reports/Izcal OS integration.md` adopted as a requirement**: on macOS the `Izcal.app` bundle executable **is the broker**, children come from versioned helper frameworks inside the bundle (`Contents/Frameworks/Izcal <version>.framework/Helpers/`) pinned at start, cold command-line launches go through Launch Services, and an update stages a complete signed bundle and swaps it with an atomic exchange (6.5, DU-D11; the sibling-bundle layout of 0.2 is withdrawn). The launcher crate is OI's `hg-launch-main` with its hand-off `hg-launch-wire`; the install ledger becomes OI-D10's registration journal (`hg-os-journal`) with four record kinds added; Linux launchers sit at `/usr/bin/izcal[-<channel>]` and locks in OI's per-channel runtime directory; this spec fixes the per-channel application identifiers (Q-OI-12) and records the OS floors OI recommends (Q-OI-14). No target threshold changed.*

*Revision 0.2 (24 October 2026): phase 8 applied. The independent fact-check checked 80 links, every CRA quote against the Official Journal text and every internal ID, and raised 38 findings (0 blockers, 2 major, 36 minor); the independent design critique raised 40 (3 blockers, 17 major, 20 minor). **All 78 are applied** (Appendix B). What changed: `BuildId` is a digest of the build inputs compiled into every binary, not of the file manifest (C1); Linux packages get automatic security updates by default through the distribution's own unattended-upgrade tools configured for Izcal's repository, and are recorded as a per-machine install whose elevated component is the package manager (C2, C5); the online `rollout` role is bounded by the client: security releases are outside it, ramps have a maximum, holds expire unless the offline role delists (C3); ramp positions are fresh per release (C9) and ramps are a per-channel property, so the phishing, revocation and extension-blocklist channels are not ramped (C10); liveness locks move out of build directories and the broker pre-opens its whole build, with Izcal's executables carrying no Izcal-shipped shared libraries (C4, C6, F2); macOS recommends sibling build bundles (C7); the headless update session names its processes and its collision rule (C8); revert is a rebuild of the previous source with deletion replay (C11); gates are per release class with a security time budget inside SEC-1 (C12); the CRA opt-out governs browser builds only, and Nightly and Beta are designed to CRA Art. 4(3) (C14); one per-user data root holds data channels (C15); G251 is scoped to Izcal's own process kinds and the substrate gains an install-device trait (C16, C17); DU-M0 is split, and the first-public-binary gates move with the signing lead time (C18); platform targets become a rule applied at each release decision (C19); staffing is estimated per crate (C20); crash reports are described as D-S4's opt-out with a per-crash prompt (F1); citations, dates and counts are corrected throughout; the crate rules take the registry block G250–G259 assigned on 24 October 2026 (G250–G255 used).*

*Revision 0.1 (24 October 2026): first draft, written by the synthesis author from the four research notes and the accepted specs. No other report is edited; Section 12 lists the amendments this spec proposes.*

Prefix check: **DU-** (targets DU-1…, decisions DU-D1…, milestones DU-M0a…, questions Q-DU-…) collides with no ID prefix in `reports/` (checked 24 October 2026). The app host cites a research note under the link label `[DU …]`; that is a citation label, not an ID. **G-rule range:** the registry owner assigned **G250–G259** to this spec on 24 October 2026 (OS integration G230–G239, enterprise policy G240–G249, compliance G260–G269, G270 onward to the next spec); this spec uses **G250–G255** and leaves G256–G259 unassigned. Thresholds marked † are set by this document from first principles; none is measured on Izcal yet. Values that are legal floors (for example the CRA's five and ten years) carry no †.

**What this closes.** The 27 ledger rows of work item FI-2: BF-18.7, PO-1.1, 1.3, 1.4, 1.5, 1.6, 1.8, 1.9, 1.10, 1.11, 1.12, 1.13, 2.2, 2.6, 3.1, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.13, 3.14, 4.9, 11.2, 11.5, 16.9 (row map in Appendix A; the critic confirmed it matches the ledger exactly). It settles contradiction **C12** as the owner decided in **FI-D10**, answers the audit's Section 8 point 3 (the `BuildId` check under a binary replacement) with a design rather than a fallback, and gives M1 phase E and M5's process launch their inputs (Section 13). It does not deliver enterprise policy (FI-3), OS integration and the single-instance hand-off message (FI-4), or the compliance runbook (FI-1); it names what each needs from distribution.

## Summary

A browser is no longer one process that reads its files once. Izcal launches renderers, services and helpers all the time, and its IPC refuses a child from another build: `process.hgidl` sends a `BuildId` in `ready`, and the browser answers a mismatch with `ExitReason::BuildMismatch`. So the first question for distribution is not "how do we download a new build" but **"what happens to the next tab when the files change under a running browser"**. The record answers it. Chrome on Linux forks every renderer from a zygote because otherwise "a version x browser might be talking to a version y renderer", which its IPC "does not support this (and does not want to!)" ([Chromium zygote design][CZ]); Firefox, whose macOS bundle is flat and whose Linux packages are replaced in place, shows `about:restartrequired`, and in January 2026 a background update applied under a running instance still left new tabs spinning ([Bugzilla 2010328][BZ2010328]; [DUM 1.3–1.4][DUM]). **Izcal installs every build into its own versioned directory on every OS and in every package format, launches every child from the running build's own directory, pre-opens that build's files, and deletes a build only when no process of it is alive** (DU-D1). With that layout the `BuildId` refusal never fires in a normal update, which DU-1 tests.

Four more decisions carry the design. **Staged rollout without per-client data** (DU-D5): Izcal has no account in Stage 1, collects usage telemetry only by opt-in and crash reports only through D-S4's per-crash prompt, so it cannot copy rollouts that widen on vendor-side identity. Each install derives a fresh ramp position for each release from a local secret that never leaves the device; the ramp is signed TUF metadata from a new `rollout` role that can only choose among targets the `releases` role signed, whose power the client bounds (security releases are outside it, every ramp has a maximum, a hold expires unless `releases` delists the target). The CrowdStrike outage of 19 July 2024, in which an unstaged content update crashed about 8.5 million Windows devices ([Microsoft][MS-CS]; [RCA][CS291]), is why ramps are a property of every channel whose data code can crash on, not only of binaries. **Linux packages from Izcal's own signed repositories** (DU-D8, FI-D10): each build is its own package installed into its own directory, as distributions install kernels, so the system package manager adds a build beside the running one; Izcal's TUF client stays off for the browser binary there, the distribution's own unattended-upgrade tool is configured for Izcal's repository so security updates install automatically, and the TUF client remains the verifier for data channels. **The Cyber Resilience Act's update duties are product requirements** (DU-D14): automatic security updates "enabled as a default setting, with a clear and easy-to-use opt-out mechanism … and the option to temporarily postpone them", security updates "provided separately from functionality updates" where technically feasible, a support period of at least five years with its end date shown as month and year, each security update kept available for at least ten years or the rest of the support period, whichever is longer, and a way to "remove on a permanent basis all data and settings" ([CRA][CRA]; [DUL 1][DUL]). **Signing identity is on the launch critical path** (DU-D10): SmartScreen reputation builds on the publisher certificate and on each file hash and "can take several weeks and hundreds of clean installs" ([Microsoft][SSR]); EV certificates no longer shortcut it; macOS 15 removed the Control-click override, so every build that reaches a person must be notarised ([Apple][SEQ]).

Three stores are closed and are recorded as refusals, not gaps: the **Microsoft Store** requires browsers to use the Chromium or Gecko engine (policy 10.2.1, Store Policies 7.20, published 15 September 2026, effective 22 October 2026, [Microsoft][MSP]); the **Mac App Store** requires WebKit for browsing apps and forbids an app's own updater (guidelines 2.5.6 and 2.4.5; the alternative-engine entitlement is for iOS and iPadOS in the EU and Japan only, [Apple][ARG]); **Snap** is not offered in Stages 1–2 because the `.deb` reaches the same users and Snap's store controls updates outside Izcal's TUF chain (DU-D8). Flatpak stays Stage 2 as S1 13.2 decided.

Note sources are cited as **DUH** ([history and postmortems][DUH]), **DUP** ([platform packaging and signing][DUP]), **DUM** ([update mechanics and release][DUM]) and **DUL** ([lifecycle, UX and regulation][DUL]), with the row number. Primary sources are linked inline for the claims a decision or target depends on.

## 1. The problem and who has it

| Who | Task, in their words | What fails today, with evidence |
|---|---|---|
| **People using Izcal** | "Keep me safe without breaking what I'm doing" | Silent background updating put 97% of Chrome users on the newest version after 21 days, against 24% for Opera's manual reinstall ([Duebendorfer and Frei, 2009][DF09], Medium: the figures are from the paper as summarised, not re-read); about 40% of survey respondents say updates "often" or "always" need unnecessary restarts ([Mathur et al., USEC 2018][MAT18], read from a chart, Medium); bad past updates make people skip later ones, including security ones ([Vaniea, Rader, Wash, CHI 2014][WASH14]) |
| **People on Linux** | "Update it with my package manager, like everything else" | Distribution packages replace files in place; Firefox's answer is a restart page that leaves tabs broken (Bugzilla [1750250][BZ1750250], open since January 2022; [2010328][BZ2010328], January 2026) |
| **People installing for the first time** | "Let me download it and run it" | A new Windows signing identity starts with no reputation, and SmartScreen warns until reputation builds ([Microsoft][SSR]); macOS 15 blocks un-notarised software without a System Settings detour ([Apple][SEQ]) |
| **People leaving** | "Remove it, and my data if I say so" | Chrome's "Also delete your browsing data" checkbox exists only on Windows; on macOS the user deletes folders by hand ([Chrome Help][CHU]); Debian policy never removes per-user data ([Debian Policy ch. 10][DEBP]) |
| **Testers on Nightly and Beta** | "Run the preview next to my main browser" | Profiles shared across versions corrupted data until Firefox 67 gave each install its own profile and refused older builds on newer profiles ([Firefox 67 notes][FX67]) |
| **The release maintainer** (one person until SEC-D4 is met) | "Ship a fix today; stop a bad release in minutes" | Rollouts elsewhere widen on vendor-side identity Izcal does not have; CrowdStrike's unstaged content update crashed about 8.5M devices ([Microsoft][MS-CS]); Mozilla's 2019 certificate expiry needed a fix path tied to Telemetry ([Mozilla][ARMA2]) |
| **Regulators and enterprise buyers** (Stage 2) | "Show me the support period, the update mechanism and the advisories" | Izcal plans as a CRA manufacturer (FI-D11); Annex VII asks for "a description of the technical solutions chosen for the secure distribution of updates" ([DUL 1.11][DUL]) |

**What Izcal's specs already decide.** The trust chain is the security scope's: TUF roles with an offline root (UPD-R3), every data channel a TUF target (UPD-R1), fetch in the network service and verify-then-apply in a network-less updater process (UPD-R2), an emergency release class and a closed list of emergency switches (UPD-R4, UPD-R5, SEC-D11), reproducible engine images and attestations (SEC-13, SEC-15), OS signing and key drills (SEC-16), the TUF attack-class suite with a 3-day† timestamp ceiling (SEC-17), and **per-user installs with no elevated component in Stage 1** (SEC-D14). S1 7.8 names Nightly, Beta and Release and requires build attestations with provenance and an SBOM; S1 13.2 lists the package formats and keeps profiles on forward-only migrations kept for at least a year; D-S4 makes usage telemetry opt-in and crash reports opt-out with a per-crash prompt; the shell's 6.11 gives the relaunch flow ("Later" respected for 7 days†) and SH-24 forbids losing UI state across updates. FI-D9 puts per-machine installs in Stage 2 with enterprise; FI-D10 decides Linux packages; FI-D11 plans Izcal as a CRA manufacturer, and the CRA's Art. 13(8) sets the support period's floor of five years. **This spec reopens none of these.** It records one consequence of FI-D10 against SEC-D14 (6.3) and specifies what the decisions leave open: the install layout, the package forms, the signing identity, channels, cadence, rollout, revert, deltas, background behaviour, uninstall and the CRA's product-side duties.

**Success, in terms someone can measure later.** No update path ever produces a `BuildMismatch` exit or a broken tab (DU-1); a bad release can be halted for every running client not yet updated within one poll interval plus signing time, and for closed clients at their next session (DU-4), without any server learning who is who (DU-5); a security release reaches clients inside SEC-1's bounds (DU-23); the first public Stable download and every Stable release after it meet no SmartScreen or Gatekeeper warning (DU-10, DU-11); uninstall leaves nothing the person did not choose to keep beyond a published list of OS-owned records (DU-13); and every CRA update duty in Section 6.11 has a test (DU-15 to DU-17).

## 2. History: what came before and why it was built that way

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 2003 | **bsdiff** | Approximate-match binary diff; most bytes of the difference are zero, so it compresses well | Pointers move in blocks between builds | 58.3× smaller than full on FreeBSD 4.7 security patches ([Percival][BSD]); diffing needs max(17n, 9n+m) bytes of memory ([bsdiff page][BSDP]) |
| 2007–2011 | **Google Update (Omaha)** | One background updater for Google's Windows apps, with channels and proxy support; Omaha 3 added elevation-optional installs | Many products, one channel and network stack | Being replaced by the cross-platform Chromium Updater ([Google][OMAHA3]; [DUH 1.5, 1.9][DUH]) |
| 2009 | **Chrome's per-user install** | Installs without admin rights and updates in the background while running | Vista UAC prompts; non-admins could not install | Kept; it made automatic updates universal ([Chromium Blog][CHR09]) |
| 2009 | **Courgette** | Executable-aware diff: 10,385,920 bytes full, 704,512 bsdiff, 78,848 Courgette | "We want smaller updates because it narrows the window of vulnerability" | Succeeded by Zucchini, which Google's prose describes for PE, ELF and DEX on several ISAs ([Chromium Blog][COUR]; [design doc][COURD]; [DUM 2.3–2.4][DUM]) |
| 2010 | **TUF** | Roles, thresholds and expiring metadata so a single stolen key does not compromise clients | Package managers had single keys and no freeze or rollback defence | Specification 1.0.37 (8 October 2026); used by Sigstore's root and Uptane; PyPI's PEP 458 still "Accepted", not Final ([TUF][TUFS]; [PEP 458][PEP458]) |
| 2012 | **Mozilla Maintenance Service** | Windows service applies updates without UAC | Mozilla chose not to install per-user because it is "an administrative headache" for managed fleets | Still optional; MFSA 2013-83 showed the update file was not locked after its signature was verified ([MFSA 2013-83][MFSA1383]; [DUH 2.3–2.4][DUH]) |
| 2012 | **Flame** | Malware signed with a forged Microsoft certificate, posing as Windows Update | A licensing CA could still sign code with a weak hash | Microsoft moved the CAs to the Untrusted store out of band ([Advisory 2718704][FLAME]) |
| 2013 | **Tor Browser deterministic builds** | Byte-identical builds anyone can reproduce | "I don't believe it is possible to keep software signing keys secure any more" | Spread to Debian and others ([Tor][TORDB]) |
| 2016 | **Sparkle MITM** | Mac apps fetched update feeds over HTTP; release notes ran in WebKit | Small developers, cheap hosting | Now HTTPS plus EdDSA archive signatures; feed signing still optional ([Sparkle][SPARK]; [DUH 4.1–4.2][DUH]) |
| 2016–2019 | **Uptane** | TUF for vehicles: an image repository (offline, what exists) and a director (online, what this vehicle installs) | Cars need a server-chosen target without trusting the server alone | An open standard (IEEE-ISTO 6100 for 1.0, now published by the Joint Development Foundation); the director keeps a vehicle inventory ([Uptane][UPT]; [DUM 7.3][DUM]) |
| 2017 | **NotPetya (M.E.Doc), CCleaner** | An update server taken over; a validly signed trojanised build | Update servers and build environments are root on every client | Signatures proved origin, not integrity ([Talos][MEDOC]; [Talos][CCL]) |
| 2017 | **Mozilla binary transparency** | Log release hashes in CT; the updater refuses unlogged releases | Detect targeted malicious builds | Stalled: meta bug still ASSIGNED, last touched October 2022 ([Mozilla wiki][MBT]; [DUM 7.6][DUM]) |
| 2019 | **"Armagadd-on"** | An add-on signing intermediate expired on 4 May; most add-ons were disabled | The team believed Firefox ignored expiry; QA never ran the browser at future dates | A fix shipped through Normandy within 9 hours and reached most users 6–12 hours later; Normandy was tied to Telemetry, and Mozilla asked for "a mechanism that allows fast updates that isn't coupled to Telemetry and Studies" ([Mozilla][ARMA]; [post-mortem][ARMA2]) |
| 2019 | **Firefox 67 profiles** | One profile per install; older builds refuse newer profiles | A newer storage format damaged older builds' data | Kept; it makes rollback a data problem ([Firefox 67 notes][FX67]) |
| 2019 | **Keystone** | A Chrome updater release on macOS damaged the file system on Macs with SIP off | A root updater ran destructive clean-up | Release paused; victims reinstalled macOS ([TidBITS][KEYS], Medium) |
| 2019–2022 | **Ubuntu snaps** | Chromium deb became a snap stub; Firefox snap-only from 22.04 | One build for all Ubuntu releases | First launch after a reboot was the known cost; Canonical worked on it through 2022 ([Canonical][SNAPPERF]; the size of the later improvement is from secondary reports, [DUH 5.3][DUH], Medium) |
| 2020 | **SUNSPOT (SolarWinds)** | Build-server implant swapped one source file during compilation | Attackers wanted signed, normal-looking builds | The build machine is part of the trusted base ([CrowdStrike][SUNSPOT]) |
| 2021–2022 | **Firefox background update task** | A Windows scheduled task updates every 7 hours while Firefox is closed; exits if Firefox runs | Rarely opened browsers stayed unpatched | Windows only; macOS meta bug idle since July 2022 ([Firefox docs][FXBG]; [DUM 6.1][DUM]) |
| 2021 / 2026 | **Release cadence** | Chrome 4-weekly from Chrome 94 (2021) and weekly security updates from 2023; **Chrome, Edge and Firefox 2-weekly from late August and September 2026**, with Chrome and Edge keeping 8-week Extended Stable and Firefox keeping a yearly ESR | Smaller releases "minimize disruption" | Edge 152 (27 August 2026), Firefox 155 (1 September 2026), Chrome 153 (8 September 2026) ([Chrome][CHR2W]; [Edge][EDGE2W]; [Mozilla][FX2W]; [DUM 3.1–3.6][DUM]) |
| 2023 | **Apple RSR "(a)"** | A rapid security patch appended "(a)" to Safari's UA; sites broke | Patch level leaked into web-visible identity | Pulled the same day ([MacRumors][RSR]) |
| 2023 | **Mozilla APT repository; 3CX** | Vendor-built `.deb` inside APT; a supply-chain attack that caused another | Users keep the package manager, the vendor keeps the build; build hosts reachable from staff machines | Mozilla's model is FI-D10's ([DUH 5.4][DUH]; [Mandiant][3CX]) |
| 2024 | **xz-utils (CVE-2024-3094)** | Release tarballs carried a build script absent from git | Years-long maintainer takeover | Caught by chance before wide uptake ([Freund][XZ]) |
| 2024 | **CrowdStrike channel file 291** | A content update with 21 fields against code supplying 20 crashed about 8.5M Windows devices | "Content" bypassed staged rollout | Rings, canaries, rollback and customer control added ([RCA][CS291]; [Microsoft][MS-CS]) |
| 2025 | **Notepad++; Docker Content Trust** | A hosting compromise sent selected users malicious updates; Notary v1 retired because under 0.05% of pulls used it | The updater trusted the server's manifest; verification was opt-in | Signed manifests; Sigstore and Notation instead ([Notepad++][NPP]; [Docker][DCT]) |

**Technology mismatches: assumptions that no longer hold** ([DUH 10][DUH]).

| Assumption | Who made it | Why it fails for Izcal |
|---|---|---|
| Replacing files is safe until the next start | Early Firefox; distribution packaging habits | Izcal launches children continuously and refuses another build's child; the files a child comes from must never change under its parent |
| An elevated service is an acceptable price for silent updates | Mozilla Maintenance Service, Omaha per-machine, Keystone | A privileged updater is an OS-wide risk (Keystone 2019; MFSA 2013-83); SEC-D14 keeps Stage 1 per-user, and the Linux packages' elevated component is the distribution's package manager, not Izcal's (6.3) |
| HTTPS or a code signature is enough trust | Sparkle before 2016, M.E.Doc, Notepad++ | Server compromise beats transport security, and five cases (SUNSPOT, CCleaner, ShadowHammer, 3CX, xz) shipped validly signed malware |
| A rollout widens on identity the vendor collects | Apple, Google Play, Balrog, CrowdStrike's new rings | Izcal has no install identity and no account in Stage 1; Stage 2 sync (FI-D2) is end-to-end encrypted and gives the rollout no per-client data either |
| One global push is fine for data | CrowdStrike's Rapid Response Content | Data that code interprets can crash clients as surely as code |
| Rollback is free | "Pull the release" | Profile migrations are one-way (Firefox 67); TUF forbids version rollback by design |
| The emergency path can reuse an experiments channel | Normandy in 2019 | Izcal has no experiments channel (FI-D12); UPD-R5 is the emergency path and needs no consent to data collection |
| Patch level is invisible to the web | Apple RSR | Anything web-visible changed by a security release breaks sites |
| The store is the distribution channel | Mobile and console thinking | Both desktop stores are closed to a new engine by policy (Section 3.1) |
| A pre-loaded image keeps children safe | Chrome's zygote, Firefox's fork server | The zygote document itself says a binary's shared libraries cannot be held "unless we are going to write our own ld.so" ([CZ]); Izcal's answer is a build that carries no Izcal-shipped shared libraries and pre-opens its whole file set (6.2) |

## 3. Today: the landscape and its gaps

### 3.1 Per-platform facts that constrain the design

| Area | Fact | Source | Consequence |
|---|---|---|---|
| Windows signing | Code-signing certificates issued from 1 March 2026 are valid for at most **460 days**; keys have been in hardware modules since 1 June 2023 | [CA/B Forum CS BRs][CABCS] | Re-issuance at least every ~15 months; the key never leaves an HSM or a signing service |
| SmartScreen | "EV certificates no longer bypass SmartScreen"; reputation follows the publisher certificate and each file hash; "several weeks and hundreds of clean installs"; a developer has no mechanism to submit a file to speed up consumer reputation; changing the certificate "affects the publisher trust signal" | [Microsoft, SmartScreen reputation][SSR] | Signed public builds under the launch identity months before launch (DU-D10); every release's new file hashes start without file reputation (DU-10) |
| Artifact Signing (formerly Trusted Signing) | Organisations in 11 countries plus the EU; **individuals only in the US or Canada**; validation takes 1 to 20 business days, possibly longer; no EV; certificates valid about three days (stated for the anti-malware profile; Medium in general) | [Microsoft quickstart][ARTS]; [FAQ][ARTSF] | A legal entity in an eligible country, or an OV certificate on an HSM (D-S8) |
| Smart App Control | Signature checks "apply to all executable files"; it blocks unsigned files without positive reputation; DLL coverage is described by a Microsoft Defender engineer on his own blog (Medium) | [Microsoft, SmartScreen][SSR]; [SAC signing][SAC]; [Lawrence][SACL] | Sign every PE file Izcal ships |
| Windows file semantics | `MOVEFILE_DELAY_UNTIL_REBOOT` needs an administrator or LocalSystem; a running image cannot be overwritten or deleted, and whether it can be renamed is practice, not a documented contract (Q-DU-8) | [MoveFileEx][MFE]; [DUP 2.3][DUP] | A per-user install cannot schedule boot-time replacement; versioned directories avoid needing it |
| Microsoft Store | Policy 10.2.1: browsers "must use either the Chromium or the Gecko open-source engine"; 10.2.1 applies to every product, including those listed by URL under 10.2.9 (this spec's reading) | [Store policies 7.20][MSP] | Closed to Izcal; refusal recorded (DU-D9) |
| macOS signing | Developer ID Application and Installer certificates, five of each, created only by the Account Holder; notarisation needs the Hardened Runtime; Gatekeeper accepts a stapled or online ticket | [Apple][ADID]; [Apple, notarizing][NOTAR] | An organisation account; notarise every distributed build, including Nightly |
| Gatekeeper | From macOS 15 "users will no longer be able to Control-click to override Gatekeeper" | [Apple Developer News][SEQ] | Un-notarised builds are effectively unusable |
| macOS bundles | "Bundles should be treated as read-only once they have been signed"; nested code can be *replaced* with code conforming to the designated requirement "without disturbing the outer signature"; adding code to a bundle, and the behaviour of a running process whose bundle changes, are not covered | [Apple TN2206][TN2206] | Never write into a signed bundle; within OI-D3's layout, stage a complete signed bundle and exchange it atomically (DU-D11), measured at DU-M0b |
| Mac App Store | 2.4.5: sandboxed, "other update mechanisms are not allowed", no downloaded code; 2.5.6: "Apps that browse the web must use the appropriate WebKit framework", with an alternative-engine entitlement for iOS and iPadOS in the EU and Japan only | [App Review Guidelines][ARG] | Closed to Izcal; refusal recorded |
| APT signing | Third-party keys "MUST NOT be placed in /etc/apt/trusted.gpg.d" (Debian wiki; the page was not machine-re-read, Medium); `signed-by` points to a file; APT 3.0 verifies with Sequoia `sqv`; Ubuntu 24.04 accepts RSA ≥ 2048 and Ed25519, with a "future" level of RSA ≥ 3072, Ed25519 or Ed448 | [Debian wiki][DEB3P]; [LWN][APT3]; [Ubuntu][UBKEY] | Ed25519 key, binary keyring package, deb822 sources (DU-D8) |
| RPM signing | RHEL 9 distrusts SHA-1 package signatures; RPM 6.0 enforces signature checks upstream; Fedora 43 keeps the legacy behaviour | [Red Hat][RHSHA1]; [Fedora][RPM6] | SHA-256+ signatures, `gpgcheck=1` and `repo_gpgcheck=1` |
| Linux files | `rename(2)` replaces a path atomically and leaves open descriptors unaffected; `execveat(…, AT_EMPTY_PATH)` executes the file an `O_PATH` descriptor refers to (Linux 3.19, glibc 2.34); the dynamic loader opens shared libraries by path, which a held descriptor cannot replace ([CZ]) | [rename(2)][REN]; [execveat(2)][EXA] | Izcal executables link no Izcal-shipped shared library, and the broker pre-opens every file of its build (6.2) |
| Ubuntu namespaces | Since 23.10/24.04 an unprivileged user-namespace sandbox needs an AppArmor profile granting `userns` | [Ubuntu 24.04 notes][UB2404]; [Ubuntu][UBUSERNS] | The `.deb` ships a profile that matches versioned paths (DU-D8) |
| riscv64 | Debian 13 (9 August 2025) is the first Debian with official riscv64, at the RV64GC baseline; Ubuntu 25.10 requires RVA23; Fedora riscv64 is an alternative architecture | [Debian][DEB13]; [Ubuntu 25.10][UB2510]; [DUP 4.13][DUP] | Izcal builds for RVA23 (JS-D22), so its riscv64 packages refuse non-RVA23 CPUs (DU-D16) |
| Windows on Arm | An Arm64 process loads only Arm64 or Arm64X binaries; Arm64EC mixes with emulated x64 code; Chrome supports Windows on Arm only on Windows 11 | [Microsoft][A64EC]; [Chrome][CHRREQ] | Native Arm64 (DU-D16) |
| OS lifetimes | Windows 10 consumer ESU runs to **12 October 2027**; Edge on Windows 10 22H2 to at least October 2028; Firefox ESR 115 serves Windows 7–8.1 into 2027 | [Microsoft ESU][W10ESU]; [Edge][EDGEOS]; [DUL 6.3][DUL] | GR-D21's rule applied at each release decision; a terminal-version record; the Art. 13(10) question (DU-D15) |

### 3.2 How browsers update today

| Practice | State, 24 October 2026 | Source |
|---|---|---|
| Cadence | Chrome, Edge and Firefox ship Stable every two weeks; Chrome and Edge Extended Stable every 8 weeks; Firefox ESR yearly with about 12 weeks of overlap; Chrome security fixes weekly | [DUM 3.2–3.6][DUM] |
| Rollout | Apple phases over 7 days at 1, 2, 5, 10, 20, 50 and 100%; Google Play widens by hand and has no rollback; Balrog throttled background checks (25% for about 24 hours in 2016, "a very fragile process") | [Apple][APPPH]; [Play][PLAY]; [Hearsum][BALROG] |
| Identity in update checks | Chrome's protocol v4 sends an install token once and then deletes it, counts active users by date rather than ID, and lets the server throttle with `X-Retry-After` | [Protocol 4][CHRP4] (prose, OD-7) |
| Deltas | Every scheme falls back to the full payload, and every scheme verifies its base first; Mozilla serves partials from the last four versions; Windows ships forward and reverse differentials against a common baseline | [DUM 2.5–2.8][DUM] |
| Background updates | Firefox: Windows scheduled task, exits if Firefox runs; Chromium updater: periodic scheduled task plus a 4.5-hour check | [DUM 6.1–6.2][DUM] |
| Metered networks | No browser publishes what its updater does on a metered connection; Windows, NetworkManager and Apple all expose the signal | [DUM 6.4–6.8][DUM] |
| Rollback | Fix-forward is the norm; Chrome's managed rollback restores profile snapshots | [DUM 4.7, 4.9][DUM] |
| Transparency | No browser enforces binary transparency; Rekor v2 is GA (10 October 2025); TUF TAP 21 adds ML-DSA signatures | [DUM 7.2, 7.5–7.6][DUM] |

### 3.3 What is invisible today, and what Izcal can expose

- **Which build will the next tab come from?** No browser shows it. Izcal's About page shows the running build, the newest installed build and whether they differ ("an update is installed; this window still runs 0.9.3").
- **Why an update did or did not arrive.** Rollout fractions, halts and holds are server-side state elsewhere. In Izcal they are signed metadata the client evaluates locally, so `about:updates` can say "release 1.4 is rolling out to 20% of installs; yours is not yet included; Check now installs it".
- **What an update changes.** Every release has a signed file inventory, and every security release has an advisory; the update row links both and says whether the release is security-only (6.11).
- **Whether this build is the one everyone else got.** With an inclusion proof shipped beside each release target (DU-D17), the client can say its build is logged, which no browser can claim today.
- **What uninstall will remove.** Izcal records every artefact it creates outside its roots (6.12), so the remove flow lists them before it deletes them.

## 4. First principles

**The users' real tasks.** Get a safe build onto the machine with the fewest decisions; keep it current with no lost work; preview without risk; leave without residue. For the maintainer: publish quickly, stop quickly, prove what was published.

**The platform primitives.** Files and directories with OS-specific replacement semantics (Section 3.1); the OS's code-signing and reputation systems; distribution package managers, their repository signatures and their unattended-upgrade tools; the TUF specification; the OS scheduler; the OS's connection-cost signals; the CRA's text.

**Izcal's principles that bind this design.** P1 (security by construction) and M-1 (build our own, from the specifications); FI-D14 (open protocols first; proprietary only where no open route works); D-S4 and FI-D12 (usage telemetry opt-in; crash reports opt-out with a per-crash prompt); SEC-D14 (no elevated Izcal component in Stage 1); the IPC's one-build rule (VER-1, `BuildId`); FP 6.8 ("hidden means no wakeups"); AG-0 and SH-8 (AI stays off across updates).

**Assumptions kept and broken.**

| Assumption | Keep or break | Why |
|---|---|---|
| Updates are automatic and silent by default | **Keep** | The strongest evidence in the field and a CRA requirement |
| Install per user, without elevation | **Keep** for Windows, macOS and the tarball in Stage 1 | SEC-D14; Keystone; Linux packages are per-machine by FI-D10, with the package manager as the elevated component (6.3); Izcal's own per-machine installs arrive with their own threat model in Stage 2 (FI-D9) |
| Files can be replaced in place and the browser restarted later | **Break** | The `BuildId` rule makes in-place replacement a correctness bug; versioned directories everywhere (DU-D1) |
| A zygote or fork server protects new children | **Break** for Izcal | S1 Section 1's Linux spawner is deliberately "not a fork server" and holds no pre-initialised engine state (FP-D1 chose fresh `exec`); Windows has no fork; the zygote cannot hold shared libraries ([CZ]) |
| Staged rollout needs a server that knows each client | **Break** | The ramp is signed metadata and the position is local and fresh per release (DU-D5) |
| Rollback means downgrading | **Break** | Fix forward; a revert is the previous source rebuilt under a higher version, allowed only through the profile rules of DU-D6 |
| Deltas need an executable-aware disassembler | **Break, provisionally** | Start with a dictionary-style format; add an ISA-aware stage only if measurement justifies its parser size (DU-D7) |
| Download pages tag installers for attribution | **Break** | A new data flow to the vendor (FI-D12) and an obstacle to reproducible-build checks; one byte-identical installer per platform (DU-D13) |
| Each store is a distribution channel worth targeting | **Break** for Stages 1–2 | The Microsoft Store and the Mac App Store refuse non-default engines by policy (DU-D9, DU-D11); Snap is declined on reach and trust grounds (DU-D8) |
| Uninstall is the OS's job | **Break** | The CRA makes permanent data removal a product requirement, and no OS uninstaller knows Izcal's keychain items or scheduled tasks (DU-D12) |
| A distribution package uses the distribution's update path | **Keep** | FI-D10; Izcal configures that path for automatic security updates rather than replacing it (6.3) |

## 5. Lessons learned

Ideas and evidence only; no code and no template (M-1).

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Chrome's 2009 installer | Per-user, silent, background, small updates by design ([CHR09]) | Removing the admin prompt made automatic updates universal | Stage 1 per-user installs (SEC-D14); DU-D9 |
| Chrome zygote; Firefox restart page | Children from a changed file mismatch the parent ([CZ]; [BZ1750250]) | Never launch a child from a path that can change | DU-D1; DU-1 |
| Chrome zygote's own limit; Firefox fork server on FreeBSD | Shared libraries cannot be held by a pre-loaded image ([CZ]); a fork server still failed after a package update, probably because the child looked something up by path (the reporter's suspicion, [Bugzilla 1888601][BZ1888601]) | Pre-opening is only as good as the list of what is pre-opened | No Izcal-shipped shared libraries; the broker pre-opens every file in its build's inventory (6.2) |
| MFSA 2013-83 | The update file was not locked after its signature was verified | Verify the bytes you apply | Verify-then-apply into a fresh build directory that only the updater writes (UPD-R2; DU-D1) |
| Courgette, Mozilla partials, Windows differentials | Big savings, always a full fallback, always a verified base ([DUM 2][DUM]) | Deltas are an optimisation that must never be the only path | DU-D7; DU-6, DU-7 |
| Keystone 2019 | A root updater damaged Macs ([KEYS]) | An updater bug is an OS bug | No elevated Izcal component in Stage 1; Linux maintainer scripts do only declarative work (6.3) |
| Armagadd-on 2019 | An expired intermediate disabled most add-ons; QA never tested future dates ([ARMA2]) | Run the browser at future dates; keep an emergency path independent of data consent | DU-19; UPD-R5 needs no telemetry |
| Firefox 67 | Older builds corrupted newer profiles ([FX67]) | Each install its own profile; refuse newer profiles | DU-D3; DU-14 |
| Apple RSR "(a)" | A security patch changed the UA ([RSR]) | Security releases change no web-visible identity | DU-18 |
| CrowdStrike 291 | An unstaged content update crashed about 8.5M devices ([CS291]; [MS-CS]) | Stage every channel whose data code interprets in ways that can crash it | Ramps are a per-channel property (DU-D5) |
| Balrog throttling | Rollout by hand-edited server rules was "a very fragile process" ([BALROG]) | A ramp should be a schedule, not a person editing a rule | Ramp schedules in signed metadata (DU-D5) |
| Uptane | A director chooses among images the offline repository signed ([UPT]) | Separate "what exists" from "what to install now" | The `rollout` role selects only among `releases`-signed targets and holds no inventory (DU-D5) |
| SUNSPOT, 3CX, xz | Build or source tampering with validly signed output | The build is part of the trusted base | SEC-15 reproducibility, extended by stages to distributed packages (6.16) |
| CCleaner, ShadowHammer, Notepad++ | Signed malware; a few users targeted | Signatures prove origin, not intent; targeting hides in mass updates | Binary transparency with an inclusion proof per release target (DU-D17) |
| Docker Content Trust | Opt-in verification, under 0.05% use, retired ([DCT]) | Optional verification goes unused | TUF verification has no off switch; the CRA opt-out disables automatic installation of builds, never verification |
| Mozilla APT repository; Iceweasel | Vendor repository inside the package manager; a ten-year trademark rename ([DUH 5][DUH]) | Plan for distributions | DU-D8 repositories; DU-D18 published policy for distribution and third-party packaging |
| Ubuntu Firefox snap | First launch after reboot was the known cost ([SNAPPERF]) | Packaging has a measurable cost | DU-20 cold-start parity per format |
| SmartScreen and Gatekeeper | Reputation takes weeks; Gatekeeper override removed ([SSR]; [SEQ]) | Identity is on the critical path | DU-D10's lead time; DU-10, DU-11 |
| Update UX studies | Restart cost and unclear value drive avoidance ([MAT18]; [WASH14]) | Make the restart free and the value legible | SH-24; the update row names security versus features (6.8) |
| Storm-0558 | Key rotation stopped once it became painful ([CSRB]) | Rotation must be rehearsed and cheap | SEC-16 drills kept; this spec adds a key inventory (6.10) |

## 6. Design

### 6.1 Architecture at a glance

```
 build farm ──(build inputs → BuildId; reproducible build; attestation; independent rebuild)──► release artefacts
      │                                                                                │
      ├─► TUF repository (root offline; releases; rollout ⊂ releases; snapshot; timestamp) ◄── transparency log
      │        │ build targets (full + deltas), data-channel targets, terminal-version records
      │        ▼
      │   network system service ──chunks (shared memory)──► broker ──► updater (hg-update): verify, write staging/, publish
      │                                                                     ▲
      │   OS scheduler (task / LaunchAgent via SMAppService / systemd user timer) ── headless update session when closed
      │
      └─► Izcal APT/YUM repositories ── distribution package manager + its unattended-upgrade tool ──► /opt/izcal/<channel>/builds/<build>/
                                       (Izcal's updater off for the browser binary; verifies data channels into the per-user data root)

 install root:  launcher ──shared lock on live/<build>.lock──► builds/<version>-<buildid>/
                broker (pre-opens every file in its inventory) ──spawns only Izcal kinds, from its own build──► children
```

**New crates** (all `hg-*`, all Izcal's own, M-1):

| Crate | Layer | Ships? | What it does |
|---|---|---|---|
| `hg-install` | L5 services | Yes | Install policy only: layout, the `current` pointer, lock protocol, reaper rules, the per-user install ledger (6.12). All file, lock, process and OS-registration effects go through the substrate (G3) |
| `hg-delta` | L2s (formats) | Yes | The delta format and its applier (6.7), a pure format crate like `hg-cb-format`; linked only into the updater (G252) |
| `hg-launch-main` | L6 apps | Yes | The stable launcher: takes the build lock, starts the broker from that build, hands a second launch to a running broker through OS integration's `hg-launch-wire` message, following that spec's launcher algorithm (OI 6.2); the crate and its algorithm are OI-D2/OI-D3's, and this spec adds the build lock and the `current` read; links only `hg-substrate-os`'s launch entry, `hg-install`, `hg-launch-wire`, `hg-xdg` and `hg-os-journal` (G255, the same set as OI G231) |
| `hg-updater-main` | L6 apps | Yes | The updater process of S1 Section 1 (no binary for it exists in S1 2.1 yet), linking `hg-update`, `hg-install` and `hg-delta` |
| `hg-rollout` | Tools (server) | No | Writes ramp files for the `rollout` role; checks that they select only `releases`-signed targets and respect the client's bounds (6.9) |
| `hg-pkg-build` | Tools | No | Builds the `.deb`, `.rpm`, tarball and Windows installer from one signed build inventory, reproducibly; drives the OS tools for the `.dmg` (`hdiutil`) and, in Stage 2, the `.msi` (the Windows Installer database API) and `.pkg`, under H2 2.3.2's toolchain and OS-interface exception, because Apple publishes no UDIF specification and the MSI database is documented only through its API |
| `hg-repo-sign` | Tools | No | Izcal's OpenPGP signer for repository metadata and RPMs, written from [RFC 9580][RFC9580], signing through an HSM's PKCS#11 interface; `hg-crypto` gains Ed25519 signing for tools (it has verification only today); output verified in CI by the distributions' own verifiers as black-box oracles |
| `hg-advisory` | Tools | No | Generates the human and machine-readable advisory for each security release (6.11) |

**Substrate additions** (`hg-substrate-os`, with models in `hg-substrate-sim`): an **install-device** trait (atomic rename, shared and exclusive file locks, delete-tree, read-only open, open-by-inventory) and an **OS-registration** module per OS (scheduler entries, the Uninstall key, LaunchAgents through `SMAppService`, systemd user units, AppArmor references). G3 and CS-ST-18 then hold: `hg-install` decides, the substrate acts, and DST can replace a file on disk or a whole build between two launches. Q-CS-1's list of binaries that link only `hg-substrate-os` gains `hg-launch-main` and `hg-updater-main`.

**Crate rules G250–G255** (this spec's registry block is G250–G259).
- **G250.** Only `hg-launch-main` and `hg-install` read the `current` pointer; every other process resolves paths from its own build, given to it at launch.
- **G251.** **Every Izcal child process kind** is launched from a **build-relative binary handle** obtained by the broker at start-up; the substrate has no API to launch an Izcal process kind by name or through `PATH`. Launching programs Izcal does not ship (external protocol handlers, download "Open", native-messaging hosts, the OS default-apps surface, the protected-media host's OS engine) uses a separate broker-only API that the graph lint confines to the broker.
- **G252.** `hg-delta` and every update-metadata parser are linked only into `hg-updater-main`; the broker parses no update payload (extends UPD-R2).
- **G253.** Only `hg-install`, linked by the launcher and the updater, creates or deletes entries in `builds/` and the data root; nothing modifies a published entry.
- **G254.** No update-related request carries an install identifier, ramp position, profile identifier or any per-install value; T1 tests enforce it for bytes and for timing (DU-5).
- **G255.** `hg-launch-main` links only `hg-substrate-os`'s launch entry, `hg-install`, `hg-launch-wire`, `hg-xdg` and `hg-os-journal` (the same set as OS integration's G231; the last two read the Linux desktop entries and the registration journal), so it stays small and changes rarely (OI 6.2: the launcher links no engine crate).

### 6.2 The install layout and running-build safety (DU-D1)

**One layout, every OS, every format.**

```
<root>/                               per install, per channel
  izcal | izcal.exe | Izcal.app        the launcher (small, changes rarely, signed)
  current                              the release version and BuildId of the newest complete, verified build; written by atomic rename
  builds/<version>-<buildid>/          one complete build; read-only once published
      build.inventory                  signed file inventory: every file's path and hash; its header carries the BuildId
  staging/                             the updater's work area and resumable chunk ledger
<state>/                              per user, per channel, always user-writable
  live/<build>.lock                    one lock file per build
  registrations.izj                    the per-install registration journal (OI-D10), which is also the install ledger (6.12)
  rollout.secret                       the local secret for ramp positions (6.9)
<data>/                               per user, per channel: verified data-channel targets (6.3)
```

| Format | `<root>` | `<state>` and `<data>` | Who writes `builds/` |
|---|---|---|---|
| Windows per-user (Stage 1) | `%LocalAppData%\Izcal\<channel>\` (OI 6.2's `<install>`; the build directory `builds\<version>-<buildid>\` is its `<install>\<version>\`) | `%LocalAppData%\Izcal\<channel>\state\`, `…\data\` | Izcal's updater |
| Windows per-machine MSI (Stage 2) | `%ProgramFiles%\Izcal\<channel>\` | Per user, as above | The Stage 2 elevated updater, under its own threat model (FI-D9, SEC-D14's trigger) |
| macOS (Stage 1) | `Izcal.app`, whose bundle executable is the broker (OI-D3); each build's children live in a versioned helper framework inside it, `Contents/Frameworks/Izcal <version>.framework/Helpers/`; the command-line launcher is `Contents/MacOS/izcal-cli` (6.5, DU-D11) | `~/Library/Application Support/Izcal/<channel>/state/`, `…/data/` | Izcal's updater, by staging a complete signed bundle and exchanging it atomically (6.5) |
| Linux `.deb`/`.rpm` (Stage 1) | `/opt/izcal/<channel>/`, with the stable launcher at `/usr/bin/izcal` (Stable) and `/usr/bin/izcal-beta`, `/usr/bin/izcal-nightly` (OI 6.2) | `$XDG_RUNTIME_DIR/izcal-<channel>/` for locks (OI 6.2's per-channel runtime directory, which also holds the hand-off socket); `$XDG_STATE_HOME/izcal/<channel>/` and `$XDG_DATA_HOME/izcal/<channel>/data/` | **The system package manager** (FI-D10): each build is its own package (6.3) |
| Linux tarball (Stage 1) | The directory the user unpacked into | As for packages | Izcal's updater |
| Flatpak (Stage 2) | The Flatpak deployment | Flatpak's per-app directories | Flatpak (whether it keeps a running deployment's files is Q-DU-11) |

**The rules.**
1. **A build directory is never modified after it is published.** The updater verifies a new build into `staging/`, renames it into `builds/`, then renames a new `current` into place. A partial or unverified build is never selectable. Repair stages a fresh, verified copy in `staging/` and publishes it under the same rule.
2. **The launcher takes a shared lock on `live/<build>.lock` for the build `current` names before it releases its read of `current`,** starts the broker from that build and passes the lock handle to the broker (the one thing a child inherits). On macOS, where Launch Services starts the broker (OI-D3), the broker takes the shared lock on its own build itself at start, before it pins its helper framework. After that the broker never reads `current`.
3. **The broker resolves its own build once, at start-up, from its own running image** (the image path the OS reports for the process, opened as a handle), never from `argv[0]`, `PATH` or `current`, and **opens every file listed in its `build.inventory`** (a few hundred descriptors): executables as launch handles, engine images, the shell image, data sets and resources as read-only handles. Every later need for a build file is served from those handles, never by path; sandboxed children already receive everything as pre-opened handles (SEC 6.7). On Linux the spawner, started from the same build, launches with `execveat(…, AT_EMPTY_PATH)` on its held descriptors. On Windows the broker holds each child executable open with `FILE_SHARE_READ` only, so no other process can rename, replace or delete it while the build runs. **Izcal's executables link no shared library that Izcal ships** (Rust links statically; only OS-provided libraries are loaded), so the loader never resolves an Izcal file by path; the T1 build check fails a build whose executables import an Izcal-shipped shared object.
4. **The reaper** (in `hg-install`, run by the launcher at start and by the updater after a publish) deletes a non-current build only after taking an **exclusive, non-blocking** lock on its `live/<build>.lock`, which succeeds only when no process of that build holds the shared lock. It keeps at most the current build plus one older build†, so installed size is at most twice FP-25's per-architecture figure during a transition. On Linux packages the reaper does not apply: the package manager owns `builds/`, and the locks only inform the About page.
5. **The launcher itself** is a TUF target (or, for packages, part of the metapackage). It is replaced by an atomic rename-over (Windows, Linux tarball) only when no launcher process holds `<state>/launcher.lock`, otherwise the replacement waits for the next updater run; on macOS there is no separate launcher (the bundle executable is the broker, and `izcal-cli` travels inside the bundle), so it is replaced by the bundle exchange of 6.5.
6. **The `BuildId` check stays strict.** If it ever fires (a file deleted by a third party, a broken package), the broker does not retry: it enters a **restart-required state**, the shell shows the update row with "Restart to finish updating" (shell 6.11), and the failed launch is recorded as a defect. DU-1 requires this never to happen on a supported update path.

**Why this and not the alternatives.**

| Option | What it costs | Verdict |
|---|---|---|
| (a) **Versioned directories everywhere**, own-build launch, external liveness locks, full pre-open, no Izcal-shipped shared libraries | About twice the install size during a transition; a reaper; a few hundred descriptors in the broker; packaging per build on Linux | **Recommended** |
| (b) A zygote or fork server holding the old image | Contradicts S1 Section 1's "not a fork server, no pre-initialised engine state" (FP-D1); Windows has no fork; cannot hold shared libraries ([CZ]) | Rejected |
| (c) Detect and force a relaunch (Firefox's page) | Broken tabs until the person restarts; the evidence of 2022–2026 is that this is the pain point | Kept only as rule 6's fallback |
| (d) Hold every binary open and forbid replacement | Windows does it for running images, and rule 3 uses it there; Linux and macOS do not stop a package manager or another process | Part of (a), not a design on its own |

**Reversal trigger:** a DU-M0b measurement shows that on some OS a running build cannot keep launching from its own build after a newer build is installed beside it (for example, AppArmor does not attach to `execveat` of a removed file, or macOS revokes a running bundle's validity). That OS then uses the stub of 6.3 or falls back to (c) with a prompt at the next safe moment, and the owner is told.

### 6.3 Linux: repositories, packages and the tarball (DU-D8, FI-D10)

**Packages.** For each channel a small **metapackage** `izcal-<channel>` ships the launcher, the desktop file, the AppArmor profile, the repository keyring, the unattended-upgrade configuration below and a tiny **exec stub** at a stable path, and **depends on exactly one build package** `izcal-<channel>-<version>`, which installs only into `/opt/izcal/<channel>/builds/<version>-<buildid>/`. An upgrade installs the new build package beside the old one; the old one becomes an automatic-removal candidate once nothing depends on it, as distributions handle versioned kernel packages. So **the package manager adds a build instead of replacing one under a running browser**, which is FI-D10's requirement. The metapackage's maintainer scripts do **only declarative work** (write `current`, register the AppArmor profile); they never signal or stop a running process. A running broker learns of a new build by watching `current` (inotify: event-driven, no periodic wake-up) and shows the update row. If autoremove deletes an old build while its browser still runs, rule 3's pre-opened handles keep it working; whether the AppArmor `userns` profile still attaches to an `execveat` of a deleted file is measured at DU-M0b (Q-DU-13), and if it does not, the spawner execs the metapackage's stub (stable path, profile-attached), which `fexecve`s the held descriptor.

**Automatic security updates by default (CRA).** Distributions' unattended-upgrade tools cover only their own origins unless configured, and `apt-get upgrade` keeps back a package whose upgrade needs a new package. So:
- the `.deb` ships an `apt.conf.d` snippet that adds Izcal's repository origin to `Unattended-Upgrade::Origins-Pattern`, and Recommends `unattended-upgrades`;
- the `.rpm` ships a `dnf-automatic` configuration limited to Izcal's repository and enables its timer preset;
- where neither tool is present or enabled, the metapackage installs **Izcal's own system timer** (a systemd system unit running the package manager for Izcal's packages only, with `--with-new-pkgs` semantics, so the metapackage's new dependency is installed);
- the running broker compares the build it runs with the newest build in TUF `releases` metadata, which it already verifies for data channels, and shows the update row when the package manager has not installed it;
- Settings > Updates shows the CRA switch on packages too; turning it off disables Izcal's snippet and timer, and the About page says that the system administrator's package-manager settings also apply.

Whether a distribution-mediated install counts as "automatic" under Annex I Part I (2)(c) is FI-1's legal check. DU-1's matrix includes `apt-get upgrade`, unattended-upgrades and `dnf-automatic`, and DU-15 has package rows.

**Per-machine install, recorded.** dpkg and rpm run maintainer scripts as root, so the Linux packages are **per-machine installs whose elevated component is the distribution's package manager**, not an Izcal service. FI-D10 decided this; Section 12 proposes the matching amendment to SEC-D14. The repository key is therefore root code execution on every Linux client, and gets the matching protections: SEC-16 custody and drills, a second offline key shipped in the keyring package for rotation, a short `Valid-Until`† (7 days†) in the APT release file for freeze protection, and a release-pipeline check that each package's payload is byte-identical to the build's `build.inventory`, whose hash is in the transparency log, so DU-D17 covers packages through the inventory even though APT cannot check inclusion proofs.

**Repositories.** Izcal's own APT and YUM repositories, signed by `hg-repo-sign`:
- **APT**: deb822 `.sources` with `Signed-By:` pointing to `/usr/share/keyrings/izcal-archive-keyring.pgp`, a binary key shipped by an `izcal-archive-keyring` package; no `apt-key`, no `trusted.gpg.d`, no pin above priority 100 ([Debian][DEB3P]); `InRelease` signed.
- **YUM/DNF**: `gpgcheck=1` and `repo_gpgcheck=1`; every RPM signed.
- **Keys**: an **Ed25519** OpenPGP key with SHA-256 or stronger binding signatures and no SHA-1 anywhere, which passes APT 3.0's `sqv` and Ubuntu's "future" level ([UBKEY]); separate from every TUF and OS-signing key, held on hardware keys with the same custody as the `releases` role (UPD-R3). CI verifies every published repository with `sqv` and `rpm` as black-box oracles.
- **Archive**: security releases stay in the pools for 10 years or the rest of the support period, whichever is longer (Art. 13(9)); Nightly packages are pruned.

**What Izcal's updater does for packages.** It is **off for the browser binary** (FI-D10). It remains the verifier of every UPD-R1 data channel, writing into the per-user data root (6.2, `<data>`), never into `/opt`. The About page says which updater owns the binary ("Updated by your system's package manager").

**Rollout for packages.** A package repository has no per-client ramp Izcal can rely on: a feature release reaches the Stable repository only after its Beta bake (6.9), and a halt removes it from the index before most clients fetch it. Whether a third-party APT repository can use APT's phased-update field is Q-DU-4.

**AppArmor.** The `.deb` ships a profile for Ubuntu 23.10 and later granting `userns` to Izcal's executables and the stub, written for the versioned paths (`/opt/izcal/*/builds/*/…`), so a new build needs no profile change. The tarball cannot install a profile: on a system that restricts unprivileged user namespaces it runs at SEC 6.7's reduced tier, shown on `izcal://sandbox`, and is **a non-reference configuration under SEC-8**; its first-run screen offers a one-step profile installer the person runs with `sudo`.

**Tarball.** The same layout under a directory the user chose, with Izcal's TUF updater and a systemd user timer for updates while closed (6.8).

**riscv64.** Izcal builds for RVA23 (JS-D22), while Debian 13's riscv64 baseline is RV64GC. The `.deb`'s `preinst` (and the launcher, for the tarball) checks the CPU's ISA string for the RVA23 profile and refuses with a clear message; the download page says "RVA23 hardware only".

**Flatpak (Stage 2)** keeps S1 13.2's decision: built from the same inventory, published on Flathub as a verified publisher, with the reduced isolation documented. **Snap and AppImage are not offered in Stages 1–2** (DU-D8): the `.deb` reaches the same Ubuntu users, Snap's store controls updates outside Izcal's TUF chain, and AppImage has neither an update path nor a sandbox story.

**Distribution-built and third-party packaging (DU-D18).** A distribution may build Izcal from source: the updater for the binary is compiled out, the build identifies itself as distribution-built in `about:version`, data channels still verify through TUF, and trademark use follows a published policy (FI-1). Third-party manifests (winget, Homebrew cask, community AUR packages) install the signed installer unchanged and declare that the app updates itself. The Iceweasel decade is the reason to publish this before anyone asks.

### 6.4 Windows (DU-D9)

**Stage 1: one per-user installer**, `IzcalSetup-<channel>-<arch>.exe`, byte-identical for every downloader (no stub, no tag), signed, silent-capable (`/silent`), writing to `%LocalAppData%\Izcal\<channel>\` without elevation. It registers an **Uninstall entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\`** with the values Microsoft documents for the Uninstall key (DisplayName, DisplayVersion, Publisher, InstallLocation, EstimatedSize, UninstallString, ModifyPath for repair, HelpLink, URLInfoAbout; the HKCU location is standard practice, [Microsoft][UNKEY], Medium for HKCU); a Start menu entry; a per-channel AUMID; and nothing else without asking. It does not pin to the taskbar (Microsoft: "Don't use installers to call the API", [Microsoft][PIN]) and asks before a desktop shortcut. The default-browser ask stays the shell's (SH-D21).

**Repair.** "Modify" in Installed apps runs the installer in repair mode: it stages a fresh copy of the current build verified against its `build.inventory`, publishes it under 6.2 rule 1, and re-registers the launcher and the scheduled task. Repair never touches the profile.

**Every PE file is signed**: the launcher, the broker, every child executable and every DLL Izcal ships, because Smart App Control's signature checks apply to all executable files ([SSR]; [SACL], Medium for DLLs).

**Stage 2: per-machine MSI** for silent enterprise deployment (`msiexec /qn`), installing to `%ProgramFiles%`, with an elevated update service decided under SEC-D14's reversal trigger with its own threat model (FI-D9, FI-3). **MSIX** is considered only if a measurement shows that a full-trust packaged process can create Izcal's own AppContainer and restricted-token children with its sandbox profile (Q-DU-6); it is never a Store submission, because **the Microsoft Store is closed** to a non-Chromium, non-Gecko engine (policy 10.2.1).

**Arm64**: a native Arm64 build, not Arm64EC; Izcal hosts no x64 plug-ins.

### 6.5 macOS (DU-D11)

**Stage 1: a signed, notarised and stapled `.dmg`** containing the `Izcal.app` launcher (Nightly, Beta and Stable as separate launchers with separate bundle IDs). **Every build that reaches a person is notarised**, Nightly included, because macOS 15 removed the Control-click override ([SEQ]); the notary round trip is a release-pipeline step whose latency is measured (Q-DU-7). The Hardened Runtime carries `allow-jit` only on executables that run the JIT; the jitless helper lacks it (SEC 6.7).

**Layout (OI-D3, a requirement from the OS integration spec).** `Izcal.app`'s `CFBundleExecutable` **is the broker**, because Launch Services, Dock identity, notification authorisation and keychain groups attach to that process (OI 6.2). Each build's child executables (renderers, the jitless helper, services) live in a **versioned helper framework inside the bundle**, `Contents/Frameworks/Izcal <version>.framework/Helpers/`, which the broker pins at start by holding a directory descriptor; helper bundles declare no URL schemes or document types, so Launch Services never routes an open to them. The command-line launcher is `Contents/MacOS/izcal-cli`, linked onto the user's path by the installer; a cold command-line launch goes **through Launch Services**, never `exec`, so the broker always starts with its bundle identity (OI 6.2).

**Update mechanism (this spec's).** Nothing writes into a signed bundle in place ([TN2206]). The updater **stages a complete, signed and notarised `Izcal.app` for the new build** beside the installed one, containing the new build's helper framework **and the current running build's helper framework** (so the running broker's helpers stay inside a validly sealed bundle; the release pipeline signs each published bundle in that two-framework form, with the previous build's framework taken from its own signed release), and then **exchanges the two bundles atomically** with `renamex_np(…, RENAME_SWAP)`. The old bundle moves to `<state>/retired/` and is deleted after its broker exits (6.2 rule 4). The running broker keeps its mapped image and reaches its helpers through the held directory descriptor, resolving the current path of that descriptor (`fcntl(F_GETPATH)`) before each `posix_spawn`, since macOS has no execute-by-descriptor; the next launch starts the new build's broker from the swapped bundle. **The alternative, in-place update** (add the new helper framework into the installed bundle and replace its executable, which is what OI 6.2's update column describes), changes a sealed bundle in several steps and is adopted only if DU-M0b shows the bundle verifies at every step; the exchange keeps every on-disk state a state the release pipeline signed, and the OI spec's requirement (the broker is the bundle executable; children from a versioned helper bundle it pinned) holds either way.

DU-M0b checks (OI-M1 validates the same layout with a real update under a running broker, OI 9.5 risk row): a running broker keeps launching helpers after the exchange and after its old bundle moves to `retired/`; Gatekeeper and notarisation status of each published bundle form; TCC grants (camera, microphone) and Keychain items bound to Izcal's code signature (SEC-34) survive an update without a prompt; Launch Services' view of the bundle after the exchange; App Translocation of a quarantined `Izcal.app` (first run offers to move it to `/Applications`), and a standard user's updater when an administrator installed the app in `/Applications` (the updater then asks the person to authenticate the exchange, or falls back to notifying, a per-machine case that Stage 2's `.pkg` and FI-3 cover).

**Updates while closed** use a LaunchAgent registered with `SMAppService` from inside `Izcal.app` (macOS 13+), which starts the bundle's broker with `--update-only`, which disappears with the app and shows the system's "background item added" notice, which the user information explains (6.12).

**Stage 2: a Developer ID Installer–signed `.pkg`** for MDM, re-signed whenever the Installer certificate expires ([ADID]). **The Mac App Store is refused** (2.4.5, 2.5.6).

**Identity.** An Apple Developer Program **organisation** account (D-U-N-S number), so the publisher reads "Izcal" rather than a person's name; because only the Account Holder can create Developer ID certificates, SEC-D4's second person becomes an Admin with the Developer ID role before the first public binary.

### 6.6 Channels, cadence and side-by-side installs (DU-D3, DU-D4)

| Channel | Built from | Cadence | Audience | Stage |
|---|---|---|---|---|
| **Nightly** | `main`, every night | Daily | Contributors and testers; ring 0 | 1 |
| **Beta** | The next Stable's branch | Weekly† | Testers; ring 1 | 1 |
| **Stable** | Release branch | **Feature release every 4 weeks†**; **security-only point releases between them** whenever SEC-1's bounds require | Everyone | 1 |
| **Extended** | Every sixth Stable† (about 24 weeks) | Security-only point releases for two cycles† with 8 weeks† of overlap | Managed fleets | 2 (with FI-3) |

**Nightly and Beta under the CRA.** CRA Art. 4(3) lets Member States allow "unfinished software" for "a limited period required for testing purposes with a visible sign clearly indicating that it does not comply with this Regulation", and recital 37 adds that such software should follow a risk assessment, meet the security requirements "to the extent possible", and that manufacturers "should not force users to upgrade to versions only released for testing purposes" ([CRA][CRA]). So Nightly and Beta carry a visible preview notice in the shell and on the download page, a stated testing period, the same updater and signing as Stable, and Stable never moves anyone to them. Whether public Nightlies kept for SmartScreen reputation (6.10) are "testing purposes" is FI-1's question, before DU-M1.

**Why 4 weeks and not 2.** Chrome, Edge and Firefox moved to two weeks in August and September 2026 ([DUM 3.3–3.5][DUM]). Izcal's cadence is a capacity question for a team that starts with one release maintainer (SEC-D4), and the CRA requires security fixes that do not depend on feature releases (Annex I Part II (2)): security-only point releases on the current Stable meet that duty at any feature cadence. The reversal trigger is measurement: when two consecutive quarters† of releases show the full pipeline (build, independent rebuild, signing, notarisation, bake) at under 3 working days†, Stable moves to 2 weeks.

**What "security-only" means** (DU-18): a point release that changes only what fixes the advisory's issues; it changes no web-visible identity (UA string, client hints, exposed features), no UI and no default, which is the Apple RSR lesson and recital 57's reason.

**Side by side.** Each channel is its own **installation identity**: install root, launcher, Windows AUMID and Uninstall key, macOS bundle ID, Linux desktop-file ID with a matching `StartupWMClass` ([freedesktop][DESKTOP]), icon tint, default-browser registration, state and data roots, and **profile root**. Channels never share a profile. Moving between channels is a new install plus an **import** through the shell's import preview (shell 6.12); the shell's "Update channel" setting becomes a read-only "Channel" row with a "Get Beta or Nightly…" link (Section 12).

**Application identifiers (answers Q-OI-12).** One scheme for every channel, with the reverse-DNS prefix fixed by the domain that D-S9's trademark clearance secures: application and bundle ID `<prefix>.Izcal` for Stable, `<prefix>.Izcal.Beta` and `<prefix>.Izcal.Nightly`; helper and build bundles `<prefix>.Izcal[.<Channel>].Helper.<kind>`; Linux desktop-file IDs `<prefix>.Izcal[.<Channel>].desktop` with a matching `StartupWMClass`; Windows AUMIDs `Izcal.Browser.<channel>.<profile>` as OI 6.6 defines them. The identifiers are fixed before OI-M1 and never change afterwards, because TCC, Keychain, notification authorisation and default-browser registrations key on them.

**Profile stamps.** Every profile records the newest schema that wrote it. A build opens a profile stamped by **at most one migration ahead of its own schema** by restoring that profile's retained pre-migration root (6.9, revert); a profile further ahead is refused with an offer to import ("This profile was used by Izcal Beta 1.6. Import its bookmarks and history into a new profile?"), never opened silently (DU-14).

### 6.7 Delta updates (DU-D7)

- A **delta is an optional TUF target** keyed by (from-build, to-build). Its metadata names the full target it must reproduce. The full target for every build is always published and signed.
- **Verify, then parse.** The updater checks the delta's length and hash against `releases`-signed metadata before parsing a byte, so only a `releases` key holder can reach the parser.
- `hg-update` applies a delta into `staging/` using the **current build as the read-only, verified base**, then **verifies the result file by file against the target build's `build.inventory`** before publishing it. Any mismatch or error means: fetch the full target. Because builds are never modified after publication (6.2), the "modified install" failure Mozilla's partials meet cannot occur ([DUM 2.6][DUM]).
- **Format: Izcal's own, dictionary-style** (the old file as the dictionary for a long-range matcher, the shape zstd's `--patch-from` demonstrated: on source tarballs, patch sizes comparable to bsdiff at about seven times its speed, [zstd 1.4.5 notes][ZSTD]), written from first principles under M-1, with a fuzz target in SEC-17's suite. **An executable-aware stage** (Courgette-class) is added only if DU-M2 measures that it saves more than 2×† over the dictionary format on Izcal's own release pairs, weighed against its parser size and auditability (a disassembler per ISA and object format).
- **Pairs**: deltas from the last three† Stable builds, from the previous build on Beta and Nightly, and from the last feature release to each security point release.
- **Packages** (APT, YUM) carry no deltas.

### 6.8 Background updates, checks and relaunch (DU-D2)

**While Izcal runs**, update checks ride the poll the update client already makes: the phishing bundle's 15-minute† manifest poll (PH-D12) fetches the timestamp and snapshot metadata, so learning about a browser release adds no wake-up of its own. (NW 6.17 says update checks run daily and are never polled; PH-D12 already put a 15-minute poll on the shared repository, and Section 12 records the NW amendment, keeping NW's wake-grid and battery-saver deferral.) When a release applies to this install (6.9), the network system service fetches it in chunks, the broker passes each chunk to the updater as a shared-memory handle (IPC-R16 entry 3's size cap applies per chunk), and the updater writes it into `staging/` with a resumable chunk ledger, then verifies and publishes. **Staging while running is safe** because of 6.2, which Firefox cannot do ([BZ2010328]).

**While Izcal is closed**, the OS scheduler starts a **headless update session** every 6 hours† with up to 30 minutes† of random delay: a per-user scheduled task (Windows), a LaunchAgent registered with `SMAppService` (macOS) or a systemd user timer (Linux tarball). The session is a fixed set of processes: **the launcher-started broker in an `update-only` configuration, the network system service, the OS-adapter process (for OS proxy configuration, NW-D17), the crash handler and the updater**; no shell renderer, no UI host, no container or partition service, no storage service, no renderer. It reads enterprise policy (FI-3) for proxy and update settings, fetches only TUF metadata, build targets and data-channel targets (never a model file), and exits. It holds `<state>/session.lock`. **If the person opens Izcal during a session**, the launcher asks the session's broker to stop at the next safe point (after the current verified write, never mid-publish), waits up to 5 s†, then starts normally; FI-4's hand-off never sends a URL to an `update-only` broker. If any Izcal process of that install is running when the session starts, the session exits and leaves the work to it. On battery saver, no session runs. FP gains a line for sessions: CPU, energy and wall time per session† and sessions per day (Section 12). Linux packages leave closed-time updates to the package manager's tools (6.3).

**Relaunch.** The shell's 6.11 flow applies, with SH-24's guarantee that tabs, windows and ledger entries survive. This spec adds three things (Section 12): the update row says whether the waiting update is **security-only or a feature release** and links its advisory; a **security release whose advisory says "actively exploited"** shortens "Later" to 24 hours†; and the row shows "running 1.4.2; 1.4.3 is installed and will be used at next start" so the two-build state is never hidden. Shell 6.11's "applies without a relaunch" case now covers data channels only: a browser build always takes effect at the next start.

### 6.9 Staged rollout, halt and revert (DU-D5, DU-D6)

**The position.** Each install draws a 256-bit **local secret** once from `Entropy::crypto()` and keeps it in `<state>/rollout.secret`; profile reset does not touch it, because it belongs to the install, not the profile. For each release the client derives its **ramp position** as `HMAC(secret, channel ‖ release_id)`, truncated to 16 bits†. The position is fresh for every release, so a network observer who sees *when* an address first fetches a release learns nothing that links that install across releases. The first fetch inside a ramp step is spread with a random delay within the step's window†. Nothing about the secret or the position is ever sent (G254).

**Roles.** `releases` signs the build targets (UPD-R3) and marks each release's **class**: feature, security or emergency. A new **`rollout`** role, **delegated by `releases` with a path pattern limited to ramp files**, signs per channel a small ramp file that **selects among targets `releases` already signed**: for each feature release a start time and a schedule (for example 1% at T, 5% at T+24 h, 20% at T+48 h, 50% at T+72 h, 100% at T+96 h†), and optionally a `hold`. Its keys are online, in the release infrastructure. A ramp file has its own short expiry (24 h†); on stale ramp metadata the client stops widening and keeps what it has.

**What the client enforces, so a stolen `rollout` key is bounded.**
1. **Security and emergency releases are outside `rollout`.** Their ramp is fixed in `releases` metadata (24 hours† or none, the security owner's call recorded in the security-response log, UPD-R4), so only `releases` key holders can delay them.
2. **Every ramp has a maximum.** A feature release older than T + 14 days† is selected whatever `rollout` says.
3. **A hold expires** after 72 hours† unless `releases` confirms it by **delisting** the target; the durable halt is an act of the `releases` role, and `rollout` is a short emergency brake.
4. **A hold, once seen, can only be lifted onto the schedule that was signed**, never onto a faster one.
5. **A selection naming a target absent from `releases` metadata is refused.**

SEC-17's suite gains "rollout-key freeze", "rollout-key un-halt" and "unknown target in rollout" vectors (Section 12). This is Uptane's split between "what exists" and "what to install now" ([UPT]) without its inventory: the `rollout` role knows nothing about any client.

**Gates per release class.**
- **Feature releases.** Nightly users are ring 0 and Beta users ring 1. Stable starts only after Beta has run that feature release for 7 days† without a release-blocking crash signature in the crash reports people chose to send at D-S4's per-crash prompt, or a release-blocking public bug. The gates use no usage telemetry and no per-install identifier.
- **Security point releases.** The T0–T2 pipeline and SEC-2's drill path, then a Stable canary of 1–4 hours† on the maintainers' fleet and on Nightly-ring installs of the security branch, then the security ramp of rule 1. Publication plus ramp plus one poll fits inside SEC-1's bounds (72 h† for an exploited bug, 7 days† for critical and high, measured from SEC-1's time zero); DU-23 tests it.
- **Emergency (UPD-R4)** follows UPD-R4 and SEC-2, unchanged.

**Halt.** A halt is a `rollout` file with `hold` set, in a new snapshot. Running clients see it within one poll (15 minutes†) plus the time to sign and publish; closed clients at their next session. A held build that is staged but has not yet run is unstaged (`current` moves back to the running build); clients already running it keep it. Halt drills run at every feature release (S1 7.8's update drills).

**"Check now"** bypasses the ramp, as Apple's phased release and Balrog do: a person who asks for the update gets it. It never overrides a hold.

**Ramps are a per-channel property**, declared in the channel inventory (SEC-17 (c)). The **browser build** and the **GPU blocklist** are ramped, because code interprets them in ways that can crash it, which is the CrowdStrike lesson. The **phishing bundle, revocation and trust data, and the extension blocklist** are **not** ramped: they protect people, PH-D12 and PH-6 require one bundle for every client within minutes, and a per-client ramp would also make each fetch reveal a position; they get server-side canary validation before publication and the withdrawal runbook the phishing spec already has. The content-blocking bundle's choice is CB's. The **UPD-R5 emergency switches** apply at once, as today.

**Revert (DU-D6).** Fix forward is the default: a halted release is superseded by a higher version with the fix. When a fix will take longer than the harm, a **revert release** is **rebuilt from build N's source with a higher version number** (so it has a new `BuildId` and a forward version everywhere, including Windows `DisplayVersion`, `CFBundleVersion` and package versions; TUF forbids version rollback). For the revert to open profiles that build N+1 migrated, two standing rules apply to every build from the first public binary:
- **Every migration keeps the pre-migration root of each migrated store for one release†**, implemented as a **retained copy-on-write root** in the storage engine (SG-D2's page forest makes this pages, not copies). Clearing data (history, site data, passwords, a "Forget" action) also purges the retained root, so retention never undoes a deletion.
- **A build may open a profile one migration ahead** by restoring the retained root and then **replaying, onto the restored state, every user deletion and privacy-relevant setting change** made since the snapshot (cleared data, revoked permissions, deleted passwords, the AI switch, SH-8). A store for which replay cannot be done refuses the revert, and the build offers import instead.

The cost trigger is measured in retained pages per migrated store (DU-D6). Revert is drilled before each feature release (DU-21, SH-24).

**Terminal versions.** The `releases` metadata carries, per OS family and version, the **last build that supports it** and, where a security-only branch continues (6.11), that branch's channel. A client on a dropped OS stops at that build or moves to that branch and shows the end-of-support notice (6.11).

### 6.10 Signing identity, keys and reputation (DU-D10)

This spec adds a **key inventory** (SEC-16 (d) gives every key an age alert; the inventory lists them):

| Key or identity | Signs | Held | Rotation |
|---|---|---|---|
| TUF `root`, `releases`, `snapshot`, `timestamp` | Update metadata | UPD-R3, unchanged | SEC-16 drills |
| TUF **`rollout`** (new) | Ramp files only, delegated by `releases` | Online, in release infrastructure, threshold 1-of-2† | Drilled with the others |
| **Authenticode** | Every PE file | A CA-issued OV certificate on an HSM or cloud HSM, or Artifact Signing if Izcal's legal entity is eligible | At most 460 days by CA/B rule; one publisher identity across re-issuance |
| **Apple Developer ID Application / Installer** | macOS code and `.pkg` | Organisation account; Account Holder plus a second Admin (SEC-D4) | On expiry; used only in the release environment |
| **Repository OpenPGP key** (plus an offline successor) | APT `InRelease`, RPMs | Hardware keys, `releases`-class custody | Rotated through the keyring package |
| **Transparency log key** | The release log's checkpoints | Online, witnessed | Per the log's policy |

**An OS signature never authorises an update, and a TUF signature never satisfies Gatekeeper or SmartScreen**; each is checked for what it is.

**Reputation lead time.** The publisher identity that ships the first public Stable must sign **public Nightly builds at least 3 months† before** it, so SmartScreen sees hundreds of clean installs of that publisher's files ([SSR]). Because every release's files are new hashes, DU-10 is checked on every Stable release for its first 7 days†. The legal entity, its country (Artifact Signing serves individuals only in the US and Canada) and the certificate route depend on D-S8; organisation validation takes 1 to 20 business days, possibly longer ([ARTS]). **A build that resembles unwanted software is never signed** (bundled offers, a default-browser change outside the shell's ask), because it can give the identity negative reputation ([SSR]).

**Decided (owner, 25 October 2026).** The legal entity is **EU-established and comes first** (compliance CO-D1); then signed public Nightlies start, as **CRA Art. 4(3) testing builds** with their visible notice (6.6), at least 3 months before the first Stable (compliance CO-D3).

**The first public binary moves earlier.** Public signed Nightlies make the first public binary about 3 months† before the first Stable, so every gate keyed to "the first public binary" moves with it: SEC-D4's second release person, SEC-D10's external audit, SEC 6.16's P0 controls, S1 13.2's trademark clearance and export-control review, FI-1's CRA readiness for preview builds, and D-S8's signing identities. DU-M1 lists them (Section 9).

### 6.11 Cyber Resilience Act update duties, support period and end of support (DU-D14, DU-D15)

Izcal plans as a manufacturer (FI-D11). The duties below are design requirements; the legal reading of each is FI-1's, and Q-FI-1 (whether Nightly and Beta are "made available on the market") stays open, with Art. 4(3) as the design answer (6.6). The fact-check confirmed every quote below against the Official Journal text.

| CRA duty ([CRA][CRA]; [DUL 1][DUL]) | Izcal's design |
|---|---|
| Automatic security updates "enabled as a default setting, with a clear and easy-to-use opt-out mechanism", notification, "the option to temporarily postpone them" (Annex I Part I (2)(c)); recital 56: the person may approve "as a final step" | On by default on every format, Linux packages included (6.3). Settings > Updates has **"Install security updates automatically"**, a real switch, documented in the user information (Annex II 8(e)). **It governs installation of browser builds only**: data channels and UPD-R5 switches keep applying, documented as part of the product's security function (FI-1 checks the reading). Turning it off shows what is given up and keeps notification. "Later" in the update row is the postponement (shell 6.11). Verification is never switchable |
| Security updates "provided separately from functionality updates" where technically feasible (Part II (2)) | Security-only point releases on the current Stable (6.6); Extended in Stage 2 |
| Disseminated "without delay", free of charge, "accompanied by advisory messages" (Part II (8)); public disclosure of fixed vulnerabilities (Part II (4)) | One advisory per security release, published no later than the release by `hg-advisory`, as a human page and a machine-readable feed (**CSAF**, the OASIS standard, recommended; FI-1 confirms), with severity, affected and fixed builds per channel and OS, CVE IDs and whether exploited; the update row links it. Bug details may stay restricted for a stated number of days† after release (Part II (4)'s delay clause), because Izcal cannot measure "most users updated" |
| Inform users of an actively exploited vulnerability, "where appropriate in a structured, machine-readable format" (Art. 14(8)) | The advisory feed, the shortened "Later" of 6.8 and an About-page notice |
| Support period of at least five years reflecting expected use (Art. 13(8)); end date "at least the month and the year" shown at acquisition; an end-of-support notice "where technically feasible" (Art. 13(19)) | A **per-platform support statement fixed at release** (compliance CO-D5 (c), CO-D4): the support end for a release on each platform is the earlier of five years and that OS version's vendor end of security support, or Izcal's conservative estimate where the vendor publishes none (macOS); delivered as the latest release on that platform. The stated end date (month and year) is on the download page, in About and in `about:version`; for an OS whose vendor support ends within five years the download page shows both dates. Which OS versions get new releases follows GR-D21's rule at each release decision; a client past its stated date shows one notice per week†, limited to what is necessary (recital 56). Whether a period shorter than five years on an old OS is lawful is counsel's (compliance Q-CO-7); the reversal is CO-D5 (a) |
| Each security update available for at least 10 years after issue or for the rest of the support period, **whichever is longer** (Art. 13(9)); public archives must warn about unsupported versions (Art. 13(11)) | Immutable releases (SEC-13) and the package pools keep security releases at least that long, using TUF hashed-bin delegations so archived targets do not grow the live metadata; the archive page warns about every build that is no longer current |
| Patching only the latest version is allowed if users can move to it free and without extra hardware or software costs (Art. 13(10)); recital 40: the condition can be met "where a desktop operating system upgrade does not require new hardware" | Under CO-D5 (c), dropping an OS under GR-D21 ends new releases there; if the OS loses vendor support **before the end date already stated for releases on it**, the last build for that OS becomes a **security-only branch until that stated date** and no longer (at most one short branch per platform, usually none), routed by the terminal-version record (6.9). Recital 40 is why the branch exists at all. Its cost (0.2–0.5 engineer†) is the compliance spec's (CO-D5) and is counted against the engine security track (Section 9) |
| Reset to the original state (Part I (2)(b)); permanent removal of all data and settings (Part I (2)(m)); secure decommissioning information (Annex II 8(d)) | The shell recovery's "Reset profile settings" is the reset; 6.12's "Remove Izcal and all its data" is the removal; the user information documents both |
| SBOM covering at least top-level dependencies (Part II (1)); Annex VII description of update distribution | S1 7.8's SBOM per release (Section 12 proposes adding it to SEC-13); this spec's 6.2–6.10 are the Annex VII description |

Implementing Regulation (EU) 2025/2392 defines the browser category ("software products … that enable end users to access, render, and interact with web content", including standalone browsers) and says nothing about pre-release builds ([IR 2025/2392][IR2392]).

**Supported-OS policy (DU-D15).** The OS integration spec's assumed floors (Q-OI-14: Windows 11 22H2 and later; macOS 26 as the last Intel release) are what this rule gives at a release decision taken today; they are re-derived at each release decision, not fixed here. Targets are **a rule applied at each release decision**, not a list fixed in October 2026: GR-D21's rule (an OS version is supported while its vendor ships consumer security updates at the release decision), plus the per-platform support statement fixed at release (CO-D5 (c)), the terminal-version record and, only if an OS loses vendor support before its stated date, the short security-only branch. Stage 1 exits 48–60 months† out (S1 8.2), so Windows 10 (consumer ESU to 12 October 2027, [W10ESU]) and Intel Macs may be outside vendor support before the first Stable; the build matrix keeps their triples only while that is undecided, and DU-M3 decides whether `x86_64-apple-darwin` ships at all. Apple publishes no end dates, so the macOS statement uses Izcal's conservative estimate from Apple's practice of patching the three most recent releases ([DUP 3.8][DUP], Medium), as CO-D5 (c) requires.

**Out-of-date warning.** Two stages over SEC-17's state: when timestamp metadata expires (SEC-17's ceiling, 3 days†) the update row shows the quiet "updates are stale" state; when the last successful verification is older than 8 days†, the row becomes "Izcal can't check for updates" with the reason it knows (offline, clock wrong, verification failed). Both use only local state.

### 6.12 Uninstall, repair and leftover cleanup (DU-D12)

**The install ledger is the OS integration spec's registration journal** (OI-D10: `registrations.izj`, read and written by `hg-os-journal`, shared with the uninstaller), so there is one typed record of every artefact. OS registrations (handlers, default-browser registration, shortcuts, AUMIDs, login items, desktop entries) are OI's record kinds; this spec asks for four more (Section 12): **update scheduler entry**, **keychain item**, **OS cache directory** and **crash-dump directory**. Together they cover every artefact Izcal creates **outside its install, state, data and profile roots**: scheduler entries, LaunchAgents and systemd user units; Uninstall registry keys; protocol, file-type and default-browser registrations (FI-4); notification and AUMID registrations; OS keychain items (Keychain, DPAPI-protected blobs, Secret Service collections, SEC-34); caches in OS cache directories; crash dumps. Entries are **typed** (a task name, a keychain item by Izcal's service name, a registry key under Izcal's names, a path under Izcal's roots), never arbitrary paths, so a same-user process that edits the ledger cannot direct deletion outside Izcal's own artefacts. Every writer of such an artefact records it through `hg-os-journal`; T1 checks that each OS-registration call site has a journal record kind.

**Two removal paths, one result.**
- **In the product**: Settings > "Remove Izcal and all its data" lists every ledger entry and every profile with its size, removes them, then hands off to the OS uninstaller (Windows), moves the app to the Bin (macOS) or tells the person which package command to run (Linux packages). It works the same on all three OSes and is the CRA's permanent-removal path.
- **From the OS**: Windows Installed apps runs Izcal's uninstaller, which removes the install root and every ledger entry except the profile, with a checkbox **"Also delete my Izcal data (profiles, saved passwords, history)"**, **unchecked by default**. On macOS, dragging `Izcal.app` to the Bin removes every build (the helper frameworks are inside it) and the `SMAppService` LaunchAgent with it; `<state>/retired/` bundles, profiles and the journal's other artefacts remain, so the macOS user information points to the in-product path. Linux packages remove the install root on `remove` and never touch `$HOME` ([DEBP]).

**No exit survey and no uninstall ping**: a network request after the person has left is a data flow FI-D12 does not allow.

DU-13's leftover audit diffs the file system, registry, scheduler, keychain and registrations before install and after removal on each OS, against the ledger plus a **published list of OS-owned records** Izcal cannot remove (prefetch files, TCC and LaunchServices databases).

### 6.13 First run, initial preferences and install-time options (DU-D13)

- **No download attribution.** The installer is byte-identical for every downloader and carries no campaign tag; Mozilla's attribution sends the referring domain or campaign to the vendor ([Firefox privacy notice][FXPN]), which FI-D12 rules out, and identical bytes keep the reproducible-build comparison simple.
- **Initial preferences (Stage 2)**: a first-run-only `initial-preferences` file beside the launcher (homepage, startup pages, the default container set, bookmarks to import), applied once at profile creation, overridden by policy (FI-3), carrying no phone-home. It **cannot set `ai.enabled`, telemetry consent, crash-report defaults or any protective setting** (blocking, phishing, sandbox tier, update switches), so it cannot become a bundling vector. This is the shape of Chrome's `initial_preferences` ([Chrome Enterprise][CHRIP]), built as Izcal's own format.
- **No partner rebrands.** A partner that substantially modifies and redistributes Izcal may become its CRA manufacturer (Art. 22); configuration files, not rebrandable builds, are the supported customisation.
- **Install-time choices** on Windows: a Start menu entry (always), a desktop shortcut (asked, off by default), and nothing else; the first-run "Things to know" screen can offer the taskbar pin through the OS's consented API (shell 6.14).

### 6.14 Platform targets (DU-D16)

| OS | x86-64 | AArch64 | riscv64 |
|---|---|---|---|
| Windows | `x86_64-pc-windows-msvc`, on the Windows versions GR-D21 admits at each release decision | `aarch64-pc-windows-msvc`, native Arm64, Windows 11 | — |
| macOS | `x86_64-apple-darwin` while Intel Macs' macOS is patched; DU-M3 decides whether it ships | `aarch64-apple-darwin` | — |
| Linux | `x86_64-unknown-linux-gnu`: `.deb` (Debian, Ubuntu releases in vendor support), `.rpm` (Fedora current and previous, RHEL in support), tarball | `aarch64-unknown-linux-gnu`, same formats | `riscv64gc-unknown-linux-gnu` built for RVA23 (JS-D22): `.deb` for Debian 13+ and Ubuntu 25.10+ on RVA23 hardware, tarball; RPM best effort while Fedora's port is an alternative architecture |

**One build per architecture, not a macOS universal binary**: FP-25 budgets per architecture, and the updater fetches only its own. The coding standard's A.4 `deny.toml` targets (three triples) gain the four missing ones, and S1 7.1's build matrix carries all seven (Section 12). Android (Stage 2) is FI-33's, with this spec's rollout and signing rules applying: Play App Signing with Izcal's own key, signature scheme v3 for rotation, and developer verification before Stage 2 ([DUP 5][DUP]).

### 6.15 Release engineering, symbols and release notes (PO-16.9, PO-3.14)

- **On-call**: JS-16's 24-hour on-call rotation covers halts; a halt needs one person with the `rollout` key, and a durable halt (delisting) needs a `releases` key holder within the hold's 72 hours†.
- **The release dashboard** shows, per channel: the ramp's fraction, crash signatures per build from the reports people chose to send, public bug reports tagged with the build, and the transparency log's latest checkpoint. It shows no per-install data because there is none.
- **Release notes** for every release, human-written for feature releases, generated from the advisory for security releases, published with the release and linked from About.
- **Symbols**: each build's symbol files are signed, logged and archived as long as the build archive (Art. 13(9)), for crash symbolication (FI-11) and replay. Replay of a recording from a build no longer installed refuses with its `BuildId` and offers that build's replay binary from the archive.
- **Extension blocklist (PO-4.9)**: a UPD-R1 data channel like the others, a list of (extension ID, version range, action, reason, expiry) signed under a delegated targets role, not ramped (6.9), with every entry shown in the extensions UI with its reason (EX owns the UI; Stage 2).

### 6.16 Security, privacy and consent

- **No identifier leaves the machine** in any update flow (G254, DU-5): no install ID, no position, no profile ID; requests carry only what TUF needs (role names, versions, target paths), with platform and architecture in the path; first fetches are jittered within the ramp step (6.9).
- **Metered and battery behaviour (DU-D19)**: when the OS reports a metered or constrained connection (Windows `NetworkCostType` `Variable`, `OverDataLimit`, `BackgroundDataUsageRestricted` or roaming; NetworkManager `metered yes` or `guess-yes`; Apple `isExpensive` or `isConstrained`), Izcal fetches **only TUF metadata, data channels and security releases**, defers feature releases until an unmetered connection or the person's "Download now", and says so in the update row. On battery below 20%† it defers staging except for security releases; on battery saver no headless session runs. The network service reads cost through the OS-adapter process (a new OS source for it, Section 12); the updater stays network-less.
- **Binary transparency (DU-D17)**: every browser release target is logged in Izcal's witnessed release log (the shared log UPD-R1 already gives the phishing bundle), and the **inclusion proof ships with the target**, so the client needs no access to the log; `hg-update` refuses a release target whose proof does not verify. Publication waits for inclusion, with a bound† drilled in SEC-2's emergency drill. Packages are covered through the inventory hash (6.3). This moves the security scope's P2 "binary transparency" control to P1 for browser releases (Section 12), because one coerced maintainer (UPD-R3's caveat) is the risk it addresses.
- **Reproducibility of what is distributed**, as a dated proposal to SEC-15 (Section 12), in stages: engine images (SEC-15, from M7), then the Linux packages and tarball, then the Windows installer, then the `.dmg`, each when its feasibility is measured. The comparison covers **payload bytes before signing**, plus a check that each signature covers exactly those bytes (Authenticode, Developer ID, notarisation tickets and stapling are not reproducible). **UPD-R4's exception applies unchanged**: an emergency release may publish on a verified attestation while the comparison runs in parallel.
- **Threat model.** The updater's threat model (SEC 6.5) gains: the `rollout` role and its client-side bounds; the delta parser behind verify-then-parse; the headless update session; the chunked transfer; the repository key as root on Linux clients; the typed install ledger.

### 6.17 Alternatives

| Question | Options | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|
| Running-build safety | (a) versioned directories; (b) zygote; (c) detect and relaunch | **(a)**, with (c) as fallback (6.2) | 2× install size in transition | DU-M0b shows an OS where (a) cannot hold |
| Linux package shape | (a) one package per build plus a metapackage; (b) one package with files under a versioned directory, replaced on upgrade | **(a)**: under (b) dpkg and rpm delete the old build's files during the upgrade | One package name per build in the repository | Autoremove does not clean old builds on a reference distribution: add a cleanup hook to the metapackage |
| Rollout position | (a) fresh per-release position from a local secret; (b) one bucket for every release; (c) per-request dice (Balrog); (d) server-side cohorts | **(a)** | Halts are global per release | — |
| Delta format | (a) dictionary-style; (b) executable-aware; (c) none | **(a)** first | Smaller savings than (b) | DU-M2 measures (b) ≥ 2×† better |
| Stable cadence | (a) 4 weeks† plus security point releases; (b) 2 weeks | **(a)** | Features reach users later | Pipeline under 3 working days† for two quarters |
| macOS update mechanism, within OI-D3's layout | (a) stage a complete signed bundle and exchange it atomically; (b) update the installed bundle in place | **(a)** | Each published bundle carries two helper frameworks; the release pipeline signs that form | DU-M0b and OI-M1 show (b) verifies at every step and (a) breaks TCC, Keychain or Launch Services |
| Windows Stage 2 package | (a) MSI; (b) MSIX | **(a)** | Older technology | Q-DU-6 shows MSIX can host Izcal's sandbox |
| Snap | (a) not offered; (b) publish | **(a)** | Ubuntu users who prefer snaps use the `.deb` | Ubuntu stops supporting third-party `.deb` repositories, or Snap updates can be verified through Izcal's TUF chain |

### 6.18 Decisions for the owner

**All nineteen decisions were accepted by the owner as recommended on 25 October 2026** (DU-D1, DU-D5, DU-D8, DU-D14 and DU-D15 reviewed explicitly; DU-D15 together with compliance CO-D5). A bare "yes" means the recommendation. Build-versus-reuse is not on this list: the standing answer is "build our own, following the specifications" (M-1, D-S10). FI-D10 is the owner's decision of 24 October 2026 and is specified here, not reopened. **The owner most needs to see DU-D1, DU-D4, DU-D5, DU-D8, DU-D10, DU-D14 and DU-D15.**

| ID | Decision | Options | Recommendation | Reversal trigger |
|---|---|---|---|---|
| **★ DU-D1** | Running-build safety (**accepted by the owner, 25 October 2026**) | (a) versioned build directories on every OS and format, own-build launch, external liveness locks, full pre-open of the build's inventory, no Izcal-shipped shared libraries; (b) a zygote or fork server; (c) detect and force a relaunch | **(a)**, with (c) only as the fallback state (6.2) | DU-M0b shows an OS where a running build cannot keep launching from its own build beside a newer one |
| **DU-D2** | Background updates and relaunch (**accepted by the owner, 25 October 2026**) | (a) an OS-scheduled headless session of named processes every 6 h† while closed, checks on the existing 15-minute† poll while running, shell 6.11's relaunch with the additions of 6.8; (b) a resident update agent; (c) update only while running | **(a)** | FP's new session line is exceeded at DU-M2: lengthen the interval |
| **DU-D3** | Channels and side-by-side identity (**accepted by the owner, 25 October 2026**) | (a) Nightly, Beta, Stable in Stage 1, Extended in Stage 2; each channel its own install identity and profile root; Nightly and Beta designed to CRA Art. 4(3); profiles stamped, opened at most one migration ahead; (b) shared profiles across channels | **(a)** (6.6) | — |
| **★ DU-D4** | Stable cadence (**accepted by the owner, 25 October 2026**) | (a) feature release every 4 weeks† plus security-only point releases; (b) every 2 weeks, as Chrome, Edge and Firefox since August–September 2026 | **(a)** | Two quarters† with the full pipeline under 3 working days†: move to (b) |
| **★ DU-D5** | Staged rollout without per-client data (**accepted by the owner, 25 October 2026**) | (a) a fresh per-release position from a local secret; ramp files signed by an online `rollout` role delegated by `releases` and bounded by the client (security releases outside it, a 14-day† maximum, holds that expire unless delisted, no un-halt onto a faster schedule); ramps per channel (build and GPU blocklist yes; phishing, revocation, extension blocklist no); (b) one bucket for all releases; (c) server-side cohorts | **(a)** (6.9) | A halt drill misses DU-4 twice: shorten the poll for release metadata |
| **DU-D6** | Rollback (**accepted by the owner, 25 October 2026**) | (a) fix forward, plus a revert release rebuilt from the previous source under a higher version, enabled by retained copy-on-write pre-migration roots and deletion replay; (b) fix forward only; (c) true downgrade | **(a)** | Retained pages exceed a measured bound† per migrated store at DU-M2: drop to (b) for that store |
| **DU-D7** | Delta updates (**accepted by the owner, 25 October 2026**) | (a) Izcal's own dictionary-style format, verify-then-parse, full fallback, output verified against the target inventory; (b) executable-aware; (c) none | **(a)** (6.7) | DU-M2: (b) saves ≥ 2×† more |
| **★ DU-D8** | Linux packaging (**accepted by the owner, 25 October 2026**) | (a) FI-D10 as specified: one package per build plus a metapackage, automatic security updates through the distribution's unattended-upgrade tools configured for Izcal's repository (or Izcal's own system timer), the package manager recorded as the per-machine elevated component, an Ed25519-signed repository, an AppArmor profile, a tarball with Izcal's updater; Flatpak in Stage 2; **Snap and AppImage not offered in Stages 1–2**; (b) as (a) with a Snap | **(a)** | Ubuntu stops supporting third-party `.deb` repositories; FI-1 rules distribution-mediated updates are not "automatic" (then Izcal's system timer becomes the default) |
| **DU-D9** | Windows packaging (**accepted by the owner, 25 October 2026**) | (a) per-user signed installer (no stub, no tag) in Stage 1, MSI in Stage 2, **Microsoft Store refused** (policy 10.2.1), MSIX only if Q-DU-6 passes, native Arm64; (b) MSIX first | **(a)** | Store policy 10.2.1 changes: reopen the Store question |
| **★ DU-D10** | Signing identity and lead time (**accepted by the owner, 25 October 2026**) | (a) an EU-established legal entity (the EU is Artifact-Signing-eligible; otherwise an OV certificate on an HSM), one stable publisher identity, every PE file signed, public signed Nightlies at least 3 months† before the first Stable with every first-public-binary gate moved with them, an Apple organisation account with a second Admin; (b) sign only at launch | **(a)** (6.10), with an EU-established legal entity first (compliance CO-D1) and the Nightlies as CRA Art. 4(3) testing builds (CO-D3) | Q-DU-1 shows reputation does not survive re-issuance: plan re-signing windows around renewals |
| **DU-D11** | macOS update mechanism and stores (**accepted by the owner, 25 October 2026**) | Within OI-D3's layout (the bundle executable is the broker; versioned helper frameworks inside the bundle): (a) stage a complete signed bundle holding the new and the running build's helper frameworks and exchange it atomically; (b) update the installed bundle in place; in both, `.dmg` in Stage 1, `.pkg` in Stage 2, every bundle notarised, **Mac App Store refused** | **(a)**, confirmed at DU-M0b with OI-M1 | DU-M0b shows (a) breaks TCC, Keychain or Launch Services and (b) verifies at every step |
| **DU-D12** | Uninstall and data removal (**accepted by the owner, 25 October 2026**) | (a) typed install ledger, in-product "Remove Izcal and all its data", OS uninstaller with the data box unchecked, no exit survey; (b) OS uninstallers only | **(a)** (6.12) | — |
| **DU-D13** | Attribution and initial preferences (**accepted by the owner, 25 October 2026**) | (a) byte-identical installers, no attribution, a local first-run initial-preferences file in Stage 2 that cannot set AI, consent or protective settings, no partner rebrands; (b) tagged installers | **(a)** (6.13) | — |
| **★ DU-D14** | CRA update duties (**accepted by the owner, 25 October 2026**) | (a) automatic security updates on every format with a Settings opt-out governing browser builds only, security-only point releases, CSAF advisories at release, the support end date shown, archives for 10 years or the support period, whichever is longer; (b) the same without a consumer opt-out switch | **(a)** (6.11): (b) does not meet Annex I Part I (2)(c) | FI-1's legal advice changes a reading |
| **★ DU-D15** | Supported-OS policy (**accepted by the owner, 25 October 2026**) | (a) GR-D21's rule applied at each release decision for which OS versions get releases; **the per-platform support statement fixed at release, the same rule as compliance CO-D5 (c)** (the earlier of five years and the OS version's vendor end, or Izcal's conservative estimate); a terminal-version record; a security-only branch only if the OS ends before the stated date, until that date; (b) a fixed OS list chosen now | **(a)**, consistent with CO-D5 (c) | Counsel rejects a per-platform period shorter than five years (Q-CO-7): CO-D5 (a), legacy branches for five years, with its cost added to S1 8.3 |
| **DU-D16** | Platform targets (**accepted by the owner, 25 October 2026**) | (a) x86-64 and AArch64 on all three OSes, riscv64 Linux at RVA23 with an install-time ISA check, one build per architecture; (b) a macOS universal binary | **(a)** (6.14) | — |
| **DU-D17** | Binary transparency for browser releases (**accepted by the owner, 25 October 2026**) | (a) P1: the updater refuses a release target without a verifying inclusion proof shipped beside it; (b) P2, as the security scope has it | **(a)** | The log's operation costs more than its value at DU-M2: back to (b), recorded |
| **DU-D18** | Distribution-built and third-party packaging (**accepted by the owner, 25 October 2026**) | (a) a published policy: updater off for the binary in distribution builds, identified as distribution-built, data channels on TUF, trademark rules; third-party manifests install the signed installer unchanged; (b) no policy | **(a)** | — |
| **DU-D19** | Metered and battery policy (**accepted by the owner, 25 October 2026**) | (a) on metered or constrained connections fetch only metadata, data channels and security releases; defer staging below 20%† battery except security; no session on battery saver; (b) ignore cost signals | **(a)** (6.16) | — |

## 7. M-1 compliance

**Dependencies used unmodified.** None new. The cryptographic primitives `hg-update` already uses (SEC 6.16), plus Ed25519 signing in `hg-crypto` for tools; the OS interfaces of each platform (scheduler, file system, registry, Keychain, connection cost, `SMAppService`) under H2 2.3.2's OS-interface exception; Apple's notary service and `hdiutil`, the Windows Installer database API and Microsoft's or a CA's signing services, under the toolchain and OS exception, never shipped.

**Built new, by Izcal.** `hg-install`, `hg-delta` and its format, `hg-launch-main`, `hg-updater-main`, the substrate's install-device trait and OS-registration modules, `hg-rollout`, `hg-pkg-build` (the `.deb`, `.rpm`, tarball and Windows installer writers, and the drivers for the OS packaging tools), `hg-repo-sign` (OpenPGP), `hg-advisory`, the profile stamp, the retained pre-migration roots and deletion replay.

**What each part is anchored to.** The TUF specification 1.0.37 ([TUFS]) for metadata, delegations and the `rollout` role; [RFC 9580][RFC9580] for OpenPGP; the Debian Policy Manual and the Debian repository format documentation for `.deb` and APT repositories; the RPM package and repository format documentation; Microsoft's Windows Installer and Uninstall-key documentation; Apple's code-signing and notarisation documentation; the freedesktop Desktop Entry specification; the CRA's text; CSAF (OASIS) for advisories. Where a format's only complete description is an implementation's source, Izcal writes it from published prose and black-box output (for example, inspecting a package with the distribution's own tools) and records the observation in `compat/observations/`, never from source (M-1 part 3).

**Evidence rule.** The research notes cite only prose: specifications, vendor documentation, engineering blogs, bug-tracker discussion and black-box observations. Chromium prose documents hosted in its repository (the zygote, deterministic-builds and updater documents) were read as prose under OD-7's conditions; no code file beside them was opened, and no outside copy exists for the two this spec links ([CZ], [CHRP4]). Uptane is cited from its outside copy on uptane.org. The Zucchini cost figure in the notes (DUM 2.4) comes from a README inside a code directory and **is not used** by this spec. One bug report (Bugzilla 1888601) reasons from source code it links; this spec uses only its reported symptom, as the reporter's suspicion, and did not follow the links. The Chrome and Firefox bundle layouts in [DUM 1.4–1.5][DUM] are black-box directory listings of installed copies.

## 8. Targets

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **DU-1** | **No update path breaks a running browser** | **0** `BuildMismatch` exits and **100%** of new tabs load, in a matrix that installs a newer build under a running browser through every Stage 1 path (Windows per-user updater, macOS updater, tarball updater; on Debian and Ubuntu `apt upgrade`, `apt-get upgrade` and unattended-upgrades; on Fedora `dnf upgrade` and `dnf-automatic`; autoremove of the old build package while it runs; and `rm -rf` of the old build directory while it runs) and then opens 100 tabs, restarts 3 services and the GPU process | T2 harness on the reference machines; DST scenario that replaces a build on disk between launches | Ship blocker for that path; a path that cannot pass after DU-M0b's stub fallback is removed from the matrix by recorded decision, with the restart-required state as its behaviour |
| **DU-2** | Old builds are reclaimed (not for Linux packages) | A non-current build with no live process is deleted at the next launcher start or updater run; at most **2** builds plus `staging/` on disk; the About page shows space used by old build packages on Linux | T1 test | Fix before release |
| **DU-3** | Restart-required is a state, not a loop | When the `BuildId` check fires, **0** launch retries beyond the first and the update row shows within 1 s† | T1 fault-injection test | Fix before release |
| **DU-4** | Halt latency and bounds | A `hold` published at T stops **100%** of running test clients that have not yet staged the release from staging it after T + 30 min†; a staged-but-unrun held build is unstaged; a hold not confirmed by delisting expires at 72 h†, and a client never widens faster than the signed schedule | Release drill with lab clients; SEC-17 vectors | Drill repeated; the release does not widen until it passes |
| **DU-5** | Update traffic identifies no install | **0** bytes of secret, position, install ID, profile ID or any per-install value in any update, delta or repository request over a week from 3 lab installs; and across 20 lab installs and 10 releases, the rank correlation of each install's first-fetch time between releases is indistinguishable from independent draws (p > 0.05†) | T1 traffic capture and diff; timing analysis in the release lab | Ship blocker |
| **DU-6** | Every apply is verified | **100%** of applied builds match their `build.inventory` file by file before `current` moves; **100%** of failed deltas fall back to the full target; **0** delta bytes parsed before the delta's hash verifies | T1 suite with corrupted deltas and bases | Ship blocker for the updater |
| **DU-7** | Deltas are worth having | Median delta ≤ **25%†** of the full target over the last 10 Stable pairs, per OS and ISA | Release pipeline report | Revisit the format (DU-M2) |
| **DU-8** | Releases arrive | In the lab, from publication: staged on a running online client in ≤ **1 h†**; on a closed online client in ≤ **8 h†** (one 6-hour† interval, its 30-minute† delay, fetch and staging) | Release drill | Fix the scheduler or poll path |
| **DU-9** | Everything distributed is signed | **100%** of PE files Authenticode-signed; **100%** of macOS bundles, Nightly included, notarised and stapled; **100%** of repository indexes and RPMs signed with the Ed25519 key and accepted by `sqv` and `rpm` in CI; **100%** of Linux package payloads byte-identical to their build inventory | Release checklist; CI verification | No release |
| **DU-10** | No SmartScreen warning | **0** SmartScreen or Smart App Control warnings for the first public Stable download, and for every Stable release during its first 7 days†, on a fresh Windows 11 machine with default settings | Launch-day and per-release check | Launch waits; for later releases, a release defect with a postmortem |
| **DU-11** | No Gatekeeper prompt beyond the standard first-open dialog | Fresh macOS 15+ machine, downloaded `.dmg`: opens with only the standard "downloaded from the Internet" confirmation, for the launcher and every build bundle | Release check | No release |
| **DU-12** | Package install and upgrade are clean | `.deb` passes `lintian` with no errors and `.rpm` passes `rpmlint` with no errors; installs, upgrades and refuses non-RVA23 riscv64 CPUs correctly on each reference distribution, with no prompt | CI | Fix before release |
| **DU-13** | Uninstall leaves nothing unchosen | After "Remove Izcal and all its data", **0** artefacts remain outside the published list of OS-owned records (file system, registry, scheduler, keychain, registrations) on each OS; after the OS uninstaller with the data box unchecked, only the profile, state and data roots remain | T2 audit per OS | Ship blocker |
| **DU-14** | Channels are isolated and profiles never go backwards silently | Nightly, Beta and Stable side by side share **0** files and **0** profiles; a build opening a profile more than one migration ahead refuses with the import offer in **100%** of cases and changes **0** bytes of it | T1 suite | Ship blocker |
| **DU-15** | Automatic security updates are default and the opt-out is real | Fresh install on every format, Linux packages included: automatic security installs on; switch off → **0** automatic build installs over a week while notifications still appear and data channels still update; documented in the user information | T1 test per format; docs review | Ship blocker (CRA) |
| **DU-16** | Support dates are shown | The support end date stated for the build's platform at release (month and year, CO-D5 (c)) appears on the download page, About and `about:version` for **100%** of builds, and the download page shows both dates where the OS's vendor support ends within five years; a client past its OS's end date shows the weekly notice | Release check; clock-advanced VM | Ship blocker (CRA) |
| **DU-17** | Advisory at release | **100%** of security releases have a human and machine-readable advisory published no later than the release itself | Release checklist | Release held |
| **DU-18** | Security releases change no web-visible identity | Between a feature release and its security point releases, **0** changes to the UA string, client hints, exposed IDL and CSS features, and default settings | T1 diff of the exposed surface | Security release re-cut |
| **DU-19** | Credentials survive the future | The browser runs at +1, +2, +5 and +10 years† of clock offset with **every** certificate, key and metadata expiry in the key inventory either still valid or producing its designed stale state, never a silent disable | T2 clock-skew suite | Ship blocker |
| **DU-20** | Packaging costs no cold start | Cold-start time from each package format within **5%†** of the unpacked build run directly on the same machine, per OS | Perf lab | Fix the format |
| **DU-21** | Revert works and respects the person | A revert release from build N+1 to N's source restores a profile N+1 migrated, with SH-24's guarantees, and **100%** of deletions and privacy-relevant changes made under N+1 (cleared data, revoked permissions, deleted passwords, the AI switch) survive the revert | Release drill; SH-24's three-version test with a revert | Feature release waits |
| **DU-22** | Metered connections are respected | On a metered or constrained connection, **0** bytes of feature-release payload fetched without the person's action | T2 lab test per OS | Fix before release |
| **DU-23** | Security releases fit SEC-1 | In the release drill, from SEC-1's time zero to staged on a running online lab client: ≤ **72 h†** for an exploited bug and ≤ **7 days†** for critical or high, including canary, ramp and one poll | SEC-2 drill | SEC-1's failure action; shorten the canary or ramp |

## 9. Milestones and staffing

| Milestone | When (S1 8.1) | Contents | Exit |
|---|---|---|---|
| **DU-M0a** | M1 phase E | The launch rules of Section 13: launch-by-handle for Izcal kinds, own-build resolution, the `BuildId` definition, the liveness hook, the install-device trait in the substrate and its simulation model, the DST scenario | Phase E merged with these in place |
| **DU-M0b** | With M5 | OS measurements: the macOS layout checks of 6.5, Windows behaviour of held and renamed images (Q-DU-8), `apt`/`dnf` autoremove and unattended upgrades of build packages (Q-DU-3), AppArmor attachment on `execveat` of a removed file (Q-DU-13), notary latency (Q-DU-7). Depends on D-S8's signing identities for Gatekeeper, notarisation and TCC (ad-hoc signing for what it can show) | Each measurement recorded under `measurements/`; DU-D1 and DU-D11 confirmed or reversed |
| **DU-M1** | Before the first public Nightly (the first public binary), at least 3 months† before the first Stable; after M5 | `hg-install`, `hg-launch-main`, `hg-updater-main` with full targets; per-user Windows installer, `.dmg`, tarball, `.deb` and `.rpm` with the repositories and unattended-upgrade configuration; signing identities live; transparency log; DU-1, DU-2, DU-3, DU-5, DU-6, DU-9, DU-12, DU-14, DU-15 (packages). **First-public-binary gates**: SEC-D4's second release person, SEC-D10's external audit, SEC 6.16's P0 controls, trademark clearance and export-control review (S1 13.2), FI-1's Art. 4(3) design for previews, D-S8 | Nightly ships daily with every listed target and gate passing |
| **DU-M2** | Before the first public Beta | `hg-delta`, the `rollout` role and ramps, halt and revert drills, retained roots and deletion replay, headless sessions, metered policy, the in-product removal, advisories; DU-4, DU-7, DU-8, DU-13, DU-15 (all formats), DU-16 to DU-19, DU-21, DU-22, DU-23 | Beta runs on the ramp |
| **DU-M3** | First public Stable (no earlier than S1's release gates allow) | DU-10, DU-11, DU-20; the support statement published; the Intel macOS decision | Launch |
| **DU-M4** | Stage 2 | MSI, `.pkg`, Extended channel, initial preferences, Flatpak (Q-DU-11), MSIX (Q-DU-6), Android signing (FI-33) | With FI-3's delivery |

**Staffing, per crate (engineer-months†).**

| Work | Stage 1 | Stage 2 |
|---|---|---|
| `hg-install`, the substrate's install-device trait and OS-registration modules on three OSes | 8–12 | — |
| `hg-launch-main` and the hand-off integration | 3–4 | — |
| `hg-updater-main` apply path, chunked transfer, headless session (beyond SEC's TUF client) | 6–9 | — |
| `hg-delta` | 4–6 | — |
| `hg-pkg-build`: `.deb` 2–3, `.rpm` 3–4, tarball 0.5, Windows installer and uninstaller 4–6, `.dmg` via `hdiutil` 1–2 | 10.5–15.5 | `.msi` via the Windows Installer API 4–6, `.pkg` 2–3 |
| `hg-repo-sign` and Ed25519 signing in `hg-crypto` | 3–5 | — |
| `hg-rollout` and the client bounds | 2–3 | — |
| `hg-advisory` | 1–2 | — |
| Transparency log and its operation | 3–4 | — |
| Retained pre-migration roots and deletion replay (storage and shell) | 3–5 | — |
| Extended channel, Flatpak | — | 4–6 |
| **Total** | **43.5–65.5** | **10–15** |

That is about **2–3 engineers†** over the roughly two years from DU-M0a to DU-M3, in S1 8.3's Infrastructure track ("CI tiers, oracles, perf lab, fuzzing, release", 2–3†), which this work alone would fill; Section 12 proposes the row grow by that amount. **Run cost**: 0.5–1 release manager† from DU-M1. The backport load of security point releases, the Extended channel and any Art. 13(10) branch is counted against the engine security track (SEC-D2, JS-16), not here.

## 10. Risks

| Risk | Likelihood | Effect | Mitigation |
|---|---|---|---|
| The macOS bundle exchange breaks TCC, Keychain or Launch Services state, or the two-framework bundle fails notarisation | Medium | DU-D11 needs (b) | DU-M0b and OI-M1 measure with a real update under a running broker |
| AppArmor does not attach to `execveat` of a removed file | Medium | Reduced renderer tier after an autoremove | The metapackage's stable stub (6.3); DU-M0b |
| Unattended-upgrade tools do not install the metapackage's new dependency | Medium | Linux clients stay on old builds | Izcal's own system timer as default where needed; the broker's update row (6.3) |
| Autoremove does not remove old build packages, so `/opt` grows | Medium | Disk use on Linux | DU-M0b; a cleanup hook; About shows the space |
| SmartScreen reputation does not carry across certificate re-issuance or Artifact Signing's short certificates | Medium | Warnings after each renewal | Q-DU-1 tested before relying on it; one publisher identity; DU-10 per release |
| The legal entity for signing is not in place (D-S8) | High while D-S8 is open | DU-M1 slips, and the first public binary with it | Raised now |
| A single maintainer holds every key (SEC-D4) | High until SEC-D4 is met | A coerced or compromised person can ship a build | Binary transparency (DU-D17), reproducible packages, SEC-D4's second person before DU-M1 |
| The `rollout` key is stolen | Low | At most a 72-hour† hold or a slowed ramp on feature releases; no effect on security releases; no forged build | The client's bounds (6.9); SEC-17 vectors |
| Retained roots or deletion replay prove hard for a store | Medium | Revert unavailable for that store | Fix forward; revert refused for that store (DU-D6) |
| A 4-week cadence looks slow beside 2-week competitors | Medium | Perception | Security point releases close the security gap; the reversal trigger is measured |
| The first public binary arrives 3 months† earlier than planned | High if DU-D10 is accepted | Every first-public-binary gate moves | Listed in DU-M1 |

## 11. Open questions and flagged claims

**Open questions.**
- **Q-DU-1.** Does SmartScreen reputation follow the validated publisher across certificate re-issuance and Artifact Signing's short-lived certificates? Black-box test before DU-M1.
- **Q-DU-2.** Does a running macOS broker keep launching its helpers after the bundle exchange of 6.5 (through its pinned directory descriptor), do TCC and Keychain grants carry across builds, and does the two-framework bundle pass notarisation? DU-M0b, with OI-M1.
- **Q-DU-3.** Do `apt autoremove`, unattended-upgrades and `dnf autoremove`/`dnf-automatic` install and remove `izcal-<channel>-<version>` packages as intended on Debian 12/13, Ubuntu 24.04/26.04 and Fedora? DU-M0b.
- **Q-DU-4.** Can a third-party APT repository use APT's phased-update field, and do clients honour it? Not established.
- **Q-DU-5.** The exact AppArmor profile syntax for `userns` on Ubuntu 24.04 and 26.04, from the shipped profiles (black-box inspection of `/etc/apparmor.d`).
- **Q-DU-6.** Can a full-trust MSIX process create its own AppContainer and restricted-token children with job objects? Before DU-M4.
- **Q-DU-7.** Notarisation latency, for the Nightly pipeline.
- **Q-DU-8.** On Windows 10 and 11 (NTFS, ReFS), does holding a child executable open with `FILE_SHARE_READ` stop rename and delete by other same-user processes, and can a running per-user image be renamed? The design relies on the first, not the second.
- **Q-DU-9.** Whether public Nightlies kept for SmartScreen reputation are "testing purposes" under CRA Art. 4(3), and whether a distribution-mediated install is "automatic" under Annex I Part I (2)(c). FI-1, before DU-M1.
- **Q-DU-10.** Whether a free OS upgrade the hardware cannot run counts as "additional costs" under Art. 13(10); recital 40 suggests it does. FI-1.
- **Q-DU-11.** Does Flatpak keep a running deployment's files when a new one is installed? DU-M4.
- **Q-DU-12.** Closed: the registry owner assigned G250–G259 on 24 October 2026; this spec uses G250–G255.
- **Q-DU-13.** Does the AppArmor `userns` profile attach to an `execveat` of an executable whose path was removed? DU-M0b.

**Flagged claims.**
- **Duebendorfer and Frei's 2009 figures** (97/85/53/24% at 21 days): the abstract has no numbers and the PDF was not reachable; they come from the note's reading, with a Google co-author and 2009 data (Medium).
- **Mathur et al.'s "about 40%"** is read from a figure that text extraction could not check (Medium).
- **The Debian wiki's "MUST NOT … trusted.gpg.d"** could not be machine-re-read (bot challenge); the design follows it regardless.
- **APT `sqv` rejecting SHA-1 bindings from 1 February 2026** comes from user and issue reports (Medium); the design avoids SHA-1 regardless.
- **Smart App Control's DLL coverage** is from a Microsoft engineer's personal blog (Medium).
- **Windows rename of a running image** is practice, not a documented contract (Q-DU-8); the design does not rely on it.
- **Chrome's Windows `new_chrome.exe` swap** is known only from third-party IT prose (Medium); not used.
- **Apple's three-release patch practice** is practice, not a published policy (Medium).
- **Artifact Signing's three-day certificate** is stated for the anti-malware profile only (Medium in general); its price is from a product-page snippet (Medium).
- **Firefox's 2-week cadence** is described in press as an experiment (Medium); Mozilla's own post states the cadence (High).
- **The Ubuntu Firefox snap's cold-start improvement** of about 50% by July 2022 is from secondary reports (Medium); the spec uses only the known cost.

## 12. Amendments to other specs (proposed, not applied)

This document edits no other file. The owner or the spec owners apply these, with revision lines.

| Document | Proposed amendment |
|---|---|
| Feature inventory audit, FI-2 | Record this spec's file name (`reports/Izcal distribution and updates.md`, not `Izcal distribution.md`) and mark C12 settled by FI-D10 as specified here |
| S1 Section 1 | Updater row: "verifies and applies into a new versioned build directory and the data root; runs in a headless update session when Izcal is closed (DU 6.8)". Broker row: add the `update-only` configuration and the session's process set. Add a **Launcher** row (count 0 after start). Linux spawner row: "launches children with `execveat` on descriptors opened at its own start from its build". |
| S1 2.1 | Add `hg-install` (L5), `hg-delta` (L2s), `hg-launch-main` and `hg-updater-main` (L6), and the tools crates of 6.1 |
| S1 2.2 registry | Register G250–G259 for this spec (G250–G255 used; G256–G259 unassigned) |
| S1 3.1 | The substrate gains the install-device trait and the OS-registration module, modelled in `hg-substrate-sim` |
| S1 7.1 | The build matrix carries the seven triples of 6.14; release builds link no Izcal-shipped shared library |
| S1 7.8 | Channels Nightly, Beta, Stable in Stage 1 and Extended in Stage 2 (6.6); feature cadence 4 weeks† with security-only point releases; the `rollout` role and its client bounds; update drills gain halt and revert drills (6.9); distributed packages reproducible by stages (6.16); symbols archived per build (6.15) |
| S1 8.3 | Infrastructure track: +2–3† engineers DU-M0a to DU-M3 and 0.5–1† release manager from DU-M1 (Section 9) |
| S1 13.2 Packaging | Versioned install layout; Linux one package per build plus a metapackage, Izcal's own APT/YUM repositories, unattended-upgrade configuration, AppArmor profile, RVA23 check; Windows per-user installer in Stage 1, MSI in Stage 2; `.dmg` Stage 1, `.pkg` Stage 2; **Microsoft Store and Mac App Store refused, Snap and AppImage not offered, for Stages 1–2**, with their reasons |
| S1 13.2 Updates | Staged rollout by signed, bounded ramps and fresh local positions; deltas; headless sessions; metered policy; binary transparency for browser releases |
| S1 13.2 Profiles | Forward-only migrations **plus retained copy-on-write pre-migration roots for one release**, deletion replay, and the one-migration-ahead open rule; every profile stamped with its newest schema |
| S1 13.2 Privacy and legal | The first public binary is the first public Nightly, about 3 months† before the first Stable (6.10) |
| Security scope SEC-D14 | Add: "Linux `.deb`/`.rpm` (FI-D10) are per-machine installs whose elevated component is the distribution's package manager, not an Izcal service; maintainer scripts are declarative only" |
| Security scope SEC-D4 | Replace the fallback "ships labelled as a preview without auto-update" with "ships as a preview under CRA Art. 4(3), with its notice and with automatic security updates" |
| Security scope 6.5 (updater threat model) | Replace "Open: macOS and Linux packaging interplay" with this spec's answer; add the `rollout` role and its bounds, the delta parser, the headless session, the chunked transfer, the repository key as root on Linux clients and the typed ledger |
| Security scope UPD-R1 | Ramps are a per-channel property in the channel inventory: the browser build and GPU blocklist are ramped; the phishing bundle, revocation and trust data and the extension blocklist are not; UPD-R5 switches apply at once |
| Security scope UPD-R2 | "Writes only the install directory" becomes "writes only `staging/`, `builds/` and the data root of its own install; on Linux packages, only the data root" |
| Security scope (new) **UPD-R6** | The `rollout` role: online, delegated by `releases` for ramp files only; security and emergency releases outside it; a client-enforced maximum ramp; holds expire unless `releases` delists; no un-halt onto a faster schedule; unknown targets refused |
| Security scope (new) **UPD-R7** | Every browser release target ships with an inclusion proof in Izcal's witnessed release log, verified by the client; publication waits for inclusion; moves "Binary transparency" from P2 to P1 for browser releases |
| Security scope SEC-13 | Add the SBOM (today only in S1 7.8) |
| Security scope SEC-15 | A dated proposal: distributed packages reproducible in stages (Linux packages and tarball, Windows installer, `.dmg`), comparing payload bytes before signing; UPD-R4's exception unchanged |
| Security scope SEC-16 | The key inventory of 6.10: the `rollout` key, the repository key and its successor, Authenticode and Developer ID identities, each with its age alert |
| Security scope SEC-17 | Add the delta-parser fuzz target, verify-then-parse, and the "rollout-key freeze", "rollout-key un-halt" and "unknown target in rollout" vectors |
| Shell 6.9 | Updates and recovery: replace "Update channel" with a read-only "Channel" row and a "Get Beta or Nightly…" link; add "Install security updates automatically" (CRA, governs builds only) and "Download updates on metered connections" |
| Shell 6.11 | The update row names security-only versus feature releases and links the advisory; "Later" shortens to 24 h† for actively exploited issues; the row shows running versus installed build; the two-stage out-of-date warning; "applies without a relaunch" covers data channels only |
| Shell 6.14 and about pages | `about:version` and About show the channel, the running and installed build, the updater that owns the binary and the support end date; Nightly and Beta show the Art. 4(3) preview notice; the remove flow of 6.12 in Settings |
| Shell SH-24 and SH-8 | Extend the three-version update test with a revert release and its deletion replay; SH-8's AI-off check runs across the revert |
| Phishing PH-D12, PH-6 | Record that the phishing bundle is not ramped and that browser-release checks ride its poll |
| Networking NW 6.17, NW-D17, NW-36 | NW 6.17: update checks ride PH-D12's grid (the wake-grid and battery-saver deferral still apply). NW-D17's endpoint list adds transparency-log checkpoints, delta targets, terminal-version metadata, ramp files and the APT and YUM repositories. The OS-adapter process gains connection cost as an OS source |
| Content blocking CB 6.9 (and NW 6.13's quotation of it) | "Verified by the broker's update client" becomes "verified by the updater process" (UPD-R2, G252); CB decides whether its bundle is ramped |
| Storage | The retained copy-on-write pre-migration root per migrated store, purged by every clearing operation |
| Coding standard A.4, Q-CS-1, CS-ST-18 | A.4: add `aarch64-unknown-linux-gnu`, `aarch64-pc-windows-msvc`, `x86_64-apple-darwin` and `riscv64gc-unknown-linux-gnu` to `deny.toml`'s targets (and CI). Q-CS-1: `hg-launch-main` and `hg-updater-main` link only `hg-substrate-os`. CS-ST-18: the install-device and OS-registration modules are platform modules in `hg-substrate-os` |
| Resource footprint FP-25 and FP 6.8 | FP-25: installed size counts at most two builds during a transition; per-format sizes reported. FP 6.8 / 8.2: a line for headless update sessions (CPU, energy, wall time per session†, sessions per day, 0 on battery saver) |
| Graphics GR-D21 | Applied at each release decision; the drop decision records the per-platform support statements it ends (CO-D5 (c)), the terminal-version record and any short security-only branch until a stated date |
| FI-1 (compliance) | The per-platform support statement fixed at release, as CO-D5 (c) and DU-D15 now say identically; CSAF; the Art. 4(3) design for previews; Q-DU-9 and Q-DU-10; the CRA opt-out's scope (builds only) |
| FI-3 (enterprise policy) | Policy inputs the updater honours as local inputs that only narrow signed metadata: version-prefix pin, channel, check interval, automatic install off; the headless session reads policy; the Stage 2 elevated updater |
| FI-4 (OS integration) | OI-D3's layout is adopted (6.2, 6.5), and the update column of OI 6.2's macOS row becomes "a complete signed bundle exchanged atomically, holding the new and the running build's helper frameworks"; OI-D10's journal gains four record kinds (update scheduler entry, keychain item, OS cache directory, crash-dump directory); Q-OI-12 is answered in 6.6 and Q-OI-14 in 6.11; the launcher's hand-off message, which never targets an `update-only` broker; the session-stop request; per-channel default-browser identity; every OS registration goes through the install ledger |
| FI-11 (crash and telemetry) | Crash report header carries the channel and `BuildId`, and whether the build is the installed or a superseded one; symbols per build from 6.15 |

## 13. Inputs to M1 phase E and to M5

**M1 phase E (process skeletons and launcher; the `BuildId` check) = DU-M0a.**
1. **Launch by build-relative handle for Izcal kinds.** The OS substrate's launch API for Izcal process kinds takes a handle to a binary inside the caller's own build, obtained at broker start-up (G251); there is no launch-by-path or by-name for them. On Linux the spawner opens `O_PATH` descriptors at its own start and launches with `execveat(…, AT_EMPTY_PATH)`; on macOS it is a pinned directory descriptor on the helper framework, resolved with `fcntl(F_GETPATH)` before each `posix_spawn`; on Windows the handle is a path resolved once inside the build plus an open handle on the executable with `FILE_SHARE_READ` only, held from broker start (`CreateProcessAsUserW` takes a path; today's probe passes `current_exe()`). Launching non-Izcal programs (protocol handlers, download Open, native hosts, the protected-media host) is a separate broker-only API, not needed in phase E.
2. **Own build, resolved once.** The broker determines its build at start-up from its own running image, not from `argv[0]`, `PATH` or `current`, and passes it to the spawner.
3. **Install-device trait and simulation model.** `hg-substrate-os` gains the install-device trait (atomic rename, shared and exclusive locks, delete-tree, read-only open, open-by-inventory); `hg-substrate-sim` models it, so DST can replace a child binary or a whole build between two launches. The substrate's `HandleKind` set gains a file kind only when a phase needs it; phase E needs the trait and the model.
4. **Liveness hook.** The broker accepts the build-lock handle from its launcher and holds it for its lifetime; phase E needs only the hook, and `hg-install` fills in the lock protocol at DU-M1.
5. **`BuildId` semantics.** Keep the strict check. Define `BuildId` (a `u64` today) as a **truncated digest of the build inputs, fixed before compilation**: source commit, `specs.lock` and `downloads.lock`, toolchain pin, build profile, target triple, channel, release version and the `.hgidl` schema hash. It is **compiled into every binary** of the build, so every process of one build (renderer and jitless renderer alike) reports the same value, and a child from another build cannot report the parent's. `build.inventory` is a separate, signed file list whose header carries the same `BuildId`; the updater checks that they agree. A revert release, rebuilt from the previous source under a higher version, gets a new `BuildId`. T1 tests: every binary of one build reports the same `BuildId`, and builds from different inputs never collide in the 64 bits over the archive.
6. **Mismatch behaviour.** On mismatch the broker enters the **restart-required state** (no retry loop) and records the event; a DST scenario replaces a child binary between two launches and checks DU-3.
7. **No change to `process.hgidl`.** The launcher's hand-off to a running broker is FI-4's own small, versioned message, not a `.hgidl` protocol, because it crosses builds.
8. **Recording header.** FI-11's channel identity covers distribution's needs; recordings carry the `BuildId` as M1's exit already requires.

**M5 (Windows sandbox, process launch and IPC backend), with DU-M0b's measurements.**
1. `CreateProcessAsUserW` launches Izcal children by absolute path inside the broker's own build, with each executable held open as in phase E item 1; the AppContainer and file ACLs are granted on that build directory.
2. On macOS, the broker is `Izcal.app`'s bundle executable (OI-D3); the jitless helper and every helper executable are resolved inside the running build's versioned helper framework through the directory descriptor the broker pinned at start, never through a path re-resolved from the bundle root.
3. On Linux, the AppArmor profile's paths match `/opt/izcal/*/builds/*/` and the metapackage's stub; Q-DU-13 is measured; the tarball's reduced tier is SEC 6.7's.
4. The Windows per-user layout `%LocalAppData%\Izcal\<channel>\builds\<build>\` is the path the sandbox's DLL-loading policy (CIG after Izcal's own DLLs load) is measured against.
5. The broker's full pre-open of its build inventory is measured for descriptor count and start-up cost on each OS (FP).

## Appendix A. Ledger rows closed

| Row | Feature | Where this spec answers it |
|---|---|---|
| BF-18.7 | Beta / preview channels | 6.6 |
| PO-1.1 | Windows consumer installer [SB] | 6.4 |
| PO-1.3 | MSIX / Microsoft Store | 6.4 (Store refused by policy 10.2.1; MSIX conditional, Q-DU-6) |
| PO-1.4 | macOS DMG and pkg [SB] | 6.5 |
| PO-1.5 | Mac App Store | 6.5 (refused: 2.4.5, 2.5.6) |
| PO-1.6 | Linux deb and rpm [SB] | 6.3 |
| PO-1.8 | Snap | 6.3 (not offered in Stages 1–2) |
| PO-1.9 | Tarball / portable | 6.3 |
| PO-1.10 | Side-by-side channels | 6.6 |
| PO-1.11 | Initial preferences | 6.13 (Stage 2) |
| PO-1.12 | Install-time options | 6.4, 6.13 |
| PO-1.13 | Uninstall, repair, leftovers [SB] | 6.4, 6.12 |
| PO-2.2 | SmartScreen reputation [SB] | 6.10 |
| PO-2.6 | Linux repository signing [SB] | 6.3 |
| PO-3.1 | Background auto-updater [SB] | 6.8; 6.3 for packages |
| PO-3.3 | Delta updates | 6.7 |
| PO-3.4 | Release channels [SB] | 6.6 |
| PO-3.5 | Release cadence | 6.6 |
| PO-3.6 | Extended support channel | 6.6 (Stage 2) |
| PO-3.7 | Staged rollout | 6.9 |
| PO-3.8 | Version pinning and rollback | 6.9 (revert); pinning as FI-3 inputs (Section 12) |
| PO-3.13 | Weekly security updates | 6.6, 6.9 (security-only point releases inside SEC-1's bounds, DU-23) |
| PO-3.14 | Release notes | 6.15 |
| PO-4.9 | Extension blocklist | 6.15 (format; Stage 2 with EX) |
| PO-11.2 | Windows on ARM64 | 6.4, 6.14 |
| PO-11.5 | Linux ARM64 | 6.14 |
| PO-16.9 | Release engineering and on-call | 6.15, Section 9 |

## Appendix B. Review disposition

**Fact-check** (`factcheck.md`): 80 links checked (79 HTTP 200; EUR-Lex blocks scripts, so the CRA was read from the Official Journal copy at the Publications Office), every CRA quote confirmed word for word against the Official Journal, every internal ID checked; **38 findings (0 blockers, 2 major, 36 minor), all applied.**

| Finding | Disposition |
|---|---|
| F1 (major) crash-report consent | Applied: D-S4's opt-out with a per-crash prompt is stated wherever gates use crash reports (Summary, 4, 6.9, 6.15) |
| F2 (major) shared libraries under `execveat` | Applied: Izcal's executables link no Izcal-shipped shared library (T1 check), and the broker pre-opens its whole inventory (6.2 rule 3); DU-1 states how a path that cannot pass is handled |
| F3–F7 Store date, 10.2.9 reading, 11 countries, Artifact Signing lifetime and validation, submission wording | Applied (Summary, 3.1, 6.10) |
| F8–F17 citations: SAC DLLs, running-image rename, 8.5M source, Courgette quote, bsdiff memory, zstd corpus, snap 50%, Armagadd-on timing, RSR "(c)", Keystone version | Applied: re-cited, graded Medium or removed (1, 2, 3.1, 5, 6.7, 11) |
| F18–F20 Uptane status and outside copy, Flame link, LWN link | Applied (2, references) |
| F21 cadence wording | Applied (2, 6.6) |
| F22 2.5.6 entitlement scope | Applied (Summary, 3.1) |
| F23 Art. 13(9) "whichever is longer", no † | Applied (Summary, 6.3, 6.11, prefix note) |
| F24 recital 40 | Applied (6.11, Q-DU-10) |
| F25 IR 2025/2392 | Applied (6.11); the CRA half of old Q-DU-9 and its risk row are closed |
| F26 Bugzilla 1888601 wording | Applied (5, 7) |
| F27 five years attributed to Art. 13(8) | Applied (1) |
| F28 "no account" scoped to Stage 1 | Applied (Summary, 2) |
| F29 SBOM in S1 7.8 | Applied (1, 6.11, Section 12 SEC-13 row) |
| F30 "not a fork server" attribution | Applied (4, 6.2) |
| F31 four missing triples | Applied (6.14, Section 12) |
| F32 halt success wording | Applied (1) |
| F33 DU-8 worst case | Applied: delay 30 min†, target 8 h† |
| F34 out-of-date stages | Applied (6.11) |
| F35 key inventory wording | Applied (6.10, Section 12) |
| F36 DU-10 fallback | Applied: DU-10 has its own failure action; SEC-D4's fallback is amended (Section 12) |
| F37 riscv64 RVA23 | Applied (3.1, 6.3, 6.14, DU-12) |
| F38 Zucchini figure | Applied: not used (2, 7) |

**Design critique** (`critique.md`): **40 findings (3 blockers, 17 major, 20 minor), all applied**, with FI-D10 kept as the owner decided.

| Finding | Disposition |
|---|---|
| C1 (blocker) `BuildId` circular | Applied: digest of build inputs, compiled in; separate signed inventory (13 item 5). One difference from the proposal: a revert is rebuilt and gets a new `BuildId`, following C11 (4), rather than keeping the old one |
| C2 (blocker) packages without automatic security updates | Applied: unattended-upgrade and `dnf-automatic` configuration, Izcal's system timer, the broker's update row, the Settings switch on packages, DU-1 and DU-15 package rows, Q-DU-9 to FI-1 (6.3) |
| C3 (blocker) `rollout` key can freeze or un-halt | Applied: the five client bounds, delegation from `releases`, ramp-file expiry, SEC-17 vectors, corrected risk row (6.9, 10, UPD-R6) |
| C4 Linux locks, autoremove, partial pre-open, AppArmor | Applied: locks in `$XDG_RUNTIME_DIR`, full pre-open, Q-DU-13 with the stub fallback, DU-1 paths, DU-2 scope (6.2, 6.3) |
| C5 packages are per-machine | Applied: recorded, SEC-D14 amendment, declarative maintainer scripts and inotify, key custody and successor, inventory check for packages (6.3) |
| C6 layout races and launcher update | Applied: `<state>/live/<build>.lock`, shared and exclusive locks, lock before releasing `current`, repair via staging, G253 restated, launcher update rule (6.2) |
| C7 macOS nested bundles | Applied in 0.2 as sibling bundles; **superseded in 0.3** by OI-D3's layout with this spec's atomic bundle exchange, which keeps C7's point (nothing writes into a sealed bundle) and its DU-M0b checks (6.5) |
| C8 headless session | Applied: named process set, `update-only` broker configuration, collision rule, FP line (6.8) |
| C9 bucket as quasi-identifier | Applied: fresh per-release HMAC position, jitter, timing test in DU-5, storage sentence fixed (6.9) |
| C10 ramp on protective channels | Applied: ramps per channel (6.9, UPD-R1 amendment) |
| C11 revert conflicts | Applied: one-migration-ahead rule, retained copy-on-write roots purged by clearing, deletion replay, cost in pages, rebuild with a higher version (6.6, 6.9, DU-21) |
| C12 Beta gate vs security releases | Applied: gates per release class, security time budget, DU-23 (6.9) |
| C13 reproducibility vs UPD-R4 | Applied: UPD-R4 exception, pre-signing payload comparison, staged dated proposal (6.16) |
| C14 opt-out scope, SEC-D4 fallback, previews | Applied: builds only, SEC-D4 amendment, Art. 4(3) design (verified in the Official Journal: it is Art. 4(3), with recital 37) (6.6, 6.11) |
| C15 data channels' location | Applied: one per-user, per-channel data root; UPD-R2 amendment (6.2, 6.3) |
| C16 G251 too broad | Applied: scoped to Izcal kinds, a broker-only API for others, the Windows held handle (6.1, 13) |
| C17 G3 and CS-ST-18 | Applied: install-device trait and OS-registration module in the substrate, simulation model, `hg-delta` in L2s, Q-CS-1 (6.1, 13) |
| C18 lead time pulls gates forward; DU-M0 placement | Applied: DU-M0a/DU-M0b split, first-public-binary gates in DU-M1 (6.10, 9) |
| C19 targets dated now; end-date contradiction | Applied: targets as a rule at each release decision; in 0.4, the per-platform support statement fixed at release (CO-D5 (c)); Intel macOS decided at DU-M3 (6.11, 6.14) |
| C20 staffing not derived | Applied: per-crate estimate, run cost, OS tools for `.dmg` and `.msi`, HSM signing (6.1, 9) |
| C21 TUF mechanics | Applied (6.9) |
| C22 shell "Update channel" | Applied (6.6, 6.8, Section 12) |
| C23 NW 6.17 polling | Applied: NW amendment and jitter (6.8, Section 12) |
| C24 NW and CB amendments | Applied (Section 12) |
| C25 large payloads | Applied: chunked shared-memory transfer and resumable ledger (6.8) |
| C26 verify-then-parse | Applied (6.7, DU-6) |
| C27 log availability | Applied: proof ships with the target; publication waits with a drilled bound (6.16) |
| C28 crash-report wording | Applied (with F1) |
| C29 AG-0, initial preferences, model files | Applied (6.8, 6.13, SH-8 amendment) |
| C30 symbols and replay of reaped builds | Applied (6.15) |
| C31 macOS LaunchAgent, DU-13 scope, DU-20 baseline | Applied (6.5, 6.12, DU-13, DU-20) |
| C32 riscv64 baseline | Applied (with F37) |
| C33 internal inconsistencies | Applied (6.14, 9, 6.10) |
| C34 OD-7 citations | Applied (2, 7, references) |
| C35 Snap reasoning; third-party manifests | Applied (Summary, 6.3, DU-D18) |
| C36 tarball under SEC-8 | Applied (6.3) |
| C37 10-year pools; archive metadata growth | Applied (6.3, 6.11) |
| C38 G-rule block; launcher link rule | Applied: the registry owner assigned G250–G259; rules numbered G250–G255; G255 is the launcher link rule |
| C39 DU-10 per release | Applied (6.10, DU-10) |
| C40 ledger location and typing | Applied (6.12) |

[FI]: Izcal%20feature%20inventory%20audit.md
[H2]: HGBA%202%20browser%20architecture%20spec.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[SEC]: Izcal%20security%20scope.md
[SH]: Izcal%20shell.md
[GR]: Izcal%20graphics%20and%20rendering.md
[FP]: Izcal%20resource%20footprint.md
[JS]: Izcal%20JavaScript%20engine.md
[CS]: Izcal%20coding%20standard.md
[DUH]: ../research_notes/Izcal%20distribution%20and%20updates/history_postmortems.md
[DUP]: ../research_notes/Izcal%20distribution%20and%20updates/platform_packaging_signing.md
[DUM]: ../research_notes/Izcal%20distribution%20and%20updates/update_mechanics_release.md
[DUL]: ../research_notes/Izcal%20distribution%20and%20updates/lifecycle_ux_regulation.md
[CZ]: https://chromium.googlesource.com/chromium/src/+/main/docs/linux/zygote.md
[BZ1750250]: https://bugzilla.mozilla.org/show_bug.cgi?id=1750250
[BZ2010328]: https://bugzilla.mozilla.org/show_bug.cgi?id=2010328
[BZ1888601]: https://bugzilla.mozilla.org/show_bug.cgi?id=1888601
[CS291]: https://www.crowdstrike.com/wp-content/uploads/2024/08/Channel-File-291-Incident-Root-Cause-Analysis-08.06.2024.pdf
[MS-CS]: https://blogs.microsoft.com/blog/2024/07/20/helping-our-customers-through-the-crowdstrike-outage/
[CRA]: https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32024R2847
[SSR]: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation
[SEQ]: https://developer.apple.com/news/?id=saqachfa
[MSP]: https://learn.microsoft.com/en-us/windows/apps/publish/store-policies
[ARG]: https://developer.apple.com/app-store/review/guidelines/
[DF09]: https://research.google/pubs/why-silent-updates-boost-security/
[MAT18]: https://arxiv.org/abs/1805.04594
[WASH14]: https://rickwash.com/papers/conference/betrayed-by-updates.html
[CHU]: https://support.google.com/chrome/answer/95319?hl=en
[DEBP]: https://www.debian.org/doc/debian-policy/ch-files.html
[FX67]: https://www.firefox.com/en-US/firefox/67.0/releasenotes/?redirect_source=mozilla-org
[ARMA]: https://hacks.mozilla.org/2019/05/technical-details-on-the-recent-firefox-add-on-outage/
[ARMA2]: https://hacks.mozilla.org/2019/07/add-ons-outage-post-mortem-result/
[BSD]: https://www.daemonology.net/papers/bsdiff.pdf
[OMAHA3]: https://opensource.googleblog.com/2011/09/omaha-3-modernizing-automatic-software.html
[CHR09]: https://blog.chromium.org/2009/01/google-chrome-installation-and-updates.html
[COUR]: https://blog.chromium.org/2009/07/smaller-is-faster-and-safer-too.html
[TUFS]: https://theupdateframework.github.io/specification/latest/
[PEP458]: https://peps.python.org/pep-0458/
[MFSA1383]: https://www.mozilla.org/en-US/security/advisories/mfsa2013-83/
[FLAME]: https://learn.microsoft.com/en-us/security-updates/securityadvisories/2012/2718704
[TORDB]: https://blog.torproject.org/deterministic-builds-part-one-cyberwar-and-global-compromise/
[SPARK]: https://sparkle-project.org/documentation/
[UPT]: https://uptane.org/docs/latest/standard/uptane-standard
[MEDOC]: https://blog.talosintelligence.com/the-medoc-connection/
[CCL]: https://blog.talosintelligence.com/avast-distributes-malware/
[MBT]: https://wiki.mozilla.org/Security/Binary_Transparency
[KEYS]: https://tidbits.com/2019/09/25/google-keystone-update-damages-file-system-on-sip-disabled-macs/
[SNAPPERF]: https://canonical.com/blog/how-are-we-improving-firefox-snap-performance-part-1
[SUNSPOT]: https://www.crowdstrike.com/en-us/blog/sunspot-malware-technical-analysis/
[FXBG]: https://firefox-source-docs.mozilla.org/toolkit/mozapps/update/docs/BackgroundUpdates.html
[CHR2W]: https://developer.chrome.com/blog/chrome-two-week-release
[EDGE2W]: https://blogs.windows.com/msedgedev/2026/06/11/faster-updates-enterprise-friendly-schedule-the-new-microsoft-edge-release-cycle/
[FX2W]: https://blog.mozilla.org/sumo/2026/08/19/firefox-new-release-cadence-and-what-to-expect/
[RSR]: https://www.macrumors.com/2023/07/10/apple-pulls-ios-16-5-1-macos-13-4-1-rsrs/
[3CX]: https://cloud.google.com/blog/topics/threat-intelligence/3cx-software-supply-chain-compromise
[XZ]: https://www.openwall.com/lists/oss-security/2024/03/29/4
[NPP]: https://notepad-plus-plus.org/news/hijacked-incident-info-update/
[DCT]: https://www.docker.com/blog/retiring-docker-content-trust/
[CABCS]: https://cabforum.org/working-groups/code-signing/requirements/
[ARTS]: https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart
[ARTSF]: https://learn.microsoft.com/en-us/azure/artifact-signing/faq
[SAC]: https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control
[MFE]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexa
[ADID]: https://developer.apple.com/help/account/certificates/create-developer-id-certificates/
[NOTAR]: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
[TN2206]: https://developer.apple.com/library/archive/technotes/tn2206/_index.html
[DEB3P]: https://wiki.debian.org/DebianRepository/UseThirdParty
[APT3]: https://lwn.net/Articles/1017315/
[UBKEY]: https://discourse.ubuntu.com/t/new-requirements-for-apt-repository-signing-in-24-04/42854
[RHSHA1]: https://www.redhat.com/en/blog/rhel-security-sha-1-package-signatures-distrusted-rhel-9
[RPM6]: https://fedoraproject.org/wiki/Changes/RPM-6.0
[REN]: https://man7.org/linux/man-pages/man2/rename.2.html
[EXA]: https://man7.org/linux/man-pages/man2/execveat.2.html
[UB2404]: https://documentation.ubuntu.com/release-notes/24.04/
[UBUSERNS]: https://discourse.ubuntu.com/t/understanding-apparmor-user-namespace-restriction/58007
[DEB13]: https://www.debian.org/News/2025/20250809
[A64EC]: https://learn.microsoft.com/en-us/windows/arm/arm64ec
[CHRREQ]: https://support.google.com/chrome/a/answer/7100626
[W10ESU]: https://www.microsoft.com/en-us/windows/extended-security-updates
[EDGEOS]: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-supported-operating-systems
[APPPH]: https://developer.apple.com/help/app-store-connect/update-your-app/release-a-version-update-in-phases/
[PLAY]: https://support.google.com/googleplay/android-developer/answer/6346149?hl=en
[BALROG]: https://hearsum.ca/posts/streamlining-throttled-rollout-of-firefox-releases/
[CHRP4]: https://chromium.googlesource.com/chromium/src/+/main/docs/updater/protocol_4.md
[ZSTD]: https://github.com/facebook/zstd/releases/tag/v1.4.5
[DESKTOP]: https://specifications.freedesktop.org/desktop-entry/latest/recognized-keys.html
[UNKEY]: https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key
[PIN]: https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/pin-to-taskbar
[FXPN]: https://www.mozilla.org/en-US/privacy/firefox/
[CHRIP]: https://support.google.com/chrome/a/answer/187948?hl=en
[CSRB]: https://www.cisa.gov/sites/default/files/2025-03/CSRBReviewOfTheSummer2023MEOIntrusion508.pdf
[RFC9580]: https://www.rfc-editor.org/rfc/rfc9580
[PH]: Izcal%20phishing%20protection.md
[NW]: Izcal%20networking.md
[SG]: Izcal%20storage.md
[BSDP]: https://www.daemonology.net/bsdiff/
[COURD]: https://www.chromium.org/developers/design-documents/software-updates-courgette/
[SACL]: https://textslashplain.com/2026/04/28/smart-app-control/
[UB2510]: https://documentation.ubuntu.com/release-notes/25.10/
[IR2392]: http://publications.europa.eu/resource/celex/32025R2392
