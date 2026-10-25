# Izcal OS integration

*First-principles specification of how Izcal meets the operating system: registration as a browser and the default-browser status on Windows, macOS and Linux; URL-scheme and file-type handlers in both directions; the launcher, single instance and the second-launch hand-off; command-line switches; the OS share sheet; taskbar, dock, jump lists and badges; the OS side of notifications, media controls, wake locks and power events, native file dialogs, window management, per-monitor DPI and multi-monitor placement; background mode and launch at login; OS scripting hooks; OS speech voices; the Linux desktop (D-Bus, portals, Wayland and X11); one OS-registration mechanism shared with installed web apps; and the process boundary for all of it. Date: 24 October 2026. Status: draft 0.4. Research notes: `research_notes/Izcal OS integration/` (`history_postmortems.md`, `windows_platform.md`, `macos_platform.md`, `linux_freedesktop.md`; phase 8: `factcheck.md`, `critique.md`). Work item: **FI-4** of the feature inventory audit. Relates to: HGBA-2 2.4.4 [M-1, P1, P6, I-1, I-2]; HGBA-3 3.3.3 [B1, B2, B9, B10, DET-1, AG-0]; Stage 1 design 0.14.6 [Sections 1, 2.1–2.2, 3.1, 3.4, 4.1, 6, 8.1]; security scope [TB-10, SH-R6, SH-R8, SEC-20, AGT-R8, IPC-R15, IPC-R17, 6.3a, 6.7]; coding standard [CS-D8, CS-AS-4, tier P]; shell [SH-D3, 6.1–6.4, 6.10–6.13]; shell UI design [R19, UP3]; accessibility [AX-D12, AX-D18, 6.11]; editing [ED-D5, ED-D10–ED-D13, Q-ED-17]; images and media [6.12, Q-MD-16]; resource footprint [FP-1, FP-D4, 6.8, 8.2]; graphics [GR-D17, GR-D15, GR-D28]; extensions [6.x manifest keys, C17]; DOM [6.18]; networking [NW 6.1, the OS-adapter process]; feature inventory audit [FI-4, FI-D5, FI-D8, FI-D10, FI-D14, FI-D17, Section 8 points 2 and 3].*

*Revision 0.4 (25 October 2026): owner decisions on the starred items. **Accepted as recommended:** OI-D2, OI-D3, OI-D5, OI-D12, OI-D17, OI-D20 and OI-D25 (the fifth shared I-1 exception is owner-approved), and every un-starred OI decision; each row is marked. **Changed: OI-D18**, from "standard events only" to **full scripting behind debug mode and per-app consent**: 6.14 is redesigned (debug mode off in every build, turned on only by the person in Settings with a fresh OS authentication and a visible chip; an in-Izcal grant per calling app, expiring after 8 hours†; window and tab control in the person's profile, page access only in a fresh ephemeral container; private windows excluded; an enterprise policy forces it off), target OI-27 is added, and the amendments table gains rows for AGT-R8, SEC-20, DT 6.3, EP-D11 and the shell. The Stage 2 read-only dictionary is folded into the new design. G230–G239 unchanged.*

*Revision 0.3 (25 October 2026): owner decisions of 25 October 2026 recorded. **OS inputs, "both, split by source"**: `OsPort` (broker-only, scripted in the simulator) carries what the OS delivers to the broker itself — launches, default-browser status, settings, locale and time zone, hardware, displays, power and media keys; **enterprise policy does not go through `OsPort`**: it is parsed in the sandboxed policy adapter of the enterprise policy spec (EP-D7) and reaches the broker as typed `.hgidl` snapshots carrying a 32-byte `PolicyDigest`; both paths are recorded and renderers see neither (6.1, OI-D1, Appendix C). **OI-D6 is accepted as recommended**: Izcal's own `hg-dbus` with the gate, with `zbus` for the accessibility bridge only if `hg-dbus`'s core misses AX-M0. **Cross-spec**: the distribution spec (FI-2) set a 4-week Stable cadence, moving to 2 weeks once the release pipeline runs in under 3 working days for two quarters, and signed public Nightlies at least 3 months before Stable, after an EU legal entity exists; OI-2's skew window and the hand-off's support rule are restated against it. G230–G239 unchanged.*

*Revision 0.2 (24 October 2026): phase 8 applied. An independent fact-check checked 92 claims (67 correct, 16 partly correct, 3 wrong, 6 unverifiable) and 64 links (56 external, all resolving; 8 relative, all present; none pointing at source code, one redirecting into a repository blob view) and listed 5 must-fix and 13 recommended items; all are applied. An independent design critique raised 40 findings (1 Critical, 12 High, 21 Medium, 6 Low); all are applied, nine in a different form than proposed, none rejected (Appendix A). What changed: **the OS-integration service gets no D-Bus socket**: the broker's `hg-dbus` **gate** forwards only the (destination, interface, member) tuples each role declares (C1); `OsPort` carries encoded frames like `Ipc`, its typed layer sits above, and the per-OS halves are inventoried modules of `hg-substrate-os` (C2); an **OS-input inventory** adds `Policy`, `Locale`, `TimeZone` and `Hardware` and makes `Settings` a superset of AX 6.11's sources (C3); on Windows the broker's own hidden top-level window receives the OS broadcasts and the UI host's per-window reports are tagged and validated (C4); **production recordings carry only what a tab consumed**, the full stream belongs to simulation (C5); the service splits into a **desktop role** (a fifth shared exception, an owner decision) and a **speech role** per (site, container) (C6); a **TX-16 amendment** lists the OS display surfaces (C7); the broker's parsing exception is recorded and `DBusActivatable` waits for Stage 2 (C8); **inbound URLs are an allowlist** and remote paths are never opened silently (C9); bursts **queue** instead of being refused (C10); a **per-OS install and launch layout** replaces the assumption that FI-D10 covers all three OSes (C11); long-lived Linux subscriptions stay on the broker's gate and the startup theme has a bounded wait (C12); Q-OI-1 moves to SH-M0 with a third option (C13); `hg-dbus` gains its server side and lands at AX-M0 (C14); interface descriptions are hand-written from prose or black-box introspection (C15); targets are re-stated as tests (C30), and the launcher budgets are made consistent (fact-check). Fact-check corrections: the macOS 26 background-task prompt applies to the Stage 2 headless broker, not to login items; the UCPD key list is attributed to its real sources; the network OS-adapter row of Stage 1 §1 gets an amendment; a code-review citation was removed from the Windows note.*

*Revision 0.1 (24 October 2026): first draft, written by the synthesis author from the four research notes and the corpus at 3b183d1. No existing report was edited; every change to another document is in Section 9.5 as a proposal.*

[HP]: ../research_notes/Izcal%20OS%20integration/history_postmortems.md
[WP]: ../research_notes/Izcal%20OS%20integration/windows_platform.md
[MP]: ../research_notes/Izcal%20OS%20integration/macos_platform.md
[LX]: ../research_notes/Izcal%20OS%20integration/linux_freedesktop.md
[FIA]: Izcal%20feature%20inventory%20audit.md
[S1]: HGBA%20Stage%201%20architecture%20design.md
[SH]: Izcal%20shell.md

A browser is an application that other applications send things to. Every link clicked in a mail client, every `.html` file double-clicked in a file manager, every `izcal https://…` typed in a terminal and every notification click arrives from outside the engine, from an operating system that has spent twenty years changing the rules: Windows removed every programmatic way to become the default browser and now guards the choice with a kernel filter driver ([Microsoft Learn][MS-DEF]); macOS asks the person before any app changes the browser, and, by one expert's report, since macOS 26.4 before any file-type default changes ([Scripting OS X][SOX-264]; Section 11); Linux has a written standard for defaults and no portal for them ([LX 1.6, 1.12][LX]). The same edge is a recurring security hole: hostile URLs reaching handlers became argument injection or handler abuse in 2007, 2018, 2021, 2022 and 2024 ([HP 8][HP]), and external-protocol prompts leak which apps a person has installed ([Fingerprint][FP-SF]). Izcal's corpus specifies an engine and a security model in depth, but its OS edge has no owner: the extensions spec points at an "OS-integration design" that does not exist (C17), the audit finds 22 rows without a complete design (9 missing, 13 partial), four of them ship blockers ([FIA][FIA] FI-4), and two of its M1-exit inputs (OS-sourced inputs as recorded broker inputs; a versioned launch hand-off, because `.hgidl` is same-build only) are this spec's to define.

**The central idea: the operating system is one more untrusted, recorded peer.** Everything the OS tells the browser (a launch request, a default-browser status, a setting, a display change, a sleep notice, a media key, a notification click) enters the broker as **one typed, versioned input through one substrate interface**, so the simulator records and replays it exactly like a network response. Everything the browser asks of the OS is **one typed command** whose answer comes back the same way. The code that touches OS APIs is split by what it has to trust: the broker keeps only what the OS forces into the app's or window's process, plus a header-only **D-Bus gate** that is the only holder of a bus connection; a new sandboxed **OS-integration service** parses what other local programs send, with no bus socket of its own; and on Windows a medium-integrity **OS shell helper** hosts the dialogs that load third-party shell extensions. The one channel that crosses Izcal versions, the second launch handing its URLs to a running browser, gets its own tiny, versioned, Kani-proved wire format.

The decisions that matter most: **(1)** OS inputs enter through one broker substrate trait, `OsPort`, recorded at that boundary in simulation, with production tab recordings carrying only what the tab consumed (OI-D1, OI-D26, an M1-exit input); **(2)** the launch hand-off is `hg-launch-wire`, a fixed TLV format with a critical bit, the only cross-build message Izcal accepts, sent by a stable launcher to the broker in its versioned install directory (OI-D2, OI-D3, an M1-exit input); **(3)** Izcal becomes the default only by asking through the OS's own surface, reads its status as a recorded input and never writes protected state (OI-D7); **(4)** every registered handler command ends in an end-of-options marker, inbound URLs are an allowlist, and every outbound launch is broker-mediated, with outcomes the page cannot tell apart (OI-D8, OI-D9); **(5)** Izcal writes its own D-Bus implementation, `hg-dbus`, from the D-Bus specification, with a gate so that no sandboxed process holds a bus socket, and the accessibility, editing, media and shell specs move off `zbus` (OI-D6, an owner decision); **(6)** the OS-integration service's desktop role is a fifth shared, site-less exception to I-1, receiving only bounded, broker-formatted strings the OS publishes anyway (OI-D25, an owner decision); and **(7)** on Wayland compositors that refuse server-side decorations, GNOME's included, the shell draws its own window frame, which amends the shell's "the OS draws the frame" rule (OI-D20, an owner decision).

## 1. The problem and who has it

| Who | Task | What goes wrong today |
|---|---|---|
| **A person choosing a browser** | Make Izcal the default, once, and have it stay that way | Windows resets defaults on upgrade and, by Mozilla's July 2026 report, on a Windows Backup restore ([HP 1.7, 1.20][HP]); shell surfaces open `microsoft-edge:` links outside any browser's reach outside the EEA ([HP 1.11][HP]); browsers nag ([HP 1.9][HP]) |
| **A person clicking links in other apps** | The link opens in the right browser, the right profile and the window in front | `-new-tab` went to whichever instance started first and ignored the profile ([Bugzilla 1080319](https://bugzilla.mozilla.org/show_bug.cgi?id=1080319)); profile locks on network home directories strand users ([HP 4.5][HP]); a Wayland compositor may refuse to raise a window without an activation token ([LX 2.5][LX]) |
| **A person on a laptop** | Close the browser and have it stop; let video keep the screen awake; let the machine sleep | "Edge starts itself" ([HP 9.5][HP]); a background browser keeps the machine awake; system and execution power requests end 5 minutes after the sleep timeout on battery on Modern Standby ([WP 7.1][WP]) |
| **A person sharing or playing media** | Share a page to another app; pause a video from the keyboard | Desktop Web Share is uneven ([HP 5.2][HP]); media keys route to whatever app last played ([MP 5.4][MP]) |
| **A person with mixed displays** | Drag a window from a 100% to a 150% monitor; reopen windows where they were | Wrong-scale frames and cursor jumps without `WM_DPICHANGED`'s suggested rectangle ([WP 10.2][WP]); Wayland lets no client place its own window ([LX 7.16][LX]) |
| **An attacker** | Turn a URL into a command line; probe installed apps; launch a dangerous handler | `firefoxurl:` (2007), Electron (2018), `ms-officecmd:` (2021), Follina (2022), `mhtml:` (2024) ([HP 8.2–8.6][HP]); scheme flooding gave a 32-bit identifier in seconds ([Fingerprint][FP-SF]) |
| **Izcal's engineers** | Replay a bug that started with a link from another app; test a launch storm; ship an update while the browser runs | The OS edge is outside the simulator; the IPC refuses a child from another build (`ExitReason::BuildMismatch`), so a new launcher cannot talk to an old running broker over `.hgidl` ([FIA][FIA] §8.3) |
| **Installed web apps (FI-9)** | Get their own shortcut, handlers, badge, notification identity | The full PWA platform is planned (FI-D8), but nothing owns the OS registrations it rests on |

**What the corpus already decides and constrains.**
- **Process map ([S1][S1] §1).** The broker is unsandboxed and owns windows, input routing, the profile and policy. On Windows the always-on **UI host** owns content windows, TSF and the UIA adapters, never prompt windows (SH-D3). An **OS-adapter process** exists for OS calls that parse network data, one per container (NW 6.1), while configuration reads stay in the broker; S1 §1's row still describes the older role ("1 per OS source", reading proxy, DNS and certificate-store configuration), and 9.5 proposes aligning it with NW 6.1. The Linux spawner owns the sandboxed spawn path.
- **Substrate ([S1][S1] 3.1).** The `Substrate` struct grows by milestone (network and storage join later); as built at M1 phase D2 it has `clock`, `entropy`, `exec`, `mem` and `ipc` (`crates/hg-substrate/src/lib.rs`).
- **IPC ([S1][S1] 4.1).** No self-describing schema (VER-1); `process.hgidl` sends a `BuildId` in `ready` and the broker refuses a child from another build.
- **Shell.** Ask for default once, never re-ask ([SH][SH] 6.11; UI R19, UP3). The external-protocol prompt class (6.4) and SH-R8 (gesture and a prompt per (origin, scheme, container)). The OS frame and title bar (6.2). The credential store is the OS keychain, Credential Manager or the Secret Service, from the broker (6.12). OS notifications are Stage 2, and the OS is a delivery channel only (6.13).
- **Owned elsewhere, not repeated here.** OS accessibility APIs (AX); the OS clipboard and drag formats and the IME (ED 6.5, 6.10, 6.11); presentation, vsync and the Wayland surface (GR); the script dialogs and the file chooser's shell surface (FI-5); the Screen Wake Lock and other sensor APIs (FI-28); the speech API (FI-29); the installed-app product (FI-9); installers and updates (FI-2); policy delivery (FI-3); credentials beyond storage (FI-13).
- **Owner decisions of 24 October 2026.** FI-D17: profiles share one broker. FI-D10: Linux packages are updated by the system package manager into versioned install directories. FI-D5: read aloud through OS voices sits outside the AI switch. FI-D8: the full PWA platform. FI-D14: open protocols first, Izcal's own implementation wherever possible.

**Success, in terms someone can measure.** All 22 FI-4 rows have a design (Appendix B). A second launch hands its URL to the running browser, across a version skew, in ≤ 300 ms† (OI-1, OI-2). No crafted URL ever becomes a switch (OI-4). A recording that began with an OS launch replays with no OS present (OI-7). The default-browser status shown is the truth on every OS state tested (OI-8). Installing and uninstalling leaves no OS artifact behind (OI-11). FP-1 does not move and the measured 0 idle wakeups per second stay 0 (OI-12, OI-13).

## 2. History: what came before and why it was built that way

### 2.1 Chronology

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| 1990s | **DDE** on Windows | Window-message protocol; shell verbs could send `WWW_OpenURL` to a running browser ([MS Learn, About DDE](https://learn.microsoft.com/en-us/windows/win32/dataxchg/about-dynamic-data-exchange)) | One address space per app, synchronous UI loops, no other IPC | Abandoned for "launch the executable and forward to the running instance". No primary source states why; the documented costs are deadlock rules and timeouts ([HP 4.1–4.2][HP], Low) |
| 2002–2007 | **Set Program Access and Defaults; Vista Default Programs** | Per-user defaults; apps declare `Capabilities` under `RegisteredApplications` ([MS Learn, Default Programs][MS-DP]) | UAC made machine-wide defaults from an elevated installer wrong; "The choice of default should be user driven" | The declaration survived; the set APIs did not |
| 2004 | **Mozilla's X remote protocol** | Commands as X window properties ([Mozilla archive](https://www-archive.mozilla.org/unix/remote.html)) | Client and browser "need not run on the same machine" | Removed in Firefox 36, partly restored in 36.0.1 after it broke scripts, removed for good in 39: it "requires a running instance" and is "not a cross-platform option" ([Bugzilla 1080319](https://bugzilla.mozilla.org/show_bug.cgi?id=1080319)) |
| 2007 | **`firefoxurl:`** (MFSA 2007-23) | IE passed unescaped quotes to a registered handler; a page launched Firefox with `-chrome` ([MFSA 2007-23](https://www-archive.mozilla.org/security/announce/2007/mfsa2007-23)) | Handlers receive the URL as command-line text | Fixed in the handler; the class recurred (2018, 2021, 2022, 2024) |
| 2009 | **Windows 7 AppUserModelIDs and jump lists** | One taskbar identity for a multi-process app; per-mode IDs for apps that look like several ([MS Learn, AUMIDs][MS-AUMID]) | Multi-process apps broke grouping and pinning | Survived unchanged; the identity must be set in five places |
| 2012 | **Windows 8 UserChoice hash** | Set-default APIs deprecated; the per-user choice guarded by a hash ([MS-DP][MS-DP]) | Apps were silently claiming defaults | The hash was reverse-engineered by 2017 ([HP 1.5][HP], Low), which led to a kernel driver |
| 2014 | **OS X 10.10 prompt; freedesktop MIME apps 1.0.1** | macOS asks "Use" or "Keep" before an app changes the browser ([Ctrl blog](https://www.ctrl.blog/entry/osx-protect-default-browser/), Medium); freedesktop writes the eight-file lookup for `mimeapps.list` ([MIME apps][XDG-MIME]) | Defaults are the person's; desktops need one standard | Both survived and were extended |
| 2015 | **Windows 10: only Windows may prompt** | Win32 apps lost their own "make me default" prompt ([Windows blog](https://blogs.windows.com/windowsexperience/2015/05/20/announcing-windows-10-insider-preview-build-10122-for-pcs/)) | Stop browser nagging | The upgrade itself reset browsers to Edge; Mozilla's CEO wrote an open letter ([Network World](https://www.networkworld.com/article/941768/mozilla-ceo-blasts-microsoft-over-windows-10-defaults.html), Medium) |
| 2016 | **Flatpak and xdg-desktop-portal** | D-Bus interfaces "usable by all apps, including ones which are confined by sandboxes" ([portal docs][XDP]) | Sandboxed apps cannot open files or URLs any other way | Grew by versions; a missing backend makes callers hang 25–26 s ([HP 3.7][HP], Medium) |
| 2017 | **GNOME 3.26 hides status icons** | Apps must work without a tray ([GNOME guidelines](https://wiki.gnome.org/Initiatives/StatusIconMigration/Guidelines)) | XEmbed trays did not fit Wayland or the shell design | Trays survive only through extensions and other desktops |
| 2018–2024 | **Argument injection and handler abuse** | Electron CVE-2018-1000006 (the `--` fix) ([Electron](https://electronjs.org/blog/protocol-handler-fix)); URL-open RCE in at least nine apps ([Positive Security][PS-URL]); `ms-officecmd:`, Follina, `search-ms:` ([HP 8.5][HP]) | Handlers trust the command line; OS-shipped handlers are attack surface too | Fixed one product at a time; the lesson is structural |
| 2019–2020 | **External-protocol prompts** | Chrome 77 drops "always open" ([HP 8.8][HP], Medium: no Chrome source); Firefox 84 makes "always allow" per origin after a Zoom abuse ([Bugzilla 1565574](https://bugzilla.mozilla.org/show_bug.cgi?id=1565574)) | A global grant let any site start a meeting | Per-origin grants survived |
| 2019–2020 | **Media keys in browsers** | Chrome 73 maps keys to Media Session ([Chrome](https://developer.chrome.com/blog/media-updates-in-chrome-73)); Firefox 81 uses SMTC, MPRIS and Now Playing ([SUMO](https://support.mozilla.org/kb/control-audio-or-video-playback-your-headset-or-keyboard)) | Users expect hardware keys to work | Survived; one OS session per browser |
| 2021 | **Scheme flooding** | Probing 32 apps' schemes gave a cross-browser identifier in seconds ([Fingerprint][FP-SF]) | Prompt presence and timing differ when no handler exists | At the article's August 2022 update Tor Browser had a fix and Chrome and Firefox did not; Safari's status was not updated |
| 2021 | **Windows 11 per-type defaults and `microsoft-edge:`** | Choosing a browser took dozens of steps; Microsoft blocked redirecting `microsoft-edge:` ([How-To Geek](https://www.howtogeek.com/768727/microsoft-calls-firefoxs-browser-workaround-improper-will-block-it/), Medium) | Microsoft: search is "not designed to be redirected" | A one-click "Set default" came in March 2022 ([HP 1.12][HP]) |
| 2023 | **"A principled approach to app pinning and app defaults"** | Users control pins and defaults; a supported `ms-settings:` deep link; a pin API with a trusted confirmation ([Windows blog][MS-PRIN]) | Regulatory pressure and vendor complaints | The deep link and the pin API shipped ([MS Learn, pin][MS-PIN]) |
| 2023 | **macOS 14 cooperative activation** | `activate()` is "a request, as opposed to a command"; apps yield focus ([WWDC23 10054](https://developer.apple.com/videos/play/wwdc2023/10054/)) | Focus stealing | Survived |
| 2024 | **UCPD.sys** | A filter driver that "blocks apps from writing app defaults data" ([MS-DEF][MS-DEF]); that it guards the http, https and .pdf keys comes from a vendor support note and Mozilla's prose ([WP 1.8][WP]; [HP 1.14][HP], Medium). It shipped with the February 2024 updates per that note and was reported in April 2024 per the press | Hash bypasses kept working | Microsoft Learn (26 September 2026): "Registry-based changes are not supported" |
| 2024–2025 | **EEA (DMA)** | Windows honours defaults for links in the EEA; the 2025 "Set default" also claims ftp, read, .mht, .svg, .xhtml, .xml, optionally .pdf, and pins the browser ([Windows Insider, 2025][MS-DMA25]) | The Digital Markets Act | Only in the EEA; still no set-default API |
| 2026 | **macOS 26.4 per-UTI prompts** | Every file-type default change is confirmed by the person ([Scripting OS X][SOX-264], Medium) | The same principle, extended | Live |
| 2025–2026 | **X11 sessions end; host apps get portal identity** | GNOME 50 ([release notes](https://release.gnome.org/50/), 18 March 2026) removed the X11 session ([LX 7.1][LX], Medium); KDE Plasma 6.8 is Wayland-only ([KDE](https://kde.org/announcements/plasma/6/6.8.0/), 14 October 2026); host apps register an app ID with `org.freedesktop.host.portal.Registry` (stable from xdg-desktop-portal 1.20.0, February 2025) ([Registry][XDP-REG]) | Wayland is the default everywhere; portals are meant for unsandboxed apps too ([reasons][XDP-WHY]) | Live |

### 2.2 Technology mismatches: assumptions that no longer hold

| Assumption | Where it came from | Why it fails now |
|---|---|---|
| An app can make itself the default | Pre-2012 Windows, pre-2014 macOS | Every OS now treats the default as the person's decision in OS UI ([HP 10.1][HP]) |
| The default, once set, stays set | Single-vendor desktops | The OS itself resets it on upgrade and restore ([HP 9.1][HP]); status must be re-read |
| Single instance is a lock file | One machine, one home directory, one version | Network home directories, crashes leaving stale locks, side-by-side channels, and an updated launcher meeting an older running browser ([HP 4.5–4.6][HP]) |
| The launcher and the running browser are the same build | Monolithic browsers | Updates replace binaries under a running browser; FI-D10 installs versions side by side; Izcal's IPC refuses another build |
| A URL argument is just a string | The browser parses its own command line | The URL is hostile input that crosses the OS's command-line quoting ([MS Learn, URI schemes][MS-URI]) |
| A tray icon is always available | Windows 95 to GNOME 3.24 | GNOME has no tray; Windows 11 hides icons in overflow ([HP 6.2][HP]; [WP 8.3][WP]) |
| Apps place their own windows | X11 and Win32 | Wayland has no client positioning; `xx-session-management-v1` has no compositor support ([LX 7.16][LX]) |
| The OS draws the window frame | Win32, AppKit, X11 window managers | GNOME's compositor does not implement xdg-decoration, so every app draws its own frame there ([LX 7.5][LX]) |
| OS state is outside the test | Browsers tested against live machines | Izcal's determinism (DET-1) needs every external input recorded, OS inputs included |
| OS integration lives in the browser process | Single-process ancestry | Several of these APIs load third-party code in process (shell extensions in file dialogs, antivirus in attachment checks, third-party voices) ([WP 9.2, 14.1][WP]) or parse data other local apps send (D-Bus) |

## 3. Today: the landscape and its gaps

**Default browser, per OS (October 2026).**

| | Windows 11 | macOS 26–27 | Linux (GNOME, KDE) |
|---|---|---|---|
| Declare | ProgIDs, `Capabilities`, `RegisteredApplications` ([MS-DP][MS-DP]) | `CFBundleURLTypes`, `CFBundleDocumentTypes` ([MP 1.1][MP]) | Desktop entry with `MimeType=` and `x-scheme-handler/*` ([XDG-DE][XDG-DE]) |
| Become default | Only the person, in Settings; deep link `ms-settings:defaultapps?registeredAppUser=` ([MS Learn][MS-SET]); one first-install notification after `SHChangeNotify` ([MS-DP][MS-DP]) | `NSWorkspace.setDefaultApplication(at:toOpenURLsWithScheme:)`, which prompts and completes asynchronously ([Apple][AP-SETDEF]) | Write the user's `mimeapps.list`; no prompt and no portal ([LX 1.6, 1.12][LX]) |
| Read status | `QueryCurrentDefault` ([WP 1.5][WP]) | `urlForApplication(toOpen:)` ([Apple][AP-URLFOR]) | Run the spec's lookup; `xdg-settings check` "can be false" ([xdg-settings](https://portland.freedesktop.org/doc/xdg-settings.html)) |
| Protected | UserChoice hash and UCPD.sys ([MS-DEF][MS-DEF]; keys per [WP 1.8][WP]) | The prompt, per UTI since 26.4 | Nothing; any app can write the file |

**What current browsers do not show or do, with evidence.**
- **Default status is guessed, not shown.** Browsers check once and prompt (Firefox's agent ran daily per install, [HP 1.9][HP]); none explains *which* types it holds or that the OS changed them.
- **Second-launch behaviour is invisible.** Which instance and which profile receive a link is not reported anywhere; the Firefox bug shows users fighting it with flags ([HP 4.4][HP]).
- **The OS edge is not recorded.** No browser documents a replayable record of what the OS delivered; bug reports that start with "I clicked a link in Outlook" cannot be reproduced.
- **External launches leak.** "No handler" and "declined" look different to the page ([Fingerprint][FP-SF]).
- **Persistent OS effects are scattered.** Default status, launch at login, background mode, notification grants and protocol grants live on five different settings pages, and background behaviour surprises people ([HP 9.5][HP]).

**What Izcal can do that no one does.** Its substrate records inputs (B1) and its IPC is typed and simulated (B2) (`references/izcal-capabilities.md`), so the OS edge can be a recorded stream like the network; the broker's prompt ledger (SH 6.4) can hold every persistent OS effect in one place; and the launcher and broker can be separate binaries with a proved contract, so an update never strands a running browser.

**Gaps in the corpus this spec closes.** The 22 FI-4 rows (Appendix B), the C17 phantom reference, audit Section 8 points 2 and 3, and the risks the audit names: background mode for push keeps the broker alive with no window, and OS-voice synthesis needs a home that is not the ML service ([FIA][FIA] §8, "Honest limits").

## 4. First principles

### 4.1 Users' real tasks

1. **"Use Izcal for my links."** Ask once, through the OS, and show the truth afterwards.
2. **"Open this here."** A link or file from another app opens in the running Izcal, in the profile it belongs to, in front, without a second browser starting.
3. **"Hand this to another app."** Share a page; open `mailto:`; open a downloaded file; and never let a page do that without the person.
4. **"Let the machine behave."** Media keys work; video keeps the screen awake; the machine sleeps when it should; nothing runs after the person quits unless they asked.
5. **"Fit my desk."** Correct scale on every monitor; windows return where they were; the frame looks and behaves like the platform's.
6. **"Remove it cleanly."** Uninstalling removes every registration.

### 4.2 The platform primitives

- **Windows:** the registry declarations, `QueryCurrentDefault`, `ms-settings:` deep links, `ShellExecute` command lines, named pipes and `AllowSetForegroundWindow`, AUMIDs and jump lists, SMTC and `DataTransferManager` interop, power requests, the Common Item Dialog, `IAttachmentExecute`, Per-Monitor v2 ([WP][WP]).
- **macOS:** Launch Services and Apple Events into the one `NSApplication`, `NSWorkspace`, `NSDockTile`, `UNUserNotificationCenter`, `MPNowPlayingInfoCenter`, `NSSharingServicePicker`, IOPM assertions, `SMAppService`, out-of-process open panels ([MP][MP]).
- **Linux:** the Desktop Entry, MIME Applications, Desktop Notifications, MPRIS and Secret Service specifications; the D-Bus specification 0.43; `org.freedesktop.Application`; xdg-desktop-portal; Wayland's xdg-shell, xdg-activation, fractional-scale, xdg-foreign and xdg-toplevel-tag; logind ([LX][LX]).
- **The web:** Web Share ([W3C Recommendation][WS]), Media Session, Screen Wake Lock, the external-protocol model of HTML's navigation, the manifest's handlers (FI-9).
- **Izcal:** the substrate and its simulator (B1, DET-1), typed IPC (B2), the prompt ledger, broker-formatted strings (TX-16, CON-R2), the threat-model template per process (SEC 6.5), the G-rule registry.

### 4.3 Izcal's principles that bind this design

P1 and I-1 (no process holds two sites' data: the speech role is therefore per (site, container), and the desktop role, which receives bounded strings from every site, is proposed as a fifth shared exception, OI-D25); I-2 (zero ambient authority for sandboxed processes); IPC-R15 (the broker parses no web or network content); IPC-R17 (no process trusts another); DET-1 (every external input recorded); AG-0 and B10 (nothing AI runs by default; OS speech must not start the ML service, FI-D5); FP-1 (the always-on budget); M-1 (specifications and OS documentation only); FI-D14 (open protocols first, our own implementation).

### 4.4 Assumptions kept and broken

| Assumption | Keep or break | Why |
|---|---|---|
| The browser registers its capabilities with the OS at install | **Keep** | Every OS makes eligibility a declaration ([HP 10.2][HP]) |
| The browser may set itself as default | **Break** | Every OS forbids it; Izcal asks through the OS only (OI-D7) |
| Single instance is a lock file | **Break** | Single instance is a **message** to the running broker, with the lock only as a tie-breaker (OI-D4) |
| The launcher is the browser | **Break** | A stable launcher hands off or starts the broker from a versioned directory (OI-D3) |
| One message format across all processes | **Break** for exactly one channel | The hand-off is the only cross-build message (OI-D2); everything else stays same-build |
| OS integration runs in the browser process | **Break** | Only what the OS forces there; third-party in-process code and other apps' data go to sandboxed or medium-IL helpers (OI-D5) |
| OS state is read where it is used | **Break** | Read once, through `OsPort`, recorded; renderers never read it (DM's environment record already says so) |
| Background running is a convenience | **Break** | It is a persistent OS effect, opt-in and visible (OI-D17) |
| Tray icons, badges and jump lists are guaranteed | **Break** | Best-effort extras; nothing depends on them ([HP 6][HP]) |
| The OS draws the frame | **Keep where the OS offers it; break on Wayland without SSD** | OI-D20 |
| Use a mature library for D-Bus | **Break** | The protocol is fully specified; the available crate is async-runtime based, which CS-AS-4 forbids (OI-D6) |

## 5. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Windows 8 → UCPD.sys | Each protection was bypassed, so the next one was stronger ([HP 1.4–1.14][HP]) | Never fight the OS for the default | OI-D7: register, ask through the OS, read status |
| Windows 10 upgrade; Windows Backup 2026 | The OS reset the choice ([HP 1.7, 1.20][HP]) | Status must be re-read, not remembered | Default status is an `OsPort` input at launch and on activation (6.4) |
| Firefox's Default Browser Agent | A daily task, telemetry concerns ([HP 1.9][HP]) | Checking must not become surveillance or nagging | No agent, no scheduled task; status shown in Settings only |
| macOS setter returns before the answer | The return value says nothing ([HP 2.2][HP]) | The answer is asynchronous | The broker re-reads with `urlForApplication` after the completion |
| macOS 26.4 per-UTI prompts | Looping over types spams the person ([MP 1.6][MP]) | One request, one prompt | OI-D7: one http request on macOS; file types left to the person |
| Firefox `-remote` removal | Broke scripts; partial backout ([Bugzilla 1080319](https://bugzilla.mozilla.org/show_bug.cgi?id=1080319)) | The command-line interface is a public API | OI-D8: a closed, documented, versioned switch set |
| Chromium singleton locks on NFS | Stale locks strand users ([HP 4.5][HP]) | A lock must name its holder and never be deleted silently | OI-D4: host and PID in the lock; an explicit message |
| Firefox 67 dedicated profiles, downgrade protection | Channels shared a profile across versions ([HP 4.6][HP]) | Channels are separate instances | OI-D4: one rendezvous per channel |
| xdg-activation-v1 | A raise without a token may be refused ([LX 2.5][LX]) | Focus is a capability passed by the launcher | The hand-off carries the activation token (6.2) |
| `firefoxurl:`, Electron, `ms-officecmd:`, Follina | The URL became switches ([HP 8][HP]) | Structural fix, not per-bug fixes | OI-D8: end-of-options marker, one operand, a refusing parser, OI-4's corpus |
| Positive Security's URL-open study | Apps passed URLs to the OS opener and got RCE ([PS-URL][PS-URL]) | The outbound side is just as dangerous | OI-D9: scheme blocklist, no `file:`/UNC from web content |
| Zoom's localhost server | An app bypassed the browser's prompt because it was annoying ([HP 8.7][HP]) | Prompts that cost too much get bypassed | Per-(origin, scheme, container) grants keep repeat launches cheap (SH-R8) |
| Scheme flooding | Outcome differences fingerprint installed apps ([Fingerprint][FP-SF]) | Make outcomes indistinguishable | OI-D9, OI-10 |
| GNOME status-icon removal | Apps that depended on trays broke ([HP 6.2][HP]) | No feature may depend on a tray | Background mode is signalled by notification and portal status (OI-D17) |
| Firefox 81 MPRIS under Flatpak | Bus-name ownership needed a permission ([HP 7.2][HP]) | Media registration is an OS identity question | MPRIS lives in the OS-integration service with a stable name (OI-D14) |
| Portal hangs of 25–26 s | A missing backend blocked startup ([HP 3.7][HP]) | Every portal call has a timeout and a fallback | OI-D23 folded into 6.9; OI-19 |
| Edge Startup Boost | "Edge starts itself" ([HP 9.5][HP]) | Preloading at login surprises people | Refused (OI-D17) |
| Safari web apps as separate bundles | Notifications and badges are per bundle ([MP 4.4][MP]) | Per-app OS identity needs a per-app OS object | The shared registration record (OI-D10) creates one per app for FI-9 |
| Safari's "Allow JavaScript from Apple Events" | TCC protects the target only from unknown senders ([MP 12.4][MP]) | Scriptability is a capability grant | OI-D18: scripting only behind debug mode and a per-app, expiring grant inside Izcal; page access only in an ephemeral container |

These are ideas and evidence. No code and no template is taken from any of them (M-1).

## 6. Design

### 6.0 Architecture at a glance

```
          other apps, file managers, terminals, the OS shell
                         |  (URL / file opens, command lines, notification clicks)
                         v
   +----------------------------------+      hg-launch-wire (IZLH v1)      +--------------------------------+
   | LAUNCHER (stable path, tiny)     |----------------------------------->| BROKER                         |
   |  argv parse (OI-D8), rendezvous, |   per-user, per-channel endpoint   |  OsPort (OI-D1): typed inputs  |
   |  verifies the server, hands off  |<-----------------------------------|  in, commands out; window-     |
   |  or starts the broker            |  accepted / busy-retry / refused   |  owning APIs; hg-dbus GATE:    |
   +----------------------------------+                                    |  the only D-Bus connections    |
                                                                           +---+-----------+------------+---+
                 typed IPC (same build), forwarded D-Bus bodies (gated)        |           |            |
              +----------------------------------------+-----------------------+           |            |
              v                                        v                                   v            v
 +-------------------------------+   +------------------------------+   +------------------------+  UI HOST (Windows):
 | OS-INTEGRATION SERVICE        |   | OS-INTEGRATION SERVICE       |   | OS SHELL HELPER        |  per-window reports,
 | desktop role (1, site-less)   |   | speech role (per site, cont.)|   | (Windows, medium IL)   |  tagged, validated
 |  portal/MPRIS/notification/   |   |  OS voices, SSIP; PCM to the |   |  IFileDialog,          |
 |  LauncherEntry/SNI bodies;    |   |  audio service               |   |  IAttachmentExecute    |
 |  no bus socket, no network    |   |  no bus socket, no network   |   +------------------------+
 +-------------------------------+   +------------------------------+
```

**Placement by what each call has to trust.** OS code goes where the OS forces it, and no further toward the broker:

| Placement | What runs there | Why |
|---|---|---|
| **Broker** | Everything the OS delivers to the app's main process: Apple Events and `application(_:open:)`, `NSDockTile`, the Dock menu, `NSSharingServicePicker`, `NSOpenPanel` (already out of process since macOS 10.15, [MP 9.1][MP]), `UNUserNotificationCenter` and now-playing (bundle identity, [MP 16.5][MP]); on Windows the shell COM calls that need medium integrity ([WP 15.2][WP]) and a hidden top-level window that receives the OS broadcasts (6.1); on Linux the Wayland toplevel requests (GR already holds the connection); the hand-off listener on every OS; and the **`hg-dbus` gate**, the only process holding session-bus and system-bus connections | The OS gives no other choice, the call carries a credential (SH 6.12), or the call is the boundary itself |
| **OS-integration service, desktop role** (new, sandboxed, one per broker, site-less) | The bodies of portal, Desktop Notifications, MPRIS, LauncherEntry and StatusNotifierItem traffic, forwarded through the gate | It parses what other local programs send; IPC-R15 keeps such parsing out of the broker, as AX-D12 already does for AT-SPI |
| **OS-integration service, speech role** (new, sandboxed, per (site, container) of the speaking document) | OS speech synthesis on every OS and the SSIP client (6.15) | Voices are third-party code on Windows and other programs' services on Linux, and utterance text is page content (I-1) |
| **OS shell helper** (new, Windows only, medium IL) | The Common Item Dialog and `IAttachmentExecute::Save` | Both are in-process COM that load third-party shell extensions and antivirus components ([WP 9.2, 14.1][WP]); a crash or a hostile extension then takes down the helper, not the broker |

Renderers, the shell renderer and the prompt renderer never touch any of it (I-2).

**The D-Bus gate (OI-D6, critique C1).** A sandboxed process with a raw session-bus socket is not sandboxed: on systemd desktops the session bus reaches `org.freedesktop.systemd1` (which starts arbitrary commands as the user), `org.freedesktop.secrets` (whose login collection is normally unlocked), the portals and Izcal's own interfaces. So no sandboxed Izcal process ever holds a bus socket. The broker holds the connections and runs the **gate**: `hg-dbus`'s header decoder (tier P, Kani-proved) reads only the fixed header fields of each message (type, serial, reply serial, destination, path, interface, member, signature) and forwards the body, unparsed, to the role that declared that tuple, or drops it. Each role has a closed, versioned allowlist of (destination, interface, member, direction) tuples and of the bus names it may have the gate own on its behalf (for example `org.mpris.MediaPlayer2.<appid>`); `BecomeMonitor`, eavesdropping match rules, name ownership outside the list and calls to any destination not on the list are refused, and every refusal is counted in `about:diagnostics`. Replies are forwarded only to the role whose call carries that serial. The same gate serves AX-D12's AT-SPI bridge and MD's portal calls, and Section 9.5 proposes that their specs adopt it.

**What the broker itself parses (recorded exception, critique C8).** IPC-R15 forbids the broker to parse page and network content; same-user local input is a different class (TB-10), and the broker already parses OS callbacks. This spec records a narrow, explicit set: the IZLH frame (6.2); D-Bus headers in the gate; and four closed D-Bus bodies the broker must understand itself: the Secret Service replies of SH 6.12, `org.freedesktop.portal.Settings.SettingChanged` for the four appearance keys, logind's `PrepareForSleep(b)` and its delay-lock reply, and (Stage 2) `org.freedesktop.Application`; and, only while debug mode is on, the closed OS scripting command set (6.14). Each has a tier-P decoder with Kani harnesses, a fuzz target and an adversarial-peer suite (IPC-R17). Everything else from the bus is a body the gate forwards without reading.

**Windows: the window owner is the UI host (critique C4, C13).** Under SH-D3 the UI host owns content HWNDs and their message loop. Window-independent OS events therefore come from windowless or broker-owned sources: `RegisterSuspendResumeNotification` and `RegisterPowerSettingNotification`, UISettings change events, display enumeration, and a **hidden top-level window owned by the broker**, which receives the broadcasts (`WM_SETTINGCHANGE`, `WM_DISPLAYCHANGE`, `WM_POWERBROADCAST`, `WM_QUERYENDSESSION`) that only top-level windows get. Per-window events that exist only in the UI host (`WM_DPICHANGED` for a content window, a move to another display, `WM_POINTER` for gestures through `hg-input-route`) arrive as typed UI-host reports, are tagged `reported-by: ui-host`, are validated against the broker's own display enumeration, and are recorded with that tag; they are untrusted input (IPC-R17). Window-scoped shell calls that need the content HWND (taskbar progress, per-window AUMIDs and relaunch properties, SMTC's `GetForWindow`, `ShowShareUIForWindow`) are the open risk: whether they work cross-process from the broker on the UI host's HWNDs, or from the UI host under its real token, is **measured at SH-M0, before M2** (Q-OI-1), alongside a third option, a broker-owned top-level frame window with the UI host's content window as a child. Which outcome would reopen SH-D3, and what security amendment each needs, is recorded in Q-OI-1 now, so the answer arrives before the shell's window platform is built.

### 6.1 `OsPort`: the OS as a recorded input (OI-D1, OI-D26, an M1-exit input)

**The interface.** One substrate trait, present only in the broker's substrate, shaped like the `Ipc` trait as built (`channel`, `send`, `listen` with a sequence, a sink and a guard), so it adds no new concurrency idea and obeys CS-AS-4: no `async`, completions as tasks, and frames as bytes with the typed layer above.

```rust
/// The broker's door to the operating system's non-input services (OI-D1).
/// Frames are `os.hgidl` encodings; `hg-os-types` (L2) decodes them.
/// Production: hg-substrate-os's per-OS modules, the gate, and the OS-integration
/// service and OS shell helper behind typed IPC. Simulation: a scripted OS model,
/// or (from OI-M1) a recorded stream.
pub trait OsPort: Send + Sync {
    /// Ask the OS to do something; never blocks. The answer arrives as an input
    /// frame carrying the returned id.
    fn command(&self, frame: Vec<u8>) -> Result<OsCallId, OsError>;
    /// Deliver every OS input frame, in order, as tasks on `seq`. Dropping the
    /// guard stops delivery. One broker dispatcher fans inputs out to consumers.
    fn listen(&self, seq: SequenceHandle, sink: OsSink) -> Result<OsListening, OsError>;
}
// OsCallId: u64, monotonic per broker incarnation. OsSink: like `Sink`, a boxed
// FnMut(OsDelivery) where OsDelivery = Frame(Vec<u8>) | Closed. OsError: closed enum
// (Unsupported, TooLarge, Closed, Refused). OsListening: a guard, as `Listening`.
pub struct Substrate { /* … as built … */ pub os: Option<Arc<dyn OsPort>> }
```

**What crosses it.** `OsInput` and `OsCommand` are closed unions declared in `os.hgidl`, generated by `hg-ipc-codegen` into `hg-os-types` (L2), so every arm gets the generated parser and Kani harness of any message, and every arm fits the generator's 64 KiB message cap (6.2 sets the bounds). Same-build is right here: `OsPort` is in-process, and recordings carry the engine build ID ([S1][S1] 3.3).

**The OS-input inventory (critique C3).** Every OS-sourced value the corpus reads, who reads it, and how it is recorded:

| `OsInput` kind | Contents (typed, bounded; "unknown" is a value) | Read by | Consumers |
|---|---|---|---|
| `Launch` | A decoded IZLH request (6.2) or an OS-delivered open (Apple Event, COM notification activation; `org.freedesktop.Application.Open` in Stage 2), normalised to one record, with its origin (`command-line`, `os-open`, `notification`, `jump-list`, `dock-menu`) | Broker listener | `hg-os-broker`'s launch policy |
| `DefaultStatus` | Per type (http, https, `.html`, `.htm`, `.xhtml`, `.svg`, `.pdf`): `is-izcal`, `is-other`, `unknown`; and the OS's source | Broker on request (6.4) | Shell Settings |
| `Settings` | A superset of AX 6.11's sources: colour scheme, accent, contrast, forced colours and the palette's system colours, reduced motion, reduced transparency, inverted colours, differentiate without colour, text scale (as a curve), caption style, caret and focus-ring widths, Apple's separate image and flash settings, Android's duration scale (Stage 2) | Broker (Windows UISettings and the hidden window, macOS `NSWorkspace`), the gate (Linux Settings portal) | `hg-ax-prefs` (AX 6.11, record unchanged), shell theme |
| `Locale` | UI languages, preferred languages, regional formats | Broker | Shell (FI-6), the environment-values record (DM 6.16) |
| `TimeZone` | Zone id and change events | Broker | The substrate clock's zone, DM 6.16 |
| `Hardware` | Logical core count, memory class, ISA | Broker | DM 6.16 (normalised there, D-S1), FP |
| `Displays` | Monitors: id, bounds, work area, scale as a rational p/q (GR-D17), refresh, primary; per-window moves (tagged `reported-by` on Windows) | Broker; UI host reports (Windows) | GR-D17, DM 6.16, the shell's placement |
| `Power` | AC or battery, battery saver or Low Power Mode, thermal state, screen locked; `will-sleep`, `session-end` and `query-end` as **deadline inputs** (below); `did-wake` | Broker, the gate | FP 6.8, the session store, the network services |
| `Media` | A media command (play, pause, next, previous, seek to t) | Broker or the desktop role | The tab owning the active session (6.10) |
| `Notification` | Activated or dismissed, with Izcal's opaque ids (Stage 2) | Broker or the desktop role | SH 6.13 |
| `Answer` | The result of a command: share completed or cancelled, dialog handles or cancel, default request completed, registration applied with its artifact list, wake lock held or refused, portal unavailable | Whoever ran the command | The command's issuer |

**Outside `OsPort`, and why.** **Enterprise policy** (owner decision, 25 October 2026): registry, managed preferences and policy files are parsed in the enterprise policy spec's sandboxed **policy adapter** (EP-D7), not in the broker and not through `OsPort`, and reach the broker as typed `.hgidl` snapshots that carry a 32-byte `PolicyDigest`; the digest is the recording and crash-report header field of FIA §8.1. Both paths are recorded inputs (the policy snapshots at the adapter's IPC boundary, the rest at `OsPort`), and renderers see neither. Pointer, keyboard and IME input stay on `hg-input-route` and ED's recorded streams; presentation timing stays GR's typed timing stream (GR-D15); network configuration stays the broker's NW reads (NW 6.1) until NW adopts `OsPort` (Section 9.5); the OS clipboard is ED's (ED-D10). G234 names exactly these exceptions, with enterprise policy.

**Deadline inputs (critique C22).** Some OS events need an answer before the OS callback returns: `WM_QUERYENDSESSION` (in the broker's hidden window), `applicationShouldTerminate:`, the sleep acknowledgement, the Inhibit portal's `QueryEnd` (which callers should answer within 1 s, [LX 3.7][LX]) and logind's delay lock. These arrive as inputs carrying a **reply token** and a deadline. The OS callback is held only by `hg-substrate-os`'s per-OS module on its own OS thread, which never waits on another process (the same rule as the coding standard's OS-callback rule); the broker answers with an `OsCommand::Reply(token, verdict)`; if no reply arrives by 80%† of the deadline, the module answers the OS with the platform's default (allow the session to end, release the delay lock) and the broker records the timeout. The reply is recorded as a command, so a replay checks it.

**Policy and fallbacks live in the broker (critique C23).** Portal timeouts, fallback choices, media-session arbitration and notification logic are decided in `hg-os-broker` from `Answer` inputs and the substrate `Clock`, not inside the service. The service is a thin codec and transport, so DST scenarios such as "portal missing" exercise the real policy.

**Recording (OI-D26, critique C5).** Two scopes:
- **Simulation and DST** see the whole `OsPort` stream. A **scripted OS model** answers commands and produces inputs ("default is another browser", "portal missing", "sleep in 3 s", "display unplugged", "1,000 launches at once"), so the launcher, the hand-off and the OS policy are tested under the seeded scheduler. The OS-integration service, the OS shell helper and the gate are on the OS side of this boundary in simulation and are tested by per-OS conformance suites.
- **A production tab recording** (S1 3.4: per tab, after "Reload to record", in a fresh renderer) carries only the OS-derived values that tab consumed: its environment-values record and the `SetEnvironment` changes (already recorded, DM 6.16), its own launch record if the tab was opened by one, and its own dialog and share answers, each joined by ID under REC-R6. Other tabs' launches, other containers' activity and the browser-wide display and default-status stream are **not** in a tab recording. A browser-wide OS recording is not offered in Stage 1; if it is ever wanted, it is a separate opt-in with its own privacy statement.

A replay makes **no OS call** (OI-7); the substrate graph lint already keeps `hg-substrate-sim` out of shipping binaries.

**Who consumes it.** `hg-ax-prefs` maps `Settings` into its `AccessibilityPreferences` record without changing that record (AX 6.11); the shell reads colour scheme and accent for its own theme; the environment-values record (DM 6.16) reads `Displays`, `Locale`, `TimeZone` and `Hardware` and applies D-S1's normalisation; FP's background policy reads `Power`. **Exposure to pages** is decided where it is today: AX-D18 for preference media features, D-S1 and DM 6.16 for discrete values. The **accent colour reaches the shell only**: CSS's `AccentColor` system colour is a style-spec decision under D-S1, and this spec recommends a fixed value for pages (Q-OI-9).

**The first frame's theme (critique C12).** The broker reads colour scheme and contrast before the shell's first paint with a bounded wait of ≤ 50 ms†; on Linux, where they come from the Settings portal through the gate, a timeout uses the last value cached in the profile and reconciles when the answer arrives, so SH-1's first paint never flashes the wrong theme and no portal wait exceeds the bound.

### 6.2 The launcher and the hand-off (OI-D2, OI-D3, OI-D4; an M1-exit input)

**Install and launch layout, per OS (critique C11).** The launcher is the stable entry point; the broker runs from a directory a running browser never sees replaced. This is a requirement handed to FI-2, which owns the layout and its clean-up.

| OS | OS-registered entry point | Broker and children | What an update may replace while Izcal runs |
|---|---|---|---|
| Windows | `izcal.exe` (`hg-launch-main`) at a stable path in the install directory | `<install>\<version>\` holds the broker and every child binary; the broker spawns children only from its own version directory | The launcher (by atomic rename) and new version directories; an old version directory is removed only after its broker exits |
| macOS | `Izcal.app`, whose `CFBundleExecutable` **is the broker**, because Launch Services, Dock identity, notification authorisation and keychain groups attach to that process ([MP 16.5][MP]) | The broker spawns children from a **versioned helper bundle** inside the app bundle (`Contents/Frameworks/Izcal <version>.framework/Helpers/`), which it pins at start; the command-line launcher is a small tool inside the bundle (`Contents/MacOS/izcal-cli`, linked to the user's path by the installer) | A new version helper bundle and the outer bundle's executable (the running broker keeps its mapped image); the old helper bundle is removed after the broker exits. A cold command-line launch goes **through Launch Services** (`NSWorkspace` open with the URLs as an open request), never `exec`, so the broker always starts with its bundle identity |
| Linux | `/usr/bin/izcal` or the tarball's `izcal`, a stable launcher | FI-D10's versioned install directories for packages; the tarball's updater uses the same layout | The package manager installs a new version directory and the launcher; the old directory stays until its broker exits (FI-D10) |

On macOS OS-originated opens arrive as Apple Events in the broker ([MP 2.1–2.3][MP]); only command-line launches need the hand-off, because they bypass Launch Services ([MP 2.5][MP]). The launcher is also the "small platform launcher" SH 6.1 requires for the recovery window.

**The launcher's algorithm (critique C17).** It links no engine crate and parses argv with 6.3's grammar. Then:
1. **Try to hand off.** Connect to the channel's endpoint; **verify the server** (below); send the request; on `accepted`, grant foreground rights (Windows: `AllowSetForegroundWindow` for the verified broker PID) and send `commit`; exit. On `busy-retry`, wait the advised interval and retry, up to 2 s† in total.
2. **If no endpoint exists, start a broker.** Take the profile root's lock (an exclusive lock file in the profile root holding host, PID and start time); create the listening endpoint; start the broker (Windows: from the version directory, passing the listening handle; macOS: through Launch Services; Linux: from the version directory, passing the listening socket); wait for the broker's `started` reply for up to 10 s†; if it does not come, start the recovery window (SH-D22) and exit.
3. **If the lock is held but no endpoint answers,** the holder is starting: retry the connection with backoff for up to 2 s†. If the holder is dead on this host (no process with that PID and start time), take over the lock. If the holder is **another host** (a network home directory), show a message naming that host and exit; the lock is never deleted automatically (OI-6).

**The rendezvous.** One endpoint per (OS user, OS session, channel, profile root). Profiles share one broker (FI-D17), so the profile is a field in the message, not part of the address; side-by-side channels (PO-1.10) have separate endpoints and never hand off to each other.

| OS | Endpoint | Server check by the launcher | Client check by the broker |
|---|---|---|---|
| Windows | Named pipe `\\.\pipe\Izcal.<channel>.<session id>.<h>`, *h* = 16 hex digits† of SHA-256 over the user SID and the canonical profile-root path; DACL for the user's SID only, a mandatory label of Medium with no-write-up so low-IL and AppContainer callers cannot connect, `PIPE_REJECT_REMOTE_CLIENTS` | `GetNamedPipeServerProcessId`; the server's image path is inside the install directory, its token user is the caller's and its integrity is Medium | The client's token user and integrity |
| macOS | A Unix-domain socket in a per-user directory under the user's temporary directory (mode 0700), named by *h* (`sun_path` is short, Q-OI-2) | `LOCAL_PEERPID`, then the peer's code signature is Izcal's (team identifier) | `getpeereid` |
| Linux | A Unix socket in `$XDG_RUNTIME_DIR/izcal-<channel>/` (mode 0700). Stage 1 has **no D-Bus rendezvous**: `DBusActivatable` is false, so there is one path into the broker. Stage 2 adds `DBusActivatable=true` with a specified activation file whose command is the launcher with `--dbus-activation`, the gate owning the name, and `org.freedesktop.Application` decoded by the broker's closed decoder, needed for portal notification actions (6.16) | `SO_PEERCRED`, then the peer's executable is inside an Izcal version directory | `SO_PEERCRED` |

Renderers, the shell renderer, the OS-integration service and every other sandboxed Izcal process cannot reach any endpoint (their sandboxes deny the path or the pipe), and OI-14 probes it.

**`hg-launch-wire` v1 ("IZLH").** A fixed, little-endian, length-prefixed TLV format, written from first principles for one job, decoded by a tier-P crate with no dependencies, Kani-proved field by field, and the **only message format Izcal accepts from another build**.

| Part | Layout |
|---|---|
| Header (12 bytes) | magic `IZLH` (4); major version u8 = 1; minor version u8; message type u8 (`request`, `reply`, `commit`, `started`); flags u8 (zero in v1); total length u32, ≤ 64,512† bytes including the header, so a request always fits the IPC generator's 64 KiB cap when the broker re-encodes it as a `Launch` input |
| Field (repeated) | tag u16; flags u8 (bit 0 = **critical**: a receiver that does not know the tag must refuse the whole message); reserved u8 = 0; length u32; value |
| Values | UTF-8 strings, validated, no NUL; URLs and paths ≤ 8,192† bytes each; other strings ≤ 512† bytes; at most 6† URLs plus paths per request, so 6 × 8 KiB plus the other fields stays under the total; integers little-endian; at most 32† fields |

| Tag | Field | Critical | Notes |
|---|---|---|---|
| 1 | Intent: `activate`, `open`, `new-window`, `new-private-window` | Yes | Closed; no automation intent exists (AGT-R8, SEC-20) |
| 2 | URL (repeatable) | Yes | Checked against the inbound allowlist (below); never a switch |
| 3 | File path (repeatable) | Yes | Absolute; re-canonicalised by the broker; checked against the inbound allowlist |
| 4 | Working directory | No | Only to resolve relative paths the launcher could not |
| 5 | Profile selector | Yes | A profile id or `default`; an unknown profile is refused, not created |
| 6 | Activation token | No | xdg-activation token or startup id (Linux); absent on macOS and Windows |
| 7 | Origin | No | `command-line`, `os-open`, `jump-list`, `notification`, `dock-menu` |
| 8 | Sender build | No | Build id u64, version string, channel |
| 9 | Nonce and batch | No | u64 nonce echoed in the reply; batch index and count when one launch carries more than 6 operands and is split across several requests on one connection |

The **reply** is `accepted`; `busy-retry` with a retry-after in milliseconds; or `refused` with a closed reason (`malformed`, `unsupported-critical-field`, `too-large`, `unknown-profile`, `not-allowed` for an operand outside the allowlist, `queue-full`, `shutting-down`). A URL longer than 8 KiB† is refused by the launcher before sending, with a message.

**Versioning rules.** A v1 receiver accepts any `major = 1` message and ignores unknown non-critical fields. A new feature that an old broker must not silently drop is sent as a critical field; on `unsupported-critical-field` the launcher retries once with only the fields the older broker understands, if the request can be expressed that way, and otherwise says "Izcal is running an older version; restart it to use this". A v1 receiver refuses `major = 2`, and every launcher can send v1 for the life of the support period; because the channels are separate instances (6.2), skew only arises within a channel, during an update, so the matrix covers FI-2's Stable cadence of 4 weeks (2 weeks later) and the signed public Nightlies FI-2 plans for at least 3 months before Stable. The skew matrix (OI-2) runs this release's launcher against the last three releases' brokers, and this release's broker against the last three releases' launchers, in CI from OI-M1.

**Bursts queue, never fail (critique C10).** The broker accepts requests into a bounded queue of 1,024† per channel and processes them in order; when the queue is above its high-water mark it answers `busy-retry` with a retry-after; only beyond the bound does it answer `queue-full`. Repeated `activate` intents are coalesced. A multi-select "Open" from a file manager or a script that opens fifty links therefore succeeds; if anything is refused, the launcher shows one message for the batch, not one per launch.

**Inbound allowlist (critique C9).** The hand-off, Apple Events, Stage 2's `Application.Open` and dropped URLs (ED 6.11's drag-in, by amendment) are the channels any local program can use to make Izcal navigate, so inbound is an **allowlist**, decided from first principles: `http` and `https` URLs; `file:` URLs and absolute paths **on a local volume**; and `about:blank`. Everything else is refused with a message, including Izcal's internal and privileged pages (settings, about pages, DevTools, `ext://`, `app://`), `view-source:`, `javascript:`, `data:` and `blob:`. **Remote paths are never opened silently**: a UNC path, `file://host/…`, or a path on a network-mounted volume (Windows `DRIVE_REMOTE`, an SMB or NFS mount on macOS and Linux) opens a prompt that names the remote host, because opening it makes the OS authenticate to that host. Every allowed URL opens as a new top-level navigation with no opener and no user activation, passes the phishing check (PH 6.2), and never carries post data or headers.

**Abuse limits.** Same-user, same-session only, with server verification in both directions; the queue bound above; at most one window raise per 500 ms†, so a flood of requests cannot keep stealing focus.

### 6.3 Command line (OI-D8)

**A closed grammar.** The launcher accepts exactly these forms; anything else is an error message, never a guess.

| Switch | Meaning | Stage |
|---|---|---|
| `--new-window`, `--private-window` | Intent | 1 |
| `--profile <id or name>` | Profile selector | 1 |
| `--profile-root <dir>` | A separate profile root (testing, portable use); its own rendezvous | 1 |
| `--os-open -- <operand>…` | The form every OS registration uses; after `--` only operands | 1 |
| `--automation-pipe` | DevTools' pipe launch into a fresh ephemeral container; never hands off (DT 6.3, AGT-R8) | 1 |
| `--headless` | FI-7's headless mode; never hands off | 1 (FI-7) |
| `--safe-mode` | Opens SH-D22's recovery actions | 1 |
| `--dbus-activation` | Started by the session bus through the activation file; hands off or starts the broker like any launch | 2 |
| `--version`, `--help` | Print and exit | 1 |
| `--` | End of options; the rest are URLs or paths | 1 |

**Refused, with a reason in `--help`.** Proxy overrides and network switches (settings and policy do this, NW 6.14); feature flags in release builds (local flags live in an about page, FI-D12); a remote-debugging port (AGT-R8: no switch attaches to a user container); a user-data directory that points at another browser's profile.

**The OS-open rule.** Registrations are `"<launcher>" --os-open -- "%1"` on Windows and `Exec=<launcher> --os-open -- %U` on Linux. In `--os-open` mode no switch is honoured anywhere, and on Windows **exactly one operand** must follow `--`: if the OS's quoting split a crafted URL into several argv entries ([MS-URI][MS-URI]), the launcher refuses all of them and says the link could not be opened safely. On Linux each `%U` item is already one argv element ([LX 1.3][LX]). An operand that is neither an absolute URL with a scheme nor an absolute path is refused, and every operand then meets 6.2's inbound allowlist. The registered command lines of other browsers are read black-box on the Windows rig as a cross-check, never their source ([HP][HP] measurement list).

### 6.4 Default browser and handlers (OI-D7)

**Registration at install** is FI-2's installer step, expressed as `OsRegistration` records (6.6) so uninstall removes exactly what was written.

| OS | What Izcal registers |
|---|---|
| Windows | Per-scheme and per-type ProgIDs; `Capabilities` with `ApplicationName`, `ApplicationDescription`, `UrlAssociations` (http, https, and `mailto` only if Izcal handles mail links, which it does not in Stage 1), `FileAssociations` (`.htm`, `.html`, `.shtml`, `.xht`, `.xhtml`, `.svg`, `.xml`, and `.pdf` because PD views PDFs; not `.mht` or `.mhtml`, because Izcal does not open MHTML in Stage 1, and no image types), so the EEA one-click "Set default" covers what Izcal handles and nothing more; `RegisteredApplications`; `StartMenuInternet` for convention ([WP 1.1–1.2][WP]); one `SHChangeNotify(SHCNE_ASSOCCHANGED)` at first install so Windows can offer Izcal once ([MS-DP][MS-DP]) |
| macOS | `CFBundleURLTypes` (http, https) and `CFBundleDocumentTypes` (`public.html`, `public.xhtml`, `com.adobe.pdf`, `public.svg-image`) in Info.plist |
| Linux | A desktop entry named after the channel's app ID with `MimeType=x-scheme-handler/http;x-scheme-handler/https;text/html;application/xhtml+xml;application/pdf;image/svg+xml;`, `DBusActivatable=false` in Stage 1 (6.2), `Exec=` as in 6.3, `Actions=new-window;new-private-window;` ([XDG-DE][XDG-DE]); the package runs `update-desktop-database` |

Izcal registers only for types it really handles, as Microsoft asks ([MS-DEF][MS-DEF]); registering `.pdf` means the PDF viewer (PD) is the handler, and the person sees that in the OS surface. When a person approves a web page as a handler for a scheme (`registerProtocolHandler`, FI-9's handlers, FI-D8), Izcal registers that scheme with the OS as an `OsRegistration` of kind `Handler` with the ledger entry as its consent reference, and removing the web handler removes the registration (OI-11). On Windows the deep link takes the registered application name URI-escaped, and needs Windows 11 21H2 or 22H2 with the April 2023 cumulative update, or 23H2 and later ([MS-SET][MS-SET]).

**Becoming the default** happens only when the person asks: the onboarding screen's "Make Izcal default" button or the Settings row (SH 6.11). The broker issues `OsCommand::Ask(DefaultBrowser)`:

| OS | What happens | Then |
|---|---|---|
| Windows | Open `ms-settings:defaultapps?registeredAppUser=<name>` (or `registeredAppMachine` for a per-machine install), on builds that support it; otherwise the Default Apps page ([MS-SET][MS-SET]). In the EEA the OS's own "Set default" covers the wider type list | Re-read on the next activation of an Izcal window |
| macOS | One `setDefaultApplication(at:toOpenURLsWithScheme:"http")`; the OS prompt is the only consent; no loop over file types, because macOS 26.4 reportedly prompts per type ([MP 1.6][MP]) | Re-read http, https and `public.html` with `urlForApplication` after the completion, which may never come; "no answer yet" is a normal state; a type the OS left with another app is shown as such |
| Linux | Write `[Default Applications]` entries for the four web types into `$XDG_CONFIG_HOME/mimeapps.list`, the location the spec reserves for configuration GUIs ([XDG-MIME][XDG-MIME]), preserving every other line; if a higher-precedence desktop-specific file (`$XDG_CONFIG_HOME/<desktop>-mimeapps.list`) names another app for a type, write that file's default entry instead, because that is the one the lookup uses, and tell the person which file changed; this click is the consent, because Linux has no OS prompt | Re-read by running the spec's lookup |

Izcal **never** writes UserChoice, never bypasses UCPD.sys, never registers for `microsoft-edge:` to redirect it, and never re-asks after a "no" (UI R19). It does not ship an agent or a scheduled task.

**Reading the status** is `OsCommand::Query(DefaultStatus)` at broker start, on window activation after an `Ask`, and when the Settings page opens; never on a timer. The Settings row shows the truth per type ("Izcal opens web links; another app opens PDF files"), with "Change in Windows Settings" or equivalent. On Linux the lookup is Izcal's own implementation of the MIME-apps algorithm over the eight locations and `$XDG_CURRENT_DESKTOP`, not `xdg-settings`, whose own manual says its check can be wrong ([xdg-settings](https://portland.freedesktop.org/doc/xdg-settings.html)). The status is never sent anywhere (D-S4).

**Handlers inbound.** Every scheme or type Izcal is opened for arrives as an `OsInput::Launch` with origin `os-open` and follows 6.2's rules. Izcal registers no custom scheme of its own in Stage 1; an `izcal:` scheme would be a new inbound attack surface with no task that needs it.

### 6.5 Outbound: external protocols and opening files (OI-D9)

A page asks for `mailto:`, `tel:` or another app's scheme; the person opens a downloaded file; a link in a PDF names `file:`. All of these become `OsCommand::Launch` from the broker only.

1. **Scheme classes.** *Web* (http, https) never leaves Izcal. *Allowed external* (`mailto`, `tel`, `sms`, `webcal`, and any scheme an installed handler declares that is not blocked) goes through the external-protocol prompt class (SH 6.4, SH-R8). *Blocked* never reaches the OS from web content: `file`, UNC paths and any URL the OS would resolve to a remote share, `javascript`, `data`, `blob`, `ms-msdt`, `search-ms`, `search`, `ms-officecmd`, `ms-appinstaller`, the Office document schemes (`ms-word`, `ms-excel`, `ms-powerpoint` and their siblings, which open remote documents), `mhtml`, `ms-cxh*`, `shell`, `vbscript`, `jar`, and the OS's other script or shell schemes as a list maintained with the phishing bundle's release process (PH-D14) ([HP 8.4–8.6][HP]). A document with a `file:` origin counts as web content for these rules.
2. **Who may ask.** A top-level document with transient activation; never a cross-origin iframe, never a background tab, never a redirect chain without a new gesture (SH-R8). At most one external launch per 5 s† per (origin, container), and a second request while a prompt is open is refused.
3. **Indistinguishable outcomes.** The page sees the same result, at the same time, whether no handler exists, the person declined, the prompt is still open, or the launch happened: the navigation to the external scheme always completes as "no navigation" with no event, no focus or visibility change caused by Izcal, and no timing difference beyond the prompt's own constant delay; the prompt for a missing handler says "No app on this device opens these links" in the same window and time window as a real prompt. This closes the scheme-flooding signal at the browser's edge (OI-10). What the OS does after a launch (another app taking focus) is outside Izcal's control and is stated as a residual (Section 11).
4. **The launch.** Windows: `ShellExecuteEx` with the URL as a single parameter, from the broker. macOS: `NSWorkspace` open, which yields activation correctly ([MP 2.6][MP]). Linux: the OpenURI portal with an activation token, and `ask` for a scheme the person has not used before ([XDP-URI][XDP-URI]); with no portal, Izcal's own MIME-apps resolution and a direct spawn of the handler's `Exec` with field codes expanded per the Desktop Entry spec, never a shell.
5. **Opening a download.** Only after the person presses Open (SH 6.10, SH-R7). On Windows the broker writes the zone mark itself first, as SH-R6 requires; the OS shell helper then runs `IAttachmentExecute::Save` for the antivirus scan (PO-8.9) ([WP 14.1, 14.3][WP]), and the broker verifies afterwards that the mark is still present; a failing result is a "blocked by your antivirus or policy" verdict in the download row (OI-25). AMSI is not used for downloads: it targets script and dynamic content ([WP 14.2][WP]). On macOS the download writer sets `com.apple.quarantine` with the origin URL (SH-R6; which process may set it under Izcal's sandbox is measured, Q-OI-5). Linux has no common mark (SH-R6 states the residual).
6. **Click-to-call (BF-16.14).** `tel:` is an external scheme like any other: the OS's handler (for example a phone-link app) decides. Izcal runs no phone-linking service of its own (no account, Stage 1).

### 6.6 One OS-registration mechanism for the browser and installed apps (OI-D10)

FI-9's installed web apps need shortcuts, handlers, badges, notification identity and launchers, and the browser needs the same things for itself. One record type and one journal serve both.

```
OsRegistration {
  id, owner: Browser | App(app_id), profile,
  kind: Handler{scheme | mime | extension} | Launcher{name, icon, target} |
        JumpTask{id, label} | NotificationIdentity | Badge | LoginItem,
  created_by_version, consent_ref (ledger entry id, when a person approved it)
}
```

- **Realisers per OS** turn a record into OS artifacts and report back, in an `Answer`, the exact list written: registry keys and values, shortcut files, desktop entries, bundle paths, D-Bus names. The **journal** keeps the record and its artifact list. It is a small, versioned, append-only plain file per install and per profile root (`registrations.izj`, records in `os.hgidl` encoding with a checksum each), not a storage-service store, because FI-2's uninstaller must read it with no browser running; the broker and the uninstaller share its tier-P reader, and the installer writes its own install-time records into the same journal, so there is one source of truth for OI-11.
- **Removal** deletes exactly the journal's artifacts. Uninstalling Izcal runs every removal (with FI-2's uninstaller), and uninstalling an app runs that app's (OI-11).
- **Reconciliation** at broker start compares the journal with the OS. An artifact the person or the OS removed is marked "removed outside Izcal" and is **not** recreated silently; the Settings page offers to restore it.
- **Per-app identity for FI-9.** Windows: an AUMID and a Start-menu shortcut per app ([WP 3.3][WP]). macOS: notification authorisation, badges and Dock presence belong to a bundle, so each app needs its own small bundle, as Safari's web apps are ([MP 4.4][MP]); how such a bundle is signed on the user's machine is FI-9's open question (Q-OI-6). Linux: the DynamicLauncher portal's Webapp type with an ID prefixed by Izcal's app ID ([XDP-DL][XDP-DL]), or a desktop entry in `~/.local/share/applications` where the portal is absent.
- **Consent.** A record that changes what the person sees in the OS (a launcher, a login item, a handler for an app) needs a ledger entry (SH 6.4); the browser's own install-time records are the installer's.

The record, the journal and the realisers are this spec's. What an app may register, and the manifest that asks, are FI-9's.

### 6.7 Taskbar, dock and jump lists (OI-D13)

| | Windows | macOS | Linux |
|---|---|---|---|
| Identity | One AUMID per (channel, profile), `Izcal.Browser.<channel>.<profile>` (≤ 128 characters), set **per window** (a process has one explicit AUMID, which is the default profile's), matching the per-profile Start-menu shortcut; every window also carries `RelaunchCommand` (the stable launcher with `--profile`), `RelaunchDisplayNameResource` and a relaunch icon, so a person who pins a running window pins the launcher, not the versioned broker, and the pin survives updates ([WP 3.1–3.3][WP]) | The one bundle; profiles appear in the Dock menu | The channel's desktop entry |
| Quick actions | Jump-list tasks: New window, New private window | Dock menu: New window, New private window, the profile list ([MP 3.1][MP]) | Desktop actions `new-window`, `new-private-window` ([XDG actions](https://specifications.freedesktop.org/desktop-entry/latest/extra-actions.html)) |
| Recent items | **Off by default**: no Recent or Frequent category, no `SHAddToRecentDocs` for web pages, `FOS_DONTADDTORECENT` on dialogs ([WP 3.4][WP]) | No `noteNewRecentDocumentURL` for web pages | No `recently-used.xbel` writes for web pages |
| Progress and badge | `ITaskbarList3` progress for downloads | A custom Dock-tile view for download progress; badge label ([MP 3.2][MP]) | `com.canonical.Unity.LauncherEntry` count and progress where a listener exists ([LX 6.4][LX], Low) |
| Pinning | "Pin to taskbar" only on a person's request, through `TaskbarManager` on builds with KB5074105 ([MS-PIN][MS-PIN]); never from the installer | — | — |
| Window tabs | — | `NSWindow.tabbingMode = .disallowed`, so system tabs do not nest Izcal's ([MP 3.4–3.5][MP]) | — |

Recent items are off by default because they put browsing history into an OS surface other users and other apps can read; a person can turn them on per profile, never in a private window. Jump-list and Dock-menu labels are Izcal's own strings; no page title ever reaches them.

### 6.8 Share (OI-D11, OI-D12)

**Source.** The shell's Share action and `navigator.share()` produce one broker command with a broker-formatted title, a URL and optional files the page provided as `File` objects (Web Share's rules: transient activation, consumed; secure context; `web-share` policy; no target enumeration; the person confirms; the page cannot learn the target ([WS][WS])).

| OS | Sheet |
|---|---|
| Windows | `IDataTransferManagerInterop::GetForWindow` and `ShowShareUIForWindow` on the tab's top-level window; `SetWebLink` for URLs, storage items for files. Works for unpackaged desktop apps per Microsoft's 19 October 2026 guide, although the API reference still says "UWP apps only" ([MS-SHARE][MS-SHARE]; [WP 6.3][WP]) |
| macOS | `NSSharingServicePicker` anchored to the toolbar ([MP 6.1][MP]) |
| Linux | No share portal and no freedesktop share standard exist ([LX 3.19][LX]), so Izcal draws **its own sheet** in a broker-owned window: Copy link, Email link (through the `mailto:` handler and OpenURI), Save to file, QR code (FI-20), and Open in another app (OpenURI with `ask`) |

`navigator.share()` is exposed in Stage 1 on all three **if** the admission rule's two-engine bar is met on desktop (WA-12.6, DM-D15): the Recommendation's table lists Chromium and WebKit desktop support, but per-OS support in October 2026 was not established ([HP 5.2][HP]), so FI-37's admission line must cite a measured per-OS table, and until it does the API is held with this design ready. The title and text a page passes are page strings bound for an OS surface, so they go through G233's formatter (TX-16 amendment, 9.5). `canShare()` answers the same on every OS for the same data, so the sheet's platform differences are not observable beyond the user agent.

**Target** (other apps sharing to Izcal or to an installed app) is **Stage 2**, with FI-9: on Windows it needs package identity ([WP 0.1, 6.4][WP]), on macOS a sandboxed share extension that Gatekeeper blocks until the main app is approved ([MP 6.3][MP]). OI-D12 decides the Windows identity package. A macOS Services entry ("Open URL in Izcal") is the cheap Stage 1 path for receiving a URL from another app, and it arrives as an ordinary `Launch` input.

### 6.9 The Linux desktop: D-Bus, portals, Wayland and X11 (OI-D6, OI-D20, OI-D24)

**`hg-dbus` (OI-D6).** Izcal's own D-Bus implementation, written from the D-Bus specification 0.43 ([DBUS][DBUS]): the type system and marshalling, the message header, SASL EXTERNAL authentication, Unix file-descriptor passing, match rules, and both sides of the protocol, because most consumers export objects: method dispatch for exported objects, `org.freedesktop.DBus.Properties` with `PropertiesChanged`, `Introspectable` output, name ownership, and the gate's header filter (6.0). It is **sans-IO**: the codec and connection state machine take bytes and return bytes and events, as Q-CS-2 decided for HTTP/2, so CS-AS-4 needs no allowlist entry. The socket under it is a **local-stream handle** with readiness delivered as tasks and handle passing, a small substrate primitive proposed for M5 with a simulated implementation (Appendix C), which the hand-off listener, the SSIP client and AX's bridge also use. The codec is tier P, with Kani proofs of the header and marshalling bounds (messages ≤ 2²⁷ bytes per the spec, Izcal's cap 1 MiB†), and fuzz targets. **Interface descriptions** are hand-written by Izcal from the published prose documentation (the portal documentation, the freedesktop specifications), or, where no prose exists, from black-box `Introspect` output of running services, as ED already does for IBus and Fcitx; the machine-readable interface XML in other projects' repositories is never used or vendored (M-1 part 3), and each interface records its evidence chain. It serves every Izcal D-Bus client: the gate and this spec's roles, AX-D12's AT-SPI bridge, ED's IBus and Fcitx front ends, MD's portal and RTKit calls, and SH 6.12's Secret Service client. Its core (codec, auth, header filter) lands at **AX-M0's date**, because AX's bridge prototype is its first consumer; the server side and the gate land at OI-M1.

**Connections.** The broker's gate holds the only bus connections: the session bus (portals, notifications, MPRIS, LauncherEntry, StatusNotifierItem, the Secret Service) and the system bus (logind). It registers Izcal's app ID with `org.freedesktop.host.portal.Registry` once, first, before any portal call ([XDP-REG][XDP-REG]). The **long-lived subscriptions** that must be live whenever the browser runs (the Settings portal's `SettingChanged`, logind's `PrepareForSleep` and the delay lock, the Inhibit portal's session state) are the gate's own, decoded by the broker's closed decoders (6.0), so the desktop role is started only for portal requests, MPRIS while media plays, notifications and badges, and exits when idle. Every message forwarded to a role is untrusted input from another local program (IPC-R17), as AX-D12 already treats AT-SPI clients.

**Portals first, with a deadline.** Izcal uses portals as the primary desktop API even unsandboxed, as their documentation recommends ([XDP-WHY][XDP-WHY]): FileChooser (FI-5's OS side), OpenURI, Notification v2, Inhibit, Settings, Background, Print (FI-8), DynamicLauncher (FI-9), with feature detection by interface version. **Every portal call has a 1 s† timeout to its first reply**, and on timeout or absence Izcal takes the fallback listed below and records the portal state in `about:diagnostics`, because a missing backend otherwise stalls a caller for 25–26 s ([HP 3.7][HP]; OI-19). The timeouts and fallbacks are decided in `hg-os-broker` on the substrate clock (6.1). Startup waits on no portal beyond the first frame's ≤ 50 ms† theme read (6.1).

| Need | Portal | Fallback |
|---|---|---|
| File dialogs | FileChooser v4 | Izcal's own chooser in a broker-owned window (FI-5 designs it) |
| Open another app | OpenURI v5 | Own MIME-apps resolution and a direct spawn (6.5) |
| Notifications (Stage 2) | Notification v2, category `browser.web-notification` | Desktop Notifications 1.3 directly ([XDG-NOTIFY][XDG-NOTIFY]) |
| Idle inhibit | Inhibit (Idle) | Wayland `idle-inhibit` on the visible surface ([idle-inhibit](https://wayland.app/protocols/idle-inhibit-unstable-v1)) |
| Appearance | Settings v2 (`color-scheme`, `accent-color`, `contrast`, `reduced-motion`) ([XDP-SET][XDP-SET]) | Defaults ("unknown") |
| Background, autostart (Stage 2) | Background v2 | A desktop entry in `~/.config/autostart` |
| Installed-app launchers (Stage 2) | DynamicLauncher v1 | A desktop entry in `~/.local/share/applications` |
| Default browser | None exists ([LX 1.12][LX]) | Own MIME-apps implementation (6.4) |
| Media controls, badges, tray, speech | None exist ([LX 3.19][LX]) | MPRIS, LauncherEntry, SNI, SSIP directly |
| Secrets | The Secret portal is for sandboxed apps only ([XDP-SEC][XDP-SEC]) | The Secret Service API directly, as SH 6.12 says |

**Wayland first (OI-D20).** GNOME 50 and KDE Plasma 6.8 have no X11 session ([LX 7.1–7.2][LX]), so Izcal designs for Wayland and keeps X11 as a compatibility tier, as GR already does. The window owner (the broker, which holds GR's Wayland connection) uses: xdg-shell; **xdg-decoration to request server-side decorations wherever the compositor offers them, and Izcal-drawn client-side decorations where it does not, which includes GNOME's compositor** ([LX 7.5][LX]); fractional-scale-v1 with viewporter ([LX 7.6][LX]); xdg-activation-v1, both to raise its own window with the token a hand-off carried and to pass a token when launching another app ([LX 2.5][LX]); xdg-foreign to parent portal dialogs, which Hyprland and Weston lack, so dialogs are unparented there ([LX 7.7][LX]); xdg-toplevel-tag-v1 with a stable per-window tag so the compositor can restore placement ([LX 7.16][LX]); idle-inhibit; cursor-shape. **Izcal's client-side frame** is part of the shell document: a title area, close, minimise and maximise controls in the order the desktop uses (from the Settings portal where it says, the platform's convention otherwise), drag by `xdg_toplevel.move`, resize edges by `xdg_toplevel.resize`, the window menu by `show_window_menu`, double-click to maximise, tiling and maximised states drawn as the compositor reports them, and accessible names and roles from the semantic tree (AX), so the frame behaves like the platform's even though Izcal draws it (OI-26). The window geometry (`set_window_geometry`), the shadow margins and the resize-edge input regions are the broker's presentation work (a GR amendment, 9.5). **When the shell is down**, during a shell-renderer restart (SH 6.1, ≤ 2 s†) or a hang, the broker shows a **fallback frame**: a minimal close, minimise, maximise and drag strip with no text, drawn in its own subsurface from fixed vector shapes, so the window always stays movable and closable without the broker running any font code (TX-15). The native recovery window and the launcher's error window draw the same minimal frame themselves. Prompt windows on Wayland are popups (SH 6.5) and carry no frame. The alternative of a broker-drawn frame for every window was considered and rejected: the title needs text, and the broker runs no font code (TX-15).

**X11.** Startup notification for activation ([LX 2.4][LX]), EWMH for state, `Xft.dpi` for scale; no feature is designed for X11 first.

**Flatpak and Snap (OI-D24).** Not in Stage 1. Inside Flatpak a browser cannot create its own namespaces and must ask the Flatpak portal to spawn sub-sandboxes, which would replace Izcal's sandbox design ([LX 13.2][LX]). Because the host build already routes desktop interaction through portals, a later port is mostly a process-launch change; the security scope's 6.7 already records Flatpak as a reduced tier.

### 6.10 Media controls, power and wake locks (OI-D14, OI-D16)

**Media controls.** The broker keeps one OS media session per window that is playing audibly where the OS allows several (SMTC per window, if Q-OI-1 confirms it), and **arbitrates** where it does not (`MPNowPlayingInfoCenter` is one per app, MPRIS one player per process): the session shown and controlled is the focused window's if it plays audibly, otherwise the most recently started audible one. The session is owned by the broker and fed by the Media Session API's metadata and actions (MD 6.12), artwork decoded in the page's media process and passed as pixels, never as a URL (MD 6.12 "Total routing").

| OS | Mechanism |
|---|---|
| Windows | `ISystemMediaTransportControlsInterop::GetForWindow` on the tab's top-level window; metadata, playback state and timeline; hardware keys reach the active SMTC session ([WP 5.1–5.2][WP]) |
| macOS | `MPNowPlayingInfoCenter` with `playbackState` set on every change, which macOS requires ([MP 5.2][MP]); `MPRemoteCommandCenter` for commands |
| Linux | One MPRIS 2.2 player, `org.mpris.MediaPlayer2.<appid>.instance<pid>`, owned through the gate and served by the desktop role, artwork as a file in a broker-owned cache ([MPRIS][MPRIS]) |

Commands arrive as `OsInput::Media` and go to the tab that owns the session, as a Media Session action (play and pause work with no page handler, next and previous only with one). **Private windows** report "Izcal" and "Playing" with no title, artist or artwork, because SMTC and Now Playing show on the lock screen and to other apps (OI-18). A per-profile setting, "Show media details on the lock screen and to other apps", on by default for ordinary windows, applies the same anonymisation everywhere. Web Audio with no media element gets no OS session.

**Power events.** `Power` inputs come from `WM_POWERBROADCAST` and `RegisterSuspendResumeNotification` (Windows), `NSWorkspace` sleep and wake notifications and `isLowPowerModeEnabled` (macOS), and logind's `PrepareForSleep` and the Inhibit portal's session state (Linux) ([WP 7.5][WP]; [MP 7.3–7.4][MP]; [LX 8.2][LX], [XDP-INH][XDP-INH]). Before sleep the broker flushes the session store and pauses network refresh within the OS's window, through 6.1's deadline inputs: on Linux the broker's gate holds a logind **delay** lock for the whole time the browser runs, which logind honours for up to `InhibitDelayMaxUSec` (5 s by default), and releases it when the flush is done ([inhibitor locks](https://systemd.io/INHIBITOR_LOCKS)); macOS lets observers delay sleep for up to 30 s ([MP 7.3][MP]); the Inhibit portal's `QueryEnd` should be answered within 1 s ([LX 3.7][LX]); on Windows the broker's hidden window receives `WM_QUERYENDSESSION`. FP's background policy (FP 6.8) and its QoS table consume battery and Low Power Mode as inputs; this spec only delivers them.

**Wake locks.** Screen Wake Lock (FI-28 owns the API and permission) and audible or visible video become one `Hold` command per need, held by the broker: `PowerCreateRequest` with `DisplayRequired` plus `SystemRequired` and a localised reason ([WP 7.1][WP]); `IOPMAssertionCreateWithName` with `PreventUserIdleDisplaySleep` ([MP 7.1][MP]); on Wayland the window owner's `idle-inhibit` on the visible surface, which ends by construction when the surface is hidden, with the Inhibit portal as fallback. A lock is released within 1 s† of the page releasing it, the tab being hidden, the renderer crashing or the window closing (OI-21). A download in progress holds a system-sleep request (not display) only while the person keeps the window open; Windows ends such requests 5 minutes after the sleep timeout on battery on Modern Standby ([WP 7.1][WP]), and the download row says so if it happens.

### 6.11 Native dialogs, the OS side (OI-D21)

FI-5 owns the script dialogs, the form pickers and the file chooser's shell surface; this section owns how a file dialog reaches the OS.

| OS | Where and how |
|---|---|
| Windows | The **OS shell helper**: a medium-IL, same-user process launched on demand, with one STA thread, which shows `IFileOpenDialog` or `IFileSaveDialog` (folder picking with `FOS_PICKFOLDERS`, `FOS_FORCEFILESYSTEM`, `SetClientGuid` per purpose) owned by the window the request came from, and returns only the chosen paths, which the broker turns into typed handles (SH-R9); the helper exits 10 s† after its last dialog ([WP 9.1–9.3][WP]). Cross-process owner modality, focus return and DPI are measured at SH-M0 with the rest of Q-OI-1 |
| macOS | The broker shows `NSOpenPanel` or `NSSavePanel` attached to the tab's window; the panel already runs in a system process ([MP 9.1][MP]) |
| Linux | The FileChooser portal with the window's xdg-foreign handle; Izcal's own chooser where the portal is absent or times out (6.9) |

In every case the renderer receives broker-minted handles, never a path (SH-R9); the dialog's recent-places list never receives web content's file names.

### 6.12 Displays, DPI, multi-monitor and window management (PO-10.10, 10.16–10.18)

- **Scale.** Windows: Per-Monitor v2 in the manifest of every process that owns an HWND; use `WM_DPICHANGED`'s suggested rectangle, which keeps the cursor's relative position and avoids a resize loop ([WP 10.1–10.2][WP]); the UI host and broker declare the same awareness, because a cross-process parent of a different awareness resets a process ([WP 10.3][WP]). macOS: `didChangeBackingPropertiesNotification` per window ([MP 10.2][MP]). Wayland: fractional-scale-v1's preferred scale in 1/120 units. Each becomes a `Displays` input; GR-D17 records the scale as a rational, and the next frame after a move renders at the new scale (OI-16).
- **Placement restore.** The session store already keeps window bounds and display (SH 6.3). On Windows and macOS a window returns to the same display if it is present, and to the primary display's work area, fully visible, if not; full-screen windows return to full screen on their display. On Wayland Izcal restores size, maximised and full-screen state and full-screen output, and leaves position to the compositor through xdg-toplevel-tag; exact multi-monitor restore is a stated non-goal on Wayland until a placement protocol has compositor support ([LX 7.16][LX]; OI-17).
- **Window management.** The native frame keeps Snap Layouts, the window menu and title-bar accessibility on Windows (SH 6.2); if the shell ever draws a custom title bar, it must answer `WM_NCHITTEST` with `HTMAXBUTTON` over its maximise control ([WP 10.5][WP]). Mica and other backdrops are a Stage 2 shell choice on build 22621 and later ([WP 10.6][WP]). macOS full screen uses Spaces as the OS does; system window tabs are disabled (6.7).
- **Touch, pen and gestures (PO-10.17).** The window owner converts `WM_POINTER` messages, macOS magnify, rotate and swipe events, and Wayland's pointer-gestures and tablet-v2 events into Pointer Events input with pressure and tilt, delivered through `hg-input-route` (CON-R3); pinch zoom is GR's and LY-D13's, back and forward swipes are shell navigation gestures. No gesture is a consent input (SH 6.4).

### 6.13 Background mode and launch at login (OI-D17)

**Stage 1 has neither.** When the last window closes, the broker exits on Windows and Linux. On macOS, where apps conventionally keep running with no windows, the broker keeps its menu bar and Dock presence until the person quits, and runs no renderer and no network work meanwhile (FP-D4's hidden-means-frozen rule, applied to the whole browser). There is no preloading at sign-in: Edge's Startup Boost is the evidence that it surprises people ([HP 9.5][HP]).

**Stage 2**, with push and notifications (S1 13.2, SH 6.13): one setting, off by default, "Receive notifications when Izcal is closed", which keeps a **headless broker** with no windows, no renderers and no shell renderer, holding only the push connection and the notification adapter. It is visible: a tray icon on Windows (`Shell_NotifyIcon`, [WP 8.3][WP]), a menu-bar item on macOS, the Background portal's status line on Linux (no tray on GNOME, [HP 6.2][HP]). Launch at login is a separate opt-in, realised as an `OsRegistration` of kind `LoginItem`: the Run key (unpackaged) or `StartupTask` (with identity) on Windows ([WP 8.1–8.2][WP]), `SMAppService.mainApp` on macOS, whose login items the person sees in System Settings ([MP 8.1–8.3][MP]), and the Background portal's autostart or an autostart desktop entry on Linux. On macOS 26 and later the headless broker is exactly the case Apple prompts for, a background task that remains active after the person quits the app ([Apple Platform Deployment](https://support.apple.com/guide/deployment/manage-login-items-background-tasks-mac-depdca572563/web)), so the setting's explanation says the OS will ask. The headless broker's footprint is FP's to budget before Stage 2 (Section 9.5; Q-OI-7).

### 6.14 OS scripting and other OS actions (OI-D18, OI-D19)

**Owner decision (25 October 2026): full scripting behind consent and debug mode.** By default Izcal accepts only the OS's standard open and activate events. A full scripting surface, able to drive tabs, read pages and run page JavaScript, exists only when two independent gates are both open.

**By default.** Apple Events `GURL` and `odoc`, the Windows and Linux command line through the launcher, and (Stage 2) `org.freedesktop.Application`. These all become `Launch` inputs with the hand-off's rules and the inbound allowlist (6.2). No scripting dictionary is published, no COM class is registered, and no D-Bus scripting interface is exported, so a caller finds nothing to talk to.

**The threat.** Infostealers kill the browser and relaunch it in a debugging or automation mode to read cookies and pages (security scope AGT-R8); and once a person lets an app control a scriptable browser, macOS checks nothing else ([MP 12.2–12.4][MP]). A scripting surface is therefore the same asset as the automation endpoint that SEC-20 protects, and it gets the same three properties: nothing outside the person's own action can open it, every caller is approved individually inside Izcal, and no caller reaches the person's credentials or page content.

**Gate 1: debug mode.**
- **Where it lives.** Settings > Advanced > Developer > "Debug mode", per profile. **Off in every build by default**, Nightly and developer builds included.
- **How it turns on.** Only from that row, by the person: a high-risk prompt (CON-R1, broker-observed OS input, automation-synthesized input rejected) followed by a fresh OS authentication, as revealing a password does (SH 6.12). No command-line switch, environment variable, policy, import, profile file or extension can turn it on (AGT-R8). The stored flag is bound to the profile key held in the OS credential store (SEC-34), so a copied or edited profile file does not carry it; on Windows, where any same-user process can use DPAPI, this binding is not a boundary against same-user malware, and gate 2 is what answers that case.
- **How it is shown.** While on: a non-dismissable "Debug mode" chip in the toolbar of every window of the profile, the window title suffixed "— Debug mode" so that the OS's own window lists show it too, a line in `about:diagnostics`, and an entry in the prompt ledger for every turn-on and turn-off. It stays on until the person turns it off; turning it off revokes every scripting grant and unregisters the surface at once.
- **Enterprise.** A policy can force it off and hide the row; no policy can turn it on (amendment for the enterprise spec's EP-D11, 9.5).

**Gate 2: per-calling-app consent, inside Izcal.**
- **Identity.** The caller is identified by what the OS can verify: on macOS the Apple Event sender's audit token, resolved to its code signature (team identifier and bundle identifier); on Windows the COM client's process, resolved to its image path and Authenticode signer; on Linux the D-Bus sender's credentials through `GetConnectionCredentials`, resolved to its executable's path and content hash. Linux identity is weaker (no signature), and the prompt says so.
- **Consent.** The first call from an identity that has no grant opens an **OS scripting** prompt (a high-risk member of SH 6.4's agent and automation class) naming the app as the OS identifies it, its path, what scripting allows, and that the grant expires. This is in addition to the OS's own prompt (macOS TCC Automation consent for the sender). Approval needs broker-observed OS input with no synthesized events, as SEC-20 requires for automation consent.
- **Expiry: 8 hours†,** set from first principles: one working session of debugging fits inside it, and a forgotten grant lasts at most until the end of that day, which is the window an infostealer would otherwise inherit. No "always" scope exists. Grants are listed in Settings > Developer > Scripting apps and in the prompt ledger, each revocable on its own.

**What a scripting session can reach.** Two scopes, decided from SEC-20's rule that no external principal attaches to a user container:
- **Window and tab control in the person's profile.** List windows and tabs with their URLs and titles; open, close, activate and reload tabs; navigate a tab to an allowed URL (6.2's allowlist). No page content, no cookies, no storage, no credentials, no script. These are the actions a person could take with the keyboard, and the person has approved this app in Izcal.
- **Page access in a fresh ephemeral scripting container.** Reading the DOM, running page JavaScript and reading results happen only in tabs the scripting app opened itself, inside a fresh, ephemeral container created for that grant: no cookies, storage or credentials of the person's containers, and the container is discarded when the grant expires or is revoked. This is the model DT 6.3 already uses for pipe launches, so a script can do what a WebDriver client can, and nothing more. **The person's real containers are never scriptable at page level.** The alternative, page access in the real profile, was rejected: it is exactly the access the infostealer wants, and the two gates reduce its likelihood but not its impact.
- **Private windows are excluded entirely**: not listed, not controllable, not counted.

**How it is built.** One automation model, not a second one. The broker receives the OS-level calls (Apple Events through its `NSApplication`; COM calls on a class object it registers with the OS only while gate 1 is open; D-Bus calls on an interface the gate exports only while gate 1 is open), decodes them with a closed, tier-P command decoder (an addition to 6.0's recorded parsing exception), checks both gates, and forwards each command as a typed message to the **automation gateway** (S1 §1), which executes it as the equivalent WebDriver BiDi command through the IIP with that grant's capabilities (DT 6.3). The dictionaries (an `.sdef` on macOS, a type library on Windows, an interface description on Linux, all written by Izcal) are generated from one command table, each entry naming its BiDi equivalent. Every scripting call is logged locally with the caller, the command and the result, readable on the Developer page.

**Two gates against the threat.** Relaunching Izcal opens neither gate: no flag, environment or file enables debug mode or creates a grant. A same-user process that forges the debug flag (possible on Windows, gate 1 above) still meets gate 2, which needs a person-present approval that the broker observes and that rejects synthesized input where the OS lets Izcal tell. A process that passes both gates, because the person approved it, gets the person's tab list and URLs and an empty container to script, never cookies or page content from the person's containers. What remains is stated: a person can be talked into approving a malicious app; the prompt's wording, the 8 h† expiry and the visible chip limit how long that lasts.

**Staging.** Built at OI-M3, after the automation gateway and DT-5's BiDi server exist; the default (standard events only) holds from OI-M1.

**Wallpaper and similar shell actions (PO-8.13).** "Set as desktop background" and similar shell verbs are **refused**: a page image becomes an OS-level persistent change for a task the person can do by saving the image (FI-38 records the refusal).

### 6.15 OS speech voices (OI-D22)

FI-D5 puts read aloud through OS voices outside the AI switch, so OS speech synthesis must not start the ML service (AG-0). It runs in the OS-integration service's **speech role**, one instance per (site, container) of the speaking document (the PDF viewer's and the shell's read aloud use their own keys), at most 2† live, exiting 10 s† after its last utterance, because voices are third-party code on Windows (SAPI 5 voices) and other programs' services on Linux, and utterance text is page content (I-1):

| OS | Engine | Audio path |
|---|---|---|
| Windows | `Windows.Media.SpeechSynthesis.SpeechSynthesizer` (OneCore voices; "only Microsoft-signed voices") and SAPI 5 for third-party voices ([WP 12.2–12.3][WP]) | Synthesise to a stream; PCM goes to the audio service, so mute, output device and recording follow Izcal's audio rules |
| macOS | `AVSpeechSynthesizer` ([MP 13.3][MP]) | Buffer callbacks to PCM, to the audio service |
| Linux | Izcal's own SSIP client to the user's speech-dispatcher ([LX 11.1][LX]) | speech-dispatcher plays the audio itself, so tab mute, output-device choice and tab capture do not apply; Izcal implements mute as pause or cancel, relays index marks as boundary events, and states the difference for FI-29 |

Whether each engine runs under the service's sandbox is measured at OI-M2 (Q-OI-4; the notes found no documentation). FI-29 owns the Web Speech API, the voice-list fingerprinting rule and read aloud's product surface; this spec delivers voices and audio.

### 6.16 Notifications, the OS side (OI-D15, Stage 2)

SH 6.13 keeps the broker as the source of truth and the OS as a delivery channel. Delivery adapters, all fed by broker-formatted strings (TX-16):

| OS | Adapter | Click when Izcal is not running |
|---|---|---|
| Windows | Izcal's own COM activator, registered per user, with the profile's AUMID on its Start-menu shortcut; no Windows App SDK dependency ([WP 4.3][WP]); elevated processes cannot notify ([WP 0.6][WP]) | COM launches the launcher with an activation argument, which becomes a `Launch` input with origin `notification` |
| macOS | `UNUserNotificationCenter` from the broker; the OS prompts once, and later requests do not prompt ([MP 4.2][MP]); Izcal's per-site permission sits on top | The system launches Izcal and delivers the response |
| Linux | Portal Notification v2 with `category=browser.web-notification` and `app.*` actions ([XDP-NOT][XDP-NOT]); fallback Desktop Notifications 1.3 with the `desktop-entry` hint and the `ActivationToken` signal ([XDG-NOTIFY][XDG-NOTIFY]) | Portal: D-Bus activation through `org.freedesktop.Application.ActivateAction`; fallback: only while Izcal runs |

### 6.17 Credentials (no new design)

SH 6.12 stands: the broker uses the macOS data protection keychain (which needs provisioned entitlements and an app-like bundle, [MP 13.1–13.2][MP]), Credential Manager and DPAPI on Windows, and the Secret Service API on Linux, now through `hg-dbus` (OI-D6) instead of `zbus`. When no Secret Service is running, the Settings page says so and offers a passphrase rather than storing a key in the clear ([LX 10.5][LX]). FI-13 owns everything beyond storage.

### 6.18 Crates, layers and rules G230–G239

The layout follows the code as built and the coding standard's tiers (critique C2): the substrate traits carry bytes and handles, the typed layer sits above them, and ambient OS effects live only in `hg-substrate-os`.

| Layer | Crate | What it holds | Tier |
|---|---|---|---|
| L1 | `hg-substrate` (amended) | The `OsPort` trait, `OsCallId`, `OsSink`, `OsError`, `OsListening`; the local-stream primitive (M5) | As today |
| L1 | `hg-substrate-os` (amended) | Inventoried per-OS modules `os_win`, `os_mac`, `os_linux`: the `OsPort` production half, the realisers, the hidden broker window, the deadline-input callbacks, and a minimal **launch entry** (process start, pipe or socket, lock file) that the launcher links | O, as today (the only crate with ambient OS effects, G3) |
| L1 | `hg-substrate-sim` (amended) | The scripted OS model (M1) and the recorded `OsPort` (OI-M1) | R |
| L2s | `hg-launch-wire` | The IZLH codec, no dependencies | P |
| L2s | `hg-dbus` | D-Bus codec, auth, connection state machine, server side and the gate's header filter, sans-IO | P |
| L2s | `hg-xdg` | Desktop Entry, MIME-apps lookup and Exec field-code expansion, as pure functions over file contents | P |
| L2s | `hg-os-journal` | The registration journal's reader and writer, shared with FI-2's uninstaller | P |
| L2 | `hg-os-types` | `OsInput`, `OsCommand`, `OsRegistration`, generated from `os.hgidl` | R |
| L5 | `hg-os-broker` | The broker's consumers: launch policy and the inbound allowlist, default status, the journal and reconciliation, external-launch policy, portal timeouts and fallbacks, media arbitration, the gate's role allowlists | R |
| L5 | `hg-os-service` | The desktop and speech roles: body codecs for portals, notifications, MPRIS, LauncherEntry and SNI; the SSIP client; OS speech engines through `hg-substrate-os` | R |
| L6 | `hg-launch-main`, `hg-os-service-main`, `hg-os-shell-helper-main` | The launcher, the service and the Windows helper binaries | — |

- **G230.** Only `hg-substrate-os` calls OS integration APIs; `hg-os-service` and the helper reach them through it. A renderer, the shell renderer or the prompt renderer linking `hg-os-broker`, `hg-os-service` or the per-OS modules fails the graph lint.
- **G231.** `hg-launch-main` links `hg-launch-wire`, `hg-xdg`, `hg-os-journal` and `hg-substrate-os`'s launch entry only: no engine crate, no `hg-ipc`, no simulator. This is G3-consistent, because the launch entry is part of the one tier-O crate.
- **G232.** `hg-launch-wire`, `hg-dbus`, `hg-xdg` and `hg-os-journal` are tier P with no dependencies outside `core` and `alloc`, and every decoder entry point has a Kani harness and a fuzz target.
- **G233.** Every string that reaches an OS display surface (TX-16's closed list, 9.5: media metadata, notifications, share title and text, file-dialog titles and suggested names, the window title, the Dock badge) passes the broker's display formatter: typed field, length cap (≤ 256† code points for titles, ≤ 1,024† for bodies), TX's script check, fuzzed with the TX-12a corpus. Jump-list and Dock-menu labels are Izcal's own strings only.
- **G234.** No production code reads an OS setting, locale, time zone, hardware, display, power or default-status value except through `OsPort`; the exceptions are exactly those in 6.1 (enterprise policy through EP-D7's adapter, input events, presentation timing, NW's configuration reads until NW adopts `OsPort`, the clipboard); the `ast-grep` hook lists the OS calls.
- **G235.** The desktop role holds no site key, no credential and no profile data, and receives only G233-formatted strings; the speech role holds one (site, container)'s utterances only.
- **G236.** No sandboxed Izcal process holds a D-Bus socket; every bus message passes the gate's role allowlist.
- **G237.** Every inbound URL or path (hand-off, Apple Event, `Application.Open`, drag-in) passes `hg-os-broker`'s allowlist before navigation.
- G238–G239 are reserved for this spec.

### 6.19 Android (Stage 2), and what is considered but not designed here

**Android (with FI-33).** The OS edge has a different shape, so the types are checked against it now: intents opening a URL map to `Launch` with origin `os-open`; `RoleManager`'s browser role, which an app may request through a system dialog, maps to `OsCommand::Ask(DefaultBrowser)` and a `DefaultStatus` read; share targets map to FI-9's registrations; system media controls and wake locks map to `Media` and `Hold`. **Custom Tabs**, a bound service through which other apps drive browser sessions with message and engagement callbacks, is a cross-app trust boundary of its own; it is recorded as a Stage 2 security design input for FI-33 and is not covered by the hand-off, which carries no session channel.

**Considered, not designed here.**

| Item | Owner | Note |
|---|---|---|
| Restart after an update or reboot (`RegisterApplicationRestart`, macOS state restoration) | FI-2 with SH 6.11 | The launcher command line is the restart target; session restore is the shell's |
| Handoff and Universal Links on macOS | FI-35 (sync) and FI-9 | No account in Stage 1; Universal Links belong to installed apps |
| Native-messaging host manifests | EX | OS registrations Izcal reads and processes it spawns; they use G230's placement and the journal when EX designs them |
| Windows Family Safety blocking other browsers for child accounts | FI-38, FI-D15 | The OS owns family settings; Izcal honours them |
| `registerProtocolHandler` and manifest protocol handlers | FI-9 | Registered through 6.4's `Handler` record |

## 7. M-1 compliance

**Dependencies used unmodified.** None new. The OS interfaces are OS exceptions (HGBA-2 2.3.2): Win32 and WinRT APIs through the MIT `windows` crate the corpus already uses (ED, SH), AppKit and other Apple frameworks through the OS, the Linux kernel, D-Bus services and Wayland compositors as peers over their published protocols. Speech-dispatcher, gnome-keyring, KWallet, portal backends and notification servers are talked to over their protocols, never linked or shipped. The MPRIS specification text is LGPL-2.1 and the SSIP manual GFDL or GPL; implementing a protocol from its description links nothing ([LX 5.2, 11.2][LX]).

**Built new by Izcal.** The launcher and `hg-launch-wire`; `OsPort` and its simulator model; `hg-dbus`; `hg-xdg`; the registration journal and realisers; the external-launch policy; the OS-integration service and the Windows OS shell helper; the Linux share sheet; the client-side frame on Wayland.

**What each part is anchored to.**

| Part | Anchors |
|---|---|
| Default-browser registration and status | [Default Programs][MS-DP]; [Windows app defaults platform][MS-DEF]; [Launch the Default Apps settings page][MS-SET]; Apple's [`setDefaultApplication`][AP-SETDEF] and [`urlForApplication`][AP-URLFOR]; [MIME Applications Associations 1.0.1][XDG-MIME] |
| Inbound handlers and command line | [Registering an Application to a URI Scheme][MS-URI]; [Desktop Entry 1.5, Exec][XDG-EXEC]; `open(1)` |
| Single instance | [`org.freedesktop.Application`][XDG-DBUS]; [SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow); [xdg-activation-v1](https://wayland.app/protocols/xdg-activation-v1); Apple's [`createsNewApplicationInstance`](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance) |
| `hg-dbus` | [D-Bus specification 0.43][DBUS] |
| Portals | [xdg-desktop-portal documentation][XDP], per interface |
| Notifications, media, tray, speech, secrets | [Desktop Notifications 1.3][XDG-NOTIFY]; [MPRIS 2.2][MPRIS]; [StatusNotifierItem draft](https://specifications.freedesktop.org/status-notifier-item/latest/); the SSIP manual ([LX 11.1][LX], Q-OI-13); [Secret Service API](https://specifications.freedesktop.org/secret-service/latest/); Microsoft's and Apple's API pages cited in Section 6 |
| Share | [W3C Web Share][WS]; [Microsoft share guide][MS-SHARE]; Apple's `NSSharingServicePicker` |
| Wayland | Protocol definitions as rendered on wayland.app (specifications), cited in 6.9 |

**Evidence rule (M-1 part 3).** No research note or section cites any third-party project's source code. The notes record where search results offered browser source trees and were discarded unread ([HP][HP], [WP][WP], [MP][MP] headers); xdg-utils behaviour comes from its manual pages only; the xdg-desktop-portal GitHub releases page was read for release-note prose only. Two OD-7 items are open: the **SSIP manual** is a prose document whose only public URL redirects into a repository blob view, so the spec cites the note's row (LX 11.1) rather than the URL, and an outside copy (for example a distribution's packaged manual) is to be found before OI-M2 (Q-OI-13); and the notes disagree on **Mozilla's set-default page** (`firefox-source-docs`): the history note read it as prose with no outside copy, the Windows note left it out because Microsoft's pages cover the same facts. The spec takes no decision from it: its Windows design rests on Microsoft Learn ([MS-DEF][MS-DEF], [MS-DP][MS-DP]), and HP rows 1.4, 1.9 and 1.14 are history only. Phase 8 removed one citation of a GitHub pull request (a code review) from the Windows note (WP 4.5). **D-Bus and portal interface descriptions** are hand-written from prose or black-box introspection, never from other projects' repository XML (6.9). Values no prose states are black-box measurements (Section 11, Q-OI-1 to Q-OI-5).

## 8. Targets

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **OI-1** | Hand-off latency | Second launch to `accepted` ≤ 100 ms† p95; to the URL's tab created in the running browser ≤ 300 ms† p95 (DC-8, warm); measured over 1,000† launches | Lab harness per OS | Profile the launcher; it must not load more than the codec |
| **OI-2** | Version skew | This release's launcher against the last three Stable releases' brokers, and this release's broker against the last three Stable releases' launchers (about 12 weeks at FI-2's 4-week cadence, 6 weeks once it moves to 2 weeks; the Nightly channel, public from at least 3 months before the first Stable, runs the same matrix against the last three Nightlies): 100% `accepted` or a clean refusal with a message; 0 crashes; 0 waits > 2 s† | CI matrix from OI-M1 | Ship blocker for the release that broke it |
| **OI-3** | Hand-off codec | Kani proofs for every field decoder and the length arithmetic; 0 crashes in 24 CPU-hours† of fuzzing per release; every malformed input refused; the golden vectors decode | CI | Tier-P fix before merge |
| **OI-4** | No argument injection, inbound allowlist | A corpus of ≥ 500† crafted URIs and paths (quotes, trailing backslash, `%22`, leading `-`, newlines, NUL, over-long, internal schemes, `view-source:`, `blob:`, UNC and `file://host/` paths, paths on network mounts) through `ShellExecute`, the Run box, another app, `xdg-open`, the portal, Launch Services and drag-in: 0 switches honoured, 0 extra URLs opened, 0 operands outside the allowlist navigated, 0 remote paths opened without the prompt | Per-OS suite at OI-M1 | Ship blocker |
| **OI-5** | Single instance under storm | 1,000† concurrent launches: exactly 1 broker; every URL delivered exactly once (with queueing and `busy-retry`, 6.2); at most one error message per batch; in DST with the scripted OS model and on each OS | DST (OI-M0 skeleton, OI-M1 full) and lab | Fix before M5 exit |
| **OI-6** | Locks | After a broker crash, the next launch succeeds 100%; a live holder on another host produces the named-host message 100% and the lock is never deleted | Lab, including an NFS home directory | Fix before shipping on that OS |
| **OI-7** | OS-input replay | (a) DST: a scenario with launches, settings, display, power and deadline inputs replays 100/100† with identical trace hashes on three OSes; (b) production: a tab recording opened from an OS launch replays the tab's environment values, launch record and dialog answers with identical hashes, contains no other tab's or container's OS inputs (checked by a canary in another container), and makes 0 OS calls during replay (syscall probe) | DST farm; recording suite | DET-1 failure procedure; (b)'s canary failure is a ship blocker for recording |
| **OI-8** | Default status is the truth | For each OS, a matrix of ≥ 12† states (Izcal default, another browser, partial per type, EEA and non-EEA on Windows, no answer yet on macOS, a desktop-specific `mimeapps.list` naming another app on Linux): reported status equals the OS's own answer 100%; after the person's click the status reads `is-izcal` for the web types on Linux 100%; 0 writes to protected state; 1 OS prompt per person request on macOS | Lab per OS | Ship blocker (PO-8.1–8.3) |
| **OI-9** | Never nags | 0 default-browser asks after "Not now" across the three-version update test | SH-M7 audit | Ship blocker (UI R19) |
| **OI-10** | Scheme flooding closed | A 32-scheme probe run by a page, ≥ 10,000† trials: a one-sided binomial test at α = 0.05 does not reject accuracy ≤ 51%† for telling installed from not-installed handlers; a deliberately leaky build as positive control is detected with power ≥ 0.9†; 0 blocked schemes reach the OS; a `tel:` launch reaches the OS handler after consent (BF-16.14) | Lab | Ship blocker for external launches |
| **OI-11** | Clean removal | Install, register everything, uninstall: 0 Izcal artifacts left (registry, shortcuts, desktop entries, AUMIDs and relaunch properties, D-Bus activation files, login items, web-handler registrations); the same for an installed app (FI-9); a window pinned by the person still launches Izcal after an update | Snapshot diff per OS; update test | Ship blocker (PO-1.13 with FI-2) |
| **OI-12** | Footprint | Desktop role ≤ 2 MiB† charged when idle, 0 when not running, exits ≤ 10 s† after its last client; speech role ≤ 24 MiB† provisional while speaking, 0 otherwise, measured at OI-M2; OS shell helper ≤ 32 MiB† provisional while a dialog is open, 0 otherwise, measured at OI-M2; launcher ≤ 1 MiB†, alive ≤ OI-1's 100 ms† p95 when handing off; the broker's gate and listener inside the broker's ≤ 20 MiB† line; FP-1 unchanged on every OS | Footprint lab (FP-M) | The owning line's owner fixes it or records a decision |
| **OI-13** | Idle wakeups | With every Stage 1 integration registered, 0 additional wakeups per second at idle (FP-M0's 0/s stays 0) | `powermetrics`, ETW, `perf` | Fix before the next milestone exit |
| **OI-14** | Sandboxes and the gate | The OS-integration service: network connect denied, file open outside its granted paths denied, process creation denied, no bus socket reachable; through the gate, calls to `org.freedesktop.systemd1`, `org.freedesktop.secrets`, `Application.Open` and any undeclared tuple are refused 100%; renderers, the service, AppContainer and low-IL callers cannot connect to the hand-off endpoint; the OS shell helper has no network | Sandbox probe suite (SEC 6.7) plus the gate suite | Reduced tier shown; ship blocker on reference devices |
| **OI-15** | No page string on an OS surface unformatted | 100% of strings reaching TX-16's OS display surfaces come through G233's formatter, fuzzed with the TX-12a corpus | Fuzz and a probe of every OS call site | Ship blocker |
| **OI-16** | Mixed DPI | Moving a window between displays of different scales: ≤ 1† frame at the old scale after the OS's notification; the cursor's relative position kept (Windows); no resize loop | Lab with two monitors per OS | Ship blocker (PO-10.10) |
| **OI-17** | Placement restore | Windows and macOS: 100% restored to the same display when present; 100% fully visible on the primary display when it is not; Wayland: the stable tag set on 100% of toplevels, size and maximised or full-screen state restored 100%, full-screen output restored where present | Lab | Fix before SH-M5 |
| **OI-18** | Media keys | Key press to the Media Session action in the arbitrated tab ≤ 100 ms† p95, including the multi-window case of 6.10; private-window metadata never reaches the OS session (0 of 100† trials) | Lab | Fix; the privacy half is a ship blocker |
| **OI-19** | Portals never block | Startup with no portal backend within 50 ms† of startup with one; no user-visible operation waits > 1 s† for a portal; the first frame's theme is never wrong when a cached value exists | Lab on GNOME, KDE, Sway and a session with no portal | Fix before Linux ships |
| **OI-20** | Share privacy | `navigator.share()` without transient activation rejects 100%; a page cannot tell "no share target" from any other outcome, by OI-10's test | WPT `web-share` plus the classifier | Ship blocker for the API |
| **OI-21** | Wake locks | Released within 1 s† of release, hide, crash or close; 0 leaked OS assertions over 10,000† cycles | Lab, OS assertion lists (`pmset -g assertions`, `powercfg /requests`, logind's list) | Fix before FI-28 ships the API |
| **OI-22** | OS speech without AI | Read aloud through OS voices works with the AI switch off and the ML service count 0 (AG-0 holds); first audio ≤ 300 ms† warm and ≤ 1 s† with a cold speech role | AG-0 audit plus lab | Fix before FI-29 ships |
| **OI-23** | Sleep and session end | Session store flushed before sleep and before session end 100% within the OS's window, over 200† cycles per OS, including `WM_QUERYENDSESSION`, `applicationShouldTerminate:` and logind's delay lock | Lab | Fix |
| **OI-24** | `hg-dbus` | Kani proofs of header and marshalling bounds; the client, server and gate interoperate with the reference bus daemons and every portal, notification server and MPRIS host in the Linux matrix, 100% of the suite; the core by AX-M0 | CI plus the Linux lab | Blocks AX-D12, ED's Linux IME and this spec's Linux milestones (OI-D6's reversal) |
| **OI-25** | Download scan and origin mark | The EICAR test file is blocked with the "blocked by your antivirus or policy" verdict 100% on Windows; the origin mark is present on 100% of saved files on Windows and macOS after the scan | Lab per OS | Ship blocker (PO-8.9, SH-R6) |
| **OI-26** | Frame parity on Wayland without SSD | On GNOME, 100% of a checklist of title-bar behaviours (drag, double-click maximise, window menu, resize edges, tiling and maximised states, button order from settings, keyboard and AT operation of every control); the fallback frame appears within 100 ms† of a shell-renderer failure | Lab on GNOME and one SSD compositor | Fix before Linux ships |
| **OI-27** | Scripting gates | In a harness driving every scripting command on each OS: 100% of calls refused while debug mode is off; 100% refused from an app with no grant, an expired grant (after 8 hours†) or a revoked grant; 100% refused when policy forces it off; 0 page reads or script runs in any of the person's containers; 0 private windows listed; a debug flag written into the profile file without the Settings flow does not open gate 1 (macOS and Linux) and still meets gate 2 (Windows); consent approval by synthesized input rejected on every OS where SEC-20's suite runs | Per-OS harness and SEC-20's red-team suite, each release from OI-M3 | Ship blocker for the scripting surface |

## 9. Staging, milestones, staffing, decisions and amendments

### 9.1 Staging

**Stage 1:** the launcher, single instance and hand-off; command line; registration and default status; inbound and outbound handlers; the registration journal; taskbar, Dock and jump lists (without recent items); share as a source; media controls; power events and wake locks; dialogs' OS side; DPI, placement and window management; Linux D-Bus, portals and Wayland; OS speech voices (FI-D5). **Stage 2:** notifications delivery, background mode and launch at login, share target and the Windows identity package, the FI-9 registrations (app launchers, per-app identity, badges), Android's equivalents with the Android shell (FI-33). **Stage 1 late (OI-M3):** the scripting surface behind debug mode and consent (6.14). **Refused:** wallpaper and similar shell actions; page-level scripting of the person's own containers; preloading at sign-in.

### 9.2 Milestones

| ID | When | Exit |
|---|---|---|
| **OI-M0** | **M1 exit** | The `OsPort` trait and its four small types in `hg-substrate`; `os.hgidl` with `Launch`, `DefaultStatus`, `Settings`, `Locale`, `TimeZone`, `Hardware`, `Displays` and `Power` (the other arms reserved; enterprise policy is EP-D7's); the scripted OS model's skeleton in `hg-substrate-sim`; `hg-launch-wire` v1 with Kani harnesses and the golden vectors. Nothing else is added to M1, whose phases A to D2 have merged |
| **SH-M0 addition** | Before M2 | Q-OI-1: the window-scoped Windows calls from the broker, from the UI host under its real token, and with a broker-owned frame window (6.0) |
| **AX-M0 date** | With AX's bridge prototype | `hg-dbus`'s core: codec, auth, the header filter |
| **OI-M1** | M5 (process launch) | The launcher and the per-OS layout (with FI-2); rendezvous, server verification, locks, queueing; command line and the inbound allowlist; registration and default status; the recorded `OsPort`; the full launch-storm DST scenario and the fuzz fleet; the skew matrix (OI-2); OI-4's corpus; `hg-dbus`'s server side and the gate; the local-stream primitive |
| **OI-M2** | M6–M7 | External launches (OI-10); journal and removal (OI-11); taskbar and Dock; share source; media controls; power, deadline inputs and wake locks; the OS shell helper and dialogs (with FI-5); DPI and placement; the desktop role with portals; the speech role with OS voices (with FI-29); the Wayland frame (OI-26) |
| **OI-M3** | M8 | The scripting surface behind debug mode and consent (6.14, OI-27), on the automation gateway and DT-5's BiDi server; every Stage 1 OI target met or waived in the decision log |
| **OI-M4** | Stage 2 | Notifications, background mode, login items, share target, FI-9 registrations, the identity package, `DBusActivatable`, Android with FI-33 |

### 9.3 Staffing

| Role | Headcount | Work |
|---|---|---|
| Windows integration | 2–2.5† | Registration, the launcher's Windows half, the hidden broker window, AUMIDs and relaunch properties, SMTC, share, power, the OS shell helper, the UI-host report path, notifications (Stage 2) |
| macOS integration | 1.5–2† | Apple Events, Launch Services and the bundle layout, Dock, now-playing, panels, SMAppService, speech |
| Linux desktop | 1.5–2† | `hg-xdg`, portals, MPRIS, LauncherEntry, SSIP, Wayland window management and the client-side and fallback frames (with the shell) |
| D-Bus | 2–3† at the peak, 1† after OI-M1 | `hg-dbus` codec, server side, the gate and its role allowlists, interface descriptions, interoperability lab |
| Launcher, `OsPort` and determinism | 1† | `hg-launch-wire`, the launcher, the scripted model, the recorded `OsPort`, DST scenarios |
| Security and fuzzing | 1† | OI-4, OI-10, OI-14, the gate suite, threat models for the new process kinds, fuzz fleet |
| **Total** | **9–11.5† at peak** | About 35–45† engineer-years across M1–M8 and Stage 2 |

No saving from dropping `zbus` is booked here: AX's, ED's and SH's rows are theirs to re-cut when they adopt `hg-dbus`, and until they do, the **net Stage 1 addition is the gross figure, 7.5–9.5†** (the Stage 2 notification and identity work excluded), folded into Stage 1 8.3 as an "OS integration" track. The estimate is a planning assumption, re-cut at OI-M1.

### 9.4 Decisions

Each has a recommended default. A bare "yes" means the recommendation. ★ marks the owner-facing ones.

| ID | Decision | Recommendation | Cost | Reversal trigger |
|---|---|---|---|---|
| **★ OI-D1** | How OS-sourced inputs enter (audit §8.2) (**decided by the owner, 25 October 2026: "both, split by source"**) | **One broker substrate trait, `OsPort`**, carrying `os.hgidl` frames like `Ipc`, with the typed layer above; the inventory of 6.1 covers every value the OS delivers to the broker itself; enterprise policy takes the policy adapter's path (EP-D7) as typed snapshots with a 32-byte `PolicyDigest` (**owner decision, 25 October 2026: "both, split by source"**); helpers are on the OS side and replaced by the simulator's model. (Alternative: only adapter processes delivering snapshots over IPC; rejected because Apple Events, power and window broadcasts arrive in the broker's own process and would bypass it) | One trait, one `.hgidl` file, a scripted OS model | A measured DET-2 cost above budget at the trait boundary: make it a generic parameter, as 3.1 allows for `Clock` |
| **★ OI-D2** | The launch hand-off message (audit §8.3) (**accepted by the owner, 25 October 2026**) | **`hg-launch-wire` v1 (IZLH)**: fixed TLV with a critical bit, ≤ 64,512† bytes, ≤ 6† operands per request, server and client verified, Kani-proved, the only cross-build message Izcal accepts | One tiny crate and a skew matrix | A need to carry handles across builds; it would then gain a handle field, never `.hgidl` |
| **★ OI-D3** | Install and launch layout (**accepted by the owner, 25 October 2026**) | Per OS as in 6.2: a stable launcher on Windows and Linux, the broker as the bundle executable with versioned helper bundles on macOS; children always from the broker's own version; handed to FI-2 as a requirement | FI-2 lays out directories; one more binary | FI-2 picks a model that keeps one directory and still never replaces a running broker's children |
| **OI-D4** | Single-instance scope (**accepted by the owner, 25 October 2026**) | One broker per (user, session, channel, profile root); the profile is a message field (FI-D17); channels never hand off to each other; bursts queue; locks name host and PID and are never deleted silently | — | The owner reverses FI-D17 (a broker per profile): the address gains the profile |
| **★ OI-D5** | Process boundary (**accepted by the owner, 25 October 2026**) | Broker for OS-forced main-process APIs and the gate; a new sandboxed **OS-integration service** in two roles (desktop, speech) with no bus socket; a new Windows **OS shell helper** at medium IL for the Common Item Dialog and `IAttachmentExecute` | Two process kinds, two threat models | An OS forces a call into the window owner (SH-M0, Q-OI-1): that one call moves, with a typed endpoint |
| **★ OI-D6** | D-Bus (**accepted by the owner, 25 October 2026, as recommended**) | **Izcal's own `hg-dbus`**, sans-IO, client and server, from the D-Bus specification 0.43, with the gate, for every Izcal D-Bus client; AX, ED, MD and SH move off `zbus`; closes Q-MD-16 and Q-ED-17. Standing rules: build our own, open protocols first (FI-D14); `zbus` is documented as an asynchronous API, which CS-AS-4 forbids outside the substrate, the same reason Q-CS-2 chose an own HTTP/2 codec; and only an Izcal-owned codec can implement the gate in the broker. (Alternative: `zbus` everywhere, with a CS-AS-4 allowlist entry and a separate filter) | 2–3† engineers at the peak; core needed at AX-M0 | OI-24's core not met at AX-M0 with AX's bridge at risk: allowlist `zbus` for the AT-SPI bridge only, as a recorded exception, behind the gate |
| **OI-D7** | Default browser (**accepted by the owner, 25 October 2026**) | Register only what Izcal handles; ask only on the person's request through the OS surface (Windows deep link, one macOS http request, the user's `mimeapps.list` or the desktop-specific file the lookup uses on Linux, after a click); never write protected state; status is an `OsPort` input re-read at start and after an ask; shown in Settings only; no agent | — | An OS ships a supported set-default API with its own consent: use it |
| **OI-D8** | Command line and inbound URLs (**accepted by the owner, 25 October 2026**) | A closed, documented switch set; registrations use `--os-open --`; in that mode no switch is honoured and Windows takes exactly one operand; inbound is an allowlist (`http`, `https`, local `file:`, `about:blank`), internal pages are never opened from outside, and remote paths need a prompt; no proxy, flag or debugging switches in release builds | Some power-user switches missing | A documented task that needs a switch or scheme; it joins the set with a version note |
| **OI-D9** | Outbound launches (**accepted by the owner, 25 October 2026**) | Broker only; web, allowed and blocked scheme classes; gesture, top level, one per 5 s†; indistinguishable outcomes; the broker's zone mark, then `IAttachmentExecute::Save` before opening a download on Windows | A maintained blocklist | OI-10's test finds a residual signal: tighten timing or drop the prompt for missing handlers |
| **OI-D10** | OS registrations (**accepted by the owner, 25 October 2026**) | One `OsRegistration` record, a plain-file journal shared with the uninstaller, exact artifacts, reconciliation without silent re-creation, shared with FI-9 | One file format | FI-9 finds a registration the record cannot express: the kind enum grows |
| **OI-D11** | Share (**accepted by the owner, 25 October 2026**) | Source in Stage 1 on all three OSes (OS sheet on Windows and macOS, Izcal's own sheet on Linux); `navigator.share()` exposed in Stage 1 once FI-37's admission line cites a measured per-OS table; target in Stage 2 with FI-9 | Linux sheet | A freedesktop share standard or portal appears: use it on Linux |
| **★ OI-D12** | Windows package identity (**accepted by the owner, 25 October 2026**) | Stage 1 ships unpackaged (registry only). Stage 2 adds a signed identity package ("packaging with external location") for share target, startup tasks and identity-gated notifications ([WP 0.1–0.3][WP]) | A signing and registration step in FI-2's installer | Notifications prove reliable without identity on 24H2 and later (Q-OI-8) and share target is not wanted: skip the package |
| **OI-D13** | Taskbar and Dock (**accepted by the owner, 25 October 2026**) | Per-profile AUMIDs set per window, with relaunch properties pointing at the launcher; static quick actions; recent items off by default; badges and progress best-effort; pin only on request | — | A usability finding that people want recent pages in the jump list: per-profile opt-in exists already |
| **OI-D14** | Media controls (**accepted by the owner, 25 October 2026**) | The broker arbitrates one shown session (focused audible window, else the latest); private windows anonymised; a lock-screen setting for all windows | — | — |
| **OI-D15** | Notification adapters (Stage 2) (**accepted by the owner, 25 October 2026**) | Own COM activator (no Windows App SDK); `UNUserNotificationCenter`; portal Notification v2 with `DBusActivatable`, then Desktop Notifications 1.3 | — | The unpackaged COM path fails on a supported Windows build (Q-OI-8): use the identity package |
| **OI-D16** | Power (**accepted by the owner, 25 October 2026**) | Power and session events as `OsPort` inputs, deadline inputs with reply tokens; wake locks as broker-held OS assertions or Wayland idle-inhibit; on Linux a resident logind delay lock held by the gate | — | — |
| **★ OI-D17** | Background mode and login (**accepted by the owner, 25 October 2026**) | None in Stage 1; Stage 2 opt-in headless broker for push with a visible indicator (and macOS 26's background-task prompt); launch at login a separate opt-in; preloading refused | People who expect notifications from a closed browser must opt in | Measured demand and FP's budget for the headless broker (Q-OI-7) |
| **★ OI-D18** | OS scripting (**changed by the owner, 25 October 2026**: full scripting behind consent and debug mode) | By default only the standard open and activate events. A full scripting surface (Apple Events dictionary, COM automation, a D-Bus interface) exists only while **debug mode** is on (per profile, off in every build, turned on only in Settings with a high-risk prompt and a fresh OS authentication, shown by a toolbar chip and the window title) **and** the calling app holds an in-Izcal grant (verified identity, high-risk prompt with no synthesized input, revocable, listed, 8 hours†). Window and tab control in the person's profile; DOM reads and page JavaScript only in a fresh ephemeral container per grant; private windows excluded; executed as WebDriver BiDi commands through the automation gateway; an enterprise policy forces it off | A second, OS-native automation entry point to maintain; scripts cannot read the person's real pages | A measured need for page access in real containers that a design can meet without exposing cookies or content to the caller; or OI-27 failing |
| **OI-D19** | Wallpaper and similar shell actions (**accepted by the owner, 25 October 2026**) | Refused (FI-38) | — | — |
| **★ OI-D20** | Window frame on Wayland (**accepted by the owner, 25 October 2026**) | Request server-side decorations where offered; **Izcal-drawn client-side decorations where not (GNOME's compositor)**, in the shell document with platform order and behaviour, plus a text-free broker fallback frame for shell outages; this amends SH 6.2's "the OS draws the frame". (Alternatives: refuse to run without SSD, unacceptable on the largest desktop; another project's decoration library, which M-1 and FI-D14 rule out; a broker-drawn frame for every window, which would need font code in the broker, TX-15) | The shell owns a frame on Linux; the broker draws a minimal fallback | GNOME implements xdg-decoration: use SSD there |
| **OI-D21** | Native dialogs (**accepted by the owner, 25 October 2026**) | Windows: the OS shell helper; macOS: the broker's panels; Linux: the FileChooser portal with Izcal's own chooser as fallback | One helper process | — |
| **OI-D22** | OS speech (**accepted by the owner, 25 October 2026**) | In the speech role, per (site, container); PCM to the audio service where the OS gives it; never the ML service | — | The OS engines cannot run sandboxed (Q-OI-4): a separate medium-IL speech helper on that OS, still per (site, container) |
| **OI-D24** | Flatpak and Snap (**accepted by the owner, 25 October 2026**) | Not in Stage 1; reconsider in Stage 2 | No Flatpak build | Distribution demand; portals are already the path |
| **★ OI-D25** | The desktop role as a shared process (**accepted by the owner, 25 October 2026**) | Record the desktop role as a **fifth shared, site-less exception** to I-1 (beside OD-8's four), because MPRIS metadata, notification text and badges are per-browser OS objects: it receives only G233-formatted, length-bounded strings that the OS then publishes to every local program anyway, holds no bus socket, no network and no files except the artwork cache, and gets its own FP-7 line and threat model. (Alternative: one desktop instance per (site, container), which multiplies MPRIS players and notification identities the OS shows as one app) | An owner-approved I-1 exception | A design that gives the OS one player without a shared process |
| **OI-D26** | What a production recording contains (**accepted by the owner, 25 October 2026**) | A tab recording carries only the OS-derived values that tab consumed, joined by ID (REC-R6); the full `OsPort` stream is simulation's; no browser-wide OS recording in Stage 1 | Some OS-triggered bugs replay only in DST | A browser-wide recording mode with its own privacy statement, if bug reports need it |
| **OI-D27** | Linux rendezvous in Stage 1 (**accepted by the owner, 25 October 2026**) | One path, the Unix socket; `DBusActivatable` waits for Stage 2's notification actions, with a specified activation file | No D-Bus activation in Stage 1 | Stage 2 notifications (planned) |

OI-D23 (portal timeouts and fallbacks) was folded into 6.9 as a rule, not a choice.

### 9.5 Amendments to other documents (proposed, not applied)

| Document | Section | Amendment | Source |
|---|---|---|---|
| Stage 1 design | §1 process map | Add the **OS-integration service** (desktop role: 0 or 1, site-less, a fifth shared exception under OI-D25; speech role: 0–2 per (site, container); sandboxed, utility tier, no bus socket, no network) and the **OS shell helper** (Windows; 0 or 1; medium IL; on demand); state the **launcher** as the installation's stable entry point (the "small platform launcher" SH 6.1 names); and **align the OS-adapter row with NW 6.1**: one per container, on demand, running OS calls that parse network data (resolver fallback, SSPI and GSSAPI, client-key signing), while configuration reads stay in the broker (the row now says "1 per OS source … reads … network configuration") | OI-D3, OI-D5, OI-D25; fact-check 72 |
| Stage 1 design | §1, I-1 part 3 | The shared-process exceptions become five, with the desktop role's justification and FP-7 line | OI-D25 |
| Stage 1 design | 3.1 | Add `pub os: Option<Arc<dyn OsPort>>` to `Substrate` and the trait of 6.1 here, present in the broker only; add the local-stream primitive at M5 | OI-D1 |
| Stage 1 design | 3.4 | Production recordings carry only the OS-derived values a tab consumed; the full OS stream is simulation's | OI-D26 |
| Stage 1 design | 4.1 | "Serialization (VER-1)": `hg-launch-wire` is the one message format accepted across builds, outside `.hgidl` | OI-D2 |
| Stage 1 design | 8.1 | M1 exit gains OI-M0 as stated in 9.2 (trait, types, scripted model skeleton, IZLH golden vectors only); SH-M0 gains Q-OI-1; M5 gains OI-M1 | 9.2 |
| Stage 1 design | 2.1, 2.2 | Register the crates of 6.18 and G230–G239 | 6.18 |
| Stage 1 design | §6 | Rows: "Single instance and hand-off", "Share", "Media controls", "OS integration" (pointing here); "Default-browser handling" cites OI-D7 | 6 |
| Stage 1 design | 8.3 | An "OS integration" track, 7.5–9.5† net in Stage 1 | 9.3 |
| HGBA-3 | DET-1 | OS inputs are a recorded input class, delivered through `OsPort` (as the devices spec did for its input class) | OI-D1 |
| Coding standard | 6.2 tier table, G3 | `hg-substrate-os`'s per-OS integration modules and launch entry are inventoried tier-O code; `hg-launch-wire`, `hg-dbus`, `hg-xdg` and `hg-os-journal` are tier P in L2s; the launcher links only the launch entry | 6.18 |
| Fonts and text | TX-16 | A closed list of OS display surfaces (media metadata, notifications, share title and text, file-dialog titles and suggested names, the window title, the Dock badge), each receiving only G233-formatted, length-bounded, script-checked strings, owned by TX and fuzzed with TX-12a; the shell's window title falls under the same rule | G233; critique C7 |
| Shell | 6.1 | The "platform adapter behind `hg-window`" collides with the DOM's `hg-window` crate (S1 2.1 L4, DM): name the shell's window adapter `hg-os-window` | Naming |
| Shell | 6.2 | "Native window frame": the OS frame where the OS provides one; on Wayland compositors without server-side decorations, Izcal draws the frame in the shell document, with the broker's text-free fallback frame during shell outages (OI-D20) | OI-D20 |
| Shell | 6.3 | Add `shell.os`: default status per type, the registration journal (with restore), login-item and background-mode state (Stage 2), the lock-screen media setting, and the OS-integration diagnostics (portal versions, gate refusals) | 6.4, 6.6, 6.10 |
| Shell | 6.4 | External-protocol class: indistinguishable outcomes, the scheme classes and the 5 s† rate limit of 6.5; the missing-handler variant of the prompt; a remote-path prompt class for inbound UNC and network-mount opens | OI-D8, OI-D9 |
| Shell | 6.11 | Default browser: mechanics per OS and the status row as in 6.4 here | OI-D7 |
| Shell | 6.12, §7 | The Secret Service client uses `hg-dbus` through the gate, not `zbus` | OI-D6 |
| Shell | 6.13 | Stage 2 delivery through the adapters of 6.16 here | OI-D15 |
| Shell | 6.14, SH-D22 | The recovery window and the launcher's error window draw the minimal frame on Wayland compositors without server-side decorations | OI-D20 |
| Shell UI design | R19 | Cite OI-9 as R19's measurement | OI-9 |
| Security scope | 6.3a, 6.5, 6.7, the counts | Rows and threat models for the OS-integration service (two roles; Utility sandbox; no bus socket; no network; no files except the artwork cache) and the OS shell helper (medium IL, no network); the launcher under TB-10; the process-kind and threat-model counts (thirty-seven and thirty-nine) rise by two | OI-D5 |
| Security scope | IPC-R15 note | The broker's recorded parsing exception: the IZLH frame, D-Bus headers in the gate, and four closed bodies (Secret Service replies, `SettingChanged`, `PrepareForSleep`, and Stage 2's `org.freedesktop.Application`), each tier P with Kani, fuzz and adversarial suites | 6.0; critique C8 |
| Security scope | SH-R6 | On Windows the broker writes the zone mark, then the OS shell helper's `IAttachmentExecute::Save` scans, then the broker verifies the mark | 6.5; critique C21 |
| Security scope | SH-R8, TB-10 | SH-R8 gains 6.5's scheme classes and indistinguishability; TB-10 gains the inbound rules of 6.2 and 6.3 (end-of-options marker, one operand on Windows, the inbound allowlist, remote-path prompts, server verification) | OI-D8, OI-D9 |
| Security scope | AGT-R8, SEC-20 | The hand-off format has no automation intent; `--automation-pipe` and `--headless` never hand off | 6.2, 6.3 |
| Security scope | AGT-R8, SEC-20, IPC-R15 note | AGT-R8 gains OS scripting: no flag, environment variable, file or policy enables debug mode or creates a scripting grant; scripting grants never reach a user container's page content (window and tab control only); SEC-20's target extends to the scripting surface (OI-27); the broker's recorded parsing exception adds the closed scripting-command decoder | OI-D18 |
| DevTools | 6.3, 6.5 | The automation gateway accepts a third client kind, an OS scripting caller, with a per-grant ephemeral container (as pipe launches have) and the window-and-tab capability set in the person's profile; the scripting dictionaries are generated from a table mapping each command to its BiDi command | OI-D18 |
| Enterprise policy | EP-D11 (DevTools and automation policies) | A policy that forces OS scripting off (and hides the debug-mode row); no policy can turn it on | OI-D18 |
| Shell | 6.4, 6.9 | The **OS scripting** prompt (high-risk, agent and automation class) and the debug-mode prompt; Settings > Advanced > Developer with "Debug mode" and "Scripting apps"; the debug-mode toolbar chip and window-title suffix | OI-D18 |
| Security scope | REC-R6 and the recording header (FIA §8.1) | OI-D26's recording scope | OI-D26 |
| Security scope | 6.5 UI-host model | The UI host's typed per-window reports (DPI, display moves) to the broker; and, if SH-M0 requires, the one endpoint per window-scoped call | 6.0; critique C4, C13 |
| Accessibility | 6.11 | `hg-ax-prefs` takes its raw OS values from `OsPort`'s `Settings` input, whose field set is a superset of 6.11's sources; its record is unchanged | OI-D1 |
| Accessibility | §7, AX-D12, AX-M0 | The AT-SPI bridge uses `hg-dbus`, reaching the session bus through the gate (and the accessibility bus as AX decides); the `zbus` and `zvariant` row is removed; `hg-dbus`'s core lands at AX-M0 | OI-D6 |
| Accessibility | Shell frame | The Wayland client-side frame's controls and the fallback frame are exposed with roles and names | OI-D20 |
| Editing | 6.5, §7, Q-ED-17 | IBus and Fcitx front ends over `hg-dbus`; Q-ED-17 closes | OI-D6 |
| Editing | 6.11 | Dropped URLs and files meet 6.2's inbound allowlist and the remote-path prompt | OI-D8 |
| Images and media | 6.12, Q-MD-16 | Media Session reaches the OS controls by 6.10 here; MD's D-Bus client moves from `hg-substrate-os` to `hg-dbus` behind the gate for portal and RTKit calls; Q-MD-16 closes | OI-D6, OI-D14 |
| Graphics | Wayland presentation | The broker's Wayland duties for this spec: xdg-activation, xdg-toplevel-tag, idle-inhibit, xdg-foreign export, `set_window_geometry`, shadow margins and resize input regions for the client-side frame, and the fallback frame's subsurface | OI-D20 |
| Resource footprint | 8.2, FP-7 | Rows: desktop role ≤ 2 MiB† idle, 0 when not running, 10 s† exit, as a shared-process line on FP-7; speech role ≤ 24 MiB† provisional while speaking; OS shell helper ≤ 32 MiB† provisional while open; launcher ≤ 1 MiB† transient. FP-1 unchanged on every OS (the gate's subscriptions live in the broker's line). A Stage 2 line for the headless broker (OI-D17) | OI-12 |
| Storage | — | No change: the registration journal is a plain file, not a store (6.6) | 6.6 |
| Phishing protection | PH-D14 release process | Ships the outbound scheme blocklist of 6.5 with the phishing bundle | OI-D9 |
| Extensions | 6.1 (manifest keys) | Replace "Stage 3, with the OS-integration design" by "Stage 3; OS registration through `OsRegistration` (*Izcal OS integration* 6.6) and FI-9" (closes C17's OS-integration item); native-messaging host manifests use G230's placement when designed | OI-D10 |
| DOM | 6.16, 6.18 | The environment-values record reads `Displays`, `Locale`, `TimeZone` and `Hardware` from `OsPort`; Screen Wake Lock's OS side is *Izcal OS integration* 6.10; Web Share is exposed per OI-D11, with its admission line in FI-37's table | OI-D1, OI-D11, OI-D16 |
| Networking | 6.1 | Proposed: the broker's OS configuration reads (proxy, DNS, network change) arrive through `OsPort`, so the OS edge has one recorded boundary | OI-D1 |
| HTML/CSS style | 6.10 | `AccentColor` system colour: a fixed value for pages under D-S1 (Q-OI-9) | 6.1 |
| Feature inventory audit work items | FI-2, FI-3, FI-5, FI-8, FI-9, FI-11, FI-13, FI-28, FI-29, FI-33, FI-38 | FI-2: the per-OS layout of 6.2 as a requirement, the shared journal, the Stage 2 identity package, platform floors (Windows 11 22H2 and later; macOS 26 the last Intel release); FI-3: policy arrives through the enterprise spec's policy adapter (EP-D7), not `OsPort`; FI-5: the dialogs' OS side is 6.11 here; FI-8: the Print portal; FI-9: build on `OsRegistration`; FI-11: recordings follow OI-D26; FI-13: no change; FI-28: wake-lock and gesture backends here; FI-29: OS voices and the Linux audio residual here; FI-33: 6.19's Android mapping and Custom Tabs; FI-38: record the wallpaper refusal | 6 |

## 10. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Window-scoped Windows calls work neither from the broker on the UI host's HWNDs nor from the UI host under its token | Medium | Taskbar, SMTC or share fail on Windows, or SH-D3 reopens | Measured at SH-M0, before M2 (Q-OI-1), with the broker-owned frame window as a third option and the security amendment pre-recorded |
| Windows changes default-app rules again (as in 2012, 2015, 2021, 2024) | High | The ask path breaks | Only documented paths; OI-8's matrix every release |
| `hg-dbus` or the gate slips and blocks AX-D12 and ED's Linux IME | Medium | Linux accessibility and IME late | The core lands at AX-M0; OI-D6's reversal: a recorded `zbus` exception for the bridge only, behind the gate |
| The gate's role allowlists are too narrow for a desktop's real traffic, or too wide | Medium | Features break, or a role regains ambient authority | Refusals counted in `about:diagnostics`; allowlists versioned and reviewed per release; OI-14's denial cases |
| Portal backends missing or old (Ubuntu LTS) | High | Features degrade on Linux | Timeouts, fallbacks, diagnostics (OI-19); Q-OI-10 |
| GNOME users judge Izcal's frame non-native | Medium | Perceived quality | Platform order and behaviour (OI-26); review with GNOME users at SH-M5 |
| Wayland gives no placement protocol | High | Multi-monitor restore weaker on Linux | Stated non-goal; xdg-toplevel-tag; adopt a placement protocol when compositors ship one |
| The macOS helper-bundle layout conflicts with notarisation or with an in-place update by the updater | Medium | Children fail to start after an update on macOS | FI-2 validates the layout at OI-M1 with a real update under a running broker |
| macOS notification and Dock identity per bundle makes FI-9 apps hard | Medium | Installed apps lack identity on macOS | FI-9's question (Q-OI-6) |
| OS speech engines need more than a sandbox allows | Medium | A medium-IL speech helper | OI-D22's reversal, still per (site, container) |
| The owner declines the desktop role as a fifth shared exception (OI-D25) | Low | Per-(site, container) desktop instances; several MPRIS players and notification identities | OI-D25's alternative is designed |
| The headless broker for push misses FP's budget | Medium | Stage 2 notifications delayed | FP line before Stage 2 (Q-OI-7) |
| Users rely on switches Izcal refuses | Low | Complaints | Documented set, reasons in `--help` |

## 11. Open questions and flagged claims

**Open questions (black-box measurements and decisions).**
- **Q-OI-1.** Which window-scoped Windows calls work (a) from the broker on the UI host's HWNDs, (b) from the UI host under its real token, (c) with a broker-owned top-level frame window and the UI host's content window as its child: `ITaskbarList3`, `SHGetPropertyStoreForWindow` (AUMID and relaunch properties), SMTC's `GetForWindow`, `ShowShareUIForWindow`, and the OS shell helper's dialog owned by that window (modality, focus return, DPI). **SH-M0, before M2**, rig253. Pre-recorded consequences: (a) changes nothing; (b) gives the UI host one typed endpoint per call and a SEC 6.5 amendment; (c) changes SH-D3's window ownership and needs the owner.
- **Q-OI-2.** The macOS rendezvous path: a Unix socket under the per-user temporary directory versus a Mach service; `sun_path` length on macOS. OI-M1.
- **Q-OI-3.** Other browsers' refusal sets for command-line URLs, read black-box, as a cross-check of 6.2's allowlist (not as its source). OI-M1.
- **Q-OI-4.** Whether `SpeechSynthesizer`, SAPI 5, `AVSpeechSynthesizer` and the SSIP socket work under the speech role's sandbox on each OS ([WP 12.4][WP]; [MP 13.4][MP]; [LX 11.3][LX]). OI-M2.
- **Q-OI-5.** Which Izcal process may set `com.apple.quarantine` under its sandbox profile, and what Gatekeeper shows for the origin ([MP 15.1][MP]). OI-M2.
- **Q-OI-6.** How an installed app's macOS bundle is signed and registered on the user's machine (FI-9).
- **Q-OI-7.** The headless broker's footprint for Stage 2 background push, and whether it fits FP-1's always-on budget (audit §8 "Honest limits"); it will meet macOS 26's background-task prompt.
- **Q-OI-8.** Whether a hand-registered COM toast activator works for an unpackaged app on Windows 11 24H2 and 25H2 on a cold start and under Focus ([WP 4.3–4.4][WP]).
- **Q-OI-9.** Whether CSS `AccentColor` should expose the user's accent to pages (the style spec's decision under D-S1).
- **Q-OI-10.** The portal version on the oldest supported Linux LTS, and the app ID a host app gets without `Registry` ([LX 14.2][LX]).
- **Q-OI-11.** Whether GNOME 50 and later honour `xdg-toplevel-tag-v1` for restoring position and size ([LX 14.1][LX]).
- **Q-OI-12.** The application identifier (reverse-DNS app ID, AUMID prefix, bundle identifier) per channel: FI-2's to fix before OI-M1.
- **Q-OI-13.** An outside copy of the SSIP manual (OD-7 condition 2), before OI-M2.
- **Q-OI-14.** The supported OS floors (Windows 11 22H2 and later as the notes recommend; macOS 26 the last Intel release, 27 the last with Rosetta for apps) belong to FI-2 and the graphics spec's GR-D21; this spec's deep link, pin API and Mica rows assume them.

**Claims that are single-source, unverified or conflicting.**
- **UCPD.sys's dates and key list.** Microsoft's page confirms the driver and that it "blocks apps from writing app defaults data", but names no keys and no date; the http, https and .pdf list and the February 2024 date rest on a vendor support note (whose link no longer resolves) and press, and the press dated its discovery to April 2024 ([WP 1.8][WP]; [HP 1.14][HP]).
- **"Over the Edge 2.0" (July 2026)**, that a Windows Backup restore resets the default to Edge, was read only through press coverage ([HP 1.20][HP], Low).
- **The macOS prompt's origin** is dated 10.10 by a 2014 source and about 10.13 by a 2026 blog ([HP 2.1–2.2][HP]; [MP 1.5][MP]); this spec relies only on the prompt existing.
- **macOS 26.4's per-UTI prompts** come from one expert blog (Medium); Apple's 26.4 release notes, checked by the fact-checker on 24 October 2026, do not mention it. The design asks for http once either way.
- **Desktop Web Share support per browser and OS in October 2026** was not established ([HP 5.2][HP]); the API's exposure waits on FI-37's measured table (OI-D11).
- **Chrome 77's removal of the "always open" checkbox** has no Chrome source in the notes ([HP 8.8][HP], Medium).
- **The LauncherEntry API's 2026 support** could not be read from a primary source ([LX 6.4][LX], Low); badges on Linux are best-effort for that reason.
- **The portal backend coverage table** comes from a community wiki that looks partly stale ([LX 3.17][LX], Medium).
- **That `zbus` is asynchronous** is taken from its documentation's description and the editing spec's Q-ED-17, not from a measurement of its dependency closure.
- **Microsoft's share documentation conflicts with itself**: the 2026 guide says unpackaged desktop apps can share; the API reference says "UWP apps only" ([WP 6.3][WP]). The design follows the guide and OI-M2 tests it.
- **Residuals Izcal cannot control.** What the OS does after a successful external launch (another app taking focus) can still be observed by a page through focus and visibility changes (OI-10). The FileChooser portal's backend may record chosen files in the desktop's recent-files list, and the Windows shell may record files Izcal opens through `ShellExecute`. Normal-window media metadata appears on the lock screen unless the person turns the 6.10 setting off.

## Appendix A. Phase 8 disposition

### A.1 Design critique (`critique.md`, 40 findings)

| ID | Severity | Disposition | Where |
|---|---|---|---|
| C1 | Critical | Applied: no sandboxed process holds a bus socket; the broker's `hg-dbus` gate forwards only declared tuples; OI-14 calls `systemd1`, `secrets` and `Application.Open` and expects refusal; AX and MD amendment rows | 6.0, 6.9, G236, OI-14, 9.5 |
| C2 | High | Applied: `OsPort` carries frames like `Ipc`; types in L2; per-OS halves as inventoried tier-O modules of `hg-substrate-os`; the launcher links its launch entry; pure crates in L2s; a coding-standard row | 6.1, 6.18, 9.5 |
| C3 | High | Applied: the OS-input inventory with `Locale`, `TimeZone`, `Hardware`; `Settings` a superset of AX 6.11; G234's named exceptions. Enterprise policy was first added as an `OsPort` kind; the owner's 25 October 2026 decision routes it through EP-D7's policy adapter instead | 6.1, G234, App. C |
| C4 | High | Applied: the broker's hidden top-level window and windowless sources; UI-host per-window reports tagged and validated | 6.0, 6.1 |
| C5 | High | Applied: OI-D26, simulation sees the whole stream, a tab recording only what the tab consumed; OI-7 restated; REC row | 6.1, OI-7, OI-D26, 9.5 |
| C6 | High | Applied, in a different form: the speech role is per (site, container); the desktop role is proposed as a fifth shared exception for the owner (OI-D25), with the per-(site, container) alternative designed | 6.0, 6.15, OI-D25, 9.5 |
| C7 | High | Applied: TX-16 amendment with a closed list of OS display surfaces; G233 restated | G233, 9.5 |
| C8 | High | Applied: the broker's parsing exception is recorded with its four closed bodies; `DBusActivatable` waits for Stage 2 with a specified activation file (OI-D27) | 6.0, 6.2, 9.5 |
| C9 | High | Applied: an inbound allowlist; internal pages never opened from outside; a remote-path prompt; drag-in and `Application.Open` covered; OI-4's corpus | 6.2, G237, OI-4, 9.5 |
| C10 | High | Applied: bounded queueing with `busy-retry`; one message per batch; OI-5 and Appendix C consistent | 6.2, OI-5 |
| C11 | High | Applied: a per-OS install and launch layout (the broker is the macOS bundle executable, with versioned helper bundles; cold launches through Launch Services), handed to FI-2 as a requirement | 6.2, OI-D3, 9.5 |
| C12 | High | Applied, in a different form: the long-lived subscriptions are the broker gate's, so the desktop role stays on demand and FP-1 is unchanged; the first frame's theme has a ≤ 50 ms† bounded read and a cached fallback | 6.1, 6.9, OI-12, OI-19 |
| C13 | High | Applied: Q-OI-1 moves to SH-M0 with the third option and pre-recorded consequences | 6.0, 9.2, Q-OI-1 |
| C14 | Medium | Applied: server side, the gate, a re-estimate of 2–3† and a core at AX-M0; reversal at AX-M0; "PipeWire" corrected to RTKit and portals | 6.9, 9.2, 9.3, OI-D6 |
| C15 | Medium | Applied: interface descriptions hand-written from prose or black-box introspection, never repository XML | 6.9, §7 |
| C16 | Medium | Applied: a local-stream substrate primitive at M5 | 6.9, 6.18, App. C |
| C17 | Medium | Applied: the launcher's algorithm (cold-start race, lock, started reply, recovery window), the downgrade retry, and the skew matrix restated | 6.2, OI-2 |
| C18 | Medium | Applied: the launcher verifies the server; foreground rights only after `accepted`; the session in the pipe name; a Medium mandatory label; OI-14 probes | 6.2, OI-14 |
| C19 | Medium | Applied: per-window AUMIDs and relaunch properties pointing at the launcher; OI-11's update case | 6.7, OI-11 |
| C20 | Medium | Applied: the journal is a plain file shared with the uninstaller; the installer writes into it | 6.6, 6.18 |
| C21 | Medium | Applied: the broker's mark first, the helper's scan, then a check; SH-R6 row; OI-25 | 6.5, OI-25, 9.5 |
| C22 | Medium | Applied: deadline inputs with reply tokens; OI-23 covers session end | 6.1, OI-23 |
| C23 | Medium | Applied: timeouts and fallbacks decided in `hg-os-broker` on the substrate clock; the service is a thin codec | 6.1, 6.9 |
| C24 | Medium | Applied, in a different form: the shell-document frame stays, with a text-free broker fallback frame for outages and for the native windows; the all-broker frame is considered and rejected (TX-15); AX and GR rows; OI-26; the prompt-frame sentence removed | 6.9, OI-D20, OI-26, 9.5 |
| C25 | Medium | Applied: media-session arbitration; the multi-window case in OI-18 | 6.10, OI-18 |
| C26 | Medium | Applied: one size budget (≤ 64,512† bytes, URLs ≤ 8 KiB†, ≤ 6† operands, batches split) that fits the generator's 64 KiB cap | 6.2 |
| C27 | Medium | Applied: rows for the coding standard, TX-16, DET-1, I-1's exceptions, REC-R6, SH-R6, SEC 6.5, GR, AX, storage, PH, ED, and the process-kind counts | 9.5 |
| C28 | Medium | Applied: per-OS rows re-cut; no `zbus` saving booked; 9–11.5† peak, 7.5–9.5† net in Stage 1 | 9.3 |
| C29 | Medium | Applied: M1 gets the trait, types, the scripted model skeleton and the IZLH golden vectors only; the rest moves to OI-M1 | 9.2, App. C |
| C30 | Medium | Applied: OI-10 and OI-20 as a one-sided test with ≥ 10,000† trials and a positive control's power | OI-10, OI-20 |
| C31 | Medium | Applied: provisional bands for the helper (≤ 32 MiB†) and the speech role (≤ 24 MiB†), measured at OI-M2; OI-22 split warm and cold | OI-12, OI-22 |
| C32 | Medium | Applied: Linux writes the file the lookup uses; macOS re-reads http, https and `public.html` | 6.4, OI-8 |
| C33 | Medium | Applied: the Linux audio residual and mute as pause or cancel, recorded for FI-29 | 6.15 |
| C34 | Medium | Applied: an Android mapping and Custom Tabs as a Stage 2 security input | 6.19 |
| C35 | Low | Applied: a person-approved web handler creates an `OsRegistration` | 6.4 |
| C36 | Low | Applied: `ms-appinstaller`, the Office document schemes, and `file:`-origin documents as web content | 6.5 |
| C37 | Low | Applied: "Considered, not designed here" | 6.19 |
| C38 | Low | Applied: residuals stated; a lock-screen media setting | 6.10, §11 |
| C39 | Low | Applied: OI-16 a ship blocker; OI-17's Wayland criterion; a `tel:` case in OI-10 | OI-10, OI-16, OI-17 |
| C40 | Low | Applied: `listen` returns a guard; the four types defined; one broker dispatcher | 6.1 |

### A.2 Independent fact-check (`factcheck.md`, 92 claims, 64 links)

| Item | Disposition |
|---|---|
| Must-fix 1: OI-12's launcher lifetime against OI-1 | Applied: the launcher's lifetime is bounded by OI-1's 100 ms† p95 |
| Must-fix 2: the macOS 26 prompt misattributed to login items | Applied: moved to the Stage 2 headless broker and Q-OI-7 (6.13) |
| Must-fix 3: a GitHub pull request cited in the Windows note (WP 4.5) | Applied: the row is removed in the note; Q-OI-8 cites WP 4.3–4.4; §7 records it |
| Must-fix 4: the UCPD key list attributed to Microsoft | Applied: attributed to the vendor note and HP 1.14 (2.1, §3, §11) |
| Must-fix 5: Stage 1's OS-adapter row is not "unchanged" | Applied: §1 records the conflict and 9.5 amends the row to NW 6.1's role and count |
| Recommended 1–2: macOS 26.4 flag; EEA list confirmed | Applied: flag kept with the release-notes check; the EEA caveat dropped |
| Recommended 3–5: UCPD dates; intro wording; scheme-flooding statuses | Applied |
| Recommended 6–7: KDE and GNOME primary sources; Chrome 77 | Applied: KDE's announcement and GNOME's release notes cited; Chrome 77 marked Medium |
| Recommended 8: Linux sleep lock | Applied: the broker's gate holds the delay lock (6.10) |
| Recommended 9: † on trial counts | Applied: spec-chosen trial counts marked † |
| Recommended 10–11: SSIP link; the OD-7 conflict | Applied: the SSIP URL is replaced by the note's row with Q-OI-13; §7 records the conflict and that no decision rests on that page |
| Recommended 12: small fixes | Applied: extensions 6.1; "should" for `QueryEnd`; "the OS prompts once"; MD's D-Bus client row; the deep link's floor and escaping; LX 3.3 corrected in the note; the unused link removed; the 2023 row moved; compositors lacking xdg-foreign named in the note |
| Recommended 13: platform floors | Applied as Q-OI-14, pointing at FI-2 and GR-D21 |

## Appendix B. Row closure

| Row | Feature | Section | Target |
|---|---|---|---|
| WA-12.6 | Web Share | 6.8 | OI-20 |
| BF-10.9 | Share page | 6.8 | OI-20 |
| BF-12.7 | Startup boost and background running | 6.13 (preloading refused; background Stage 2) | OI-12, OI-13 |
| BF-16.14 | Phone-number click-to-call | 6.5 item 6 | OI-10 |
| BF-17.6 | OS share sheet integration | 6.8 | OI-20 |
| PO-8.1 | Default-browser registration on Windows | 6.4 | OI-8, OI-9 |
| PO-8.2 | Default browser on macOS | 6.4 | OI-8, OI-9 |
| PO-8.3 | Default browser on Linux | 6.4 | OI-8, OI-9 |
| PO-8.6 | Protocol and file-type associations | 6.3, 6.4, 6.5 | OI-4, OI-10 |
| PO-8.7 | Taskbar and dock integration | 6.7 | OI-11, OI-15 |
| PO-8.9 | Antivirus scan on download | 6.5 item 5 | OI-25 |
| PO-8.11 | Background mode and start at login | 6.13 | OI-12 |
| PO-8.12 | OS scripting and automation hooks | 6.14 | OI-27 |
| PO-8.13 | Desktop wallpaper and OS actions | 6.14 | — (refusal recorded) |
| PO-9.8 | System media controls | 6.10 | OI-18 |
| PO-9.9 | OS share sheet | 6.8 | OI-20 |
| PO-10.10 | HiDPI and per-monitor scaling | 6.12 | OI-16 |
| PO-10.16 | Window management integration | 6.9, 6.12 | OI-17, OI-26 |
| PO-10.17 | Touch, pen and precision touchpad gestures | 6.12 | — (input routing's targets) |
| PO-10.18 | Multi-monitor window placement | 6.12 | OI-17 |
| PO-13.9 | Command-line switches | 6.2, 6.3 | OI-4 |
| PO-13.10 | Single instance and remote control | 6.2 | OI-1, OI-2, OI-5, OI-6 |

## Appendix C. M1-exit inputs, stated for adoption

Audit §8.2 asked that the pattern be decided before M1 exits, and M1's phases A to D2 have merged, so M1 takes only what fixes the shapes; everything that needs the broker's launch logic or a production recording format waits for OI-M1 at M5 (critique C29).

**1. The recorded OS-input pattern (OI-D1, OI-D26).**
- **Trait, in `hg-substrate`:** `OsPort` as in 6.1: `command(&self, frame: Vec<u8>) -> Result<OsCallId, OsError>` and `listen(&self, seq: SequenceHandle, sink: OsSink) -> Result<OsListening, OsError>`, with `OsCallId` (u64, monotonic per broker incarnation), `OsSink` (a boxed sink of `Frame(Vec<u8>)` or `Closed`, like `Ipc`'s `Sink`), `OsError` (closed: `Unsupported`, `TooLarge`, `Closed`, `Refused`) and `OsListening` (a guard; dropping it stops delivery). `Substrate` gains `os: Option<Arc<dyn OsPort>>`, `Some` only in the broker. Frames are bytes, as `Ipc`'s are, so `hg-substrate` depends on nothing above L1.
- **Types, in `os.hgidl` generated into `hg-os-types` (L2):** `OsInput` with the arms `Launch`, `DefaultStatus`, `Settings`, `Locale`, `TimeZone`, `Hardware`, `Displays` and `Power` (with the deadline-input reply token) at M1, and `Media`, `Notification` and `Answer` reserved; `OsCommand` with `Query`, `Ask`, `Register`, `Hold`, `Post`, `Launch` and `Reply`. Every arm fits the generator's 64 KiB cap and gets its Kani harness.
- **Enterprise policy is not an `OsPort` input** (owner decision, 25 October 2026): it arrives from EP-D7's sandboxed policy adapter as typed `.hgidl` snapshots with a 32-byte `PolicyDigest`, recorded at that IPC boundary.
- **Simulator, in `hg-substrate-sim`:** the **scripted** `OsPort` skeleton (a seeded OS model that answers commands and emits inputs); the **recorded** `OsPort` follows at OI-M1, with the production recording scope of OI-D26.
- **M1 exit test:** a DST run in which the scripted model delivers a 100†-launch burst of `Launch` frames, a `Settings` change and a `Power` will-sleep deadline input to a test consumer; the trace hash is identical on three OSes and two runs. The full 1,000† storm through the real launch policy is OI-M1's (OI-5).

**2. The launch hand-off message (OI-D2).**
- **Crate:** `hg-launch-wire`, L2s, tier P, no dependencies outside `core` and `alloc`; encoder and decoder for 6.2's header, message types (`request`, `reply`, `commit`, `started`), fields and reply codes.
- **Bounds:** total ≤ 64,512† bytes; URLs and paths ≤ 8,192† bytes; ≤ 6† operands per request (larger batches split across requests on one connection); ≤ 32† fields.
- **Proofs:** Kani harnesses for the header, every field decoder and the length arithmetic; a golden-vector file that every future version must still decode. The fuzz fleet joins at OI-M1.
- **Rule for the substrate and IPC:** `process.hgidl`'s `BuildId` check is unchanged; the hand-off is not a process bootstrap and never carries an endpoint or a handle.
- **M1 exit test:** the golden vectors decode; a v1.1 request with an unknown non-critical field is accepted by a v1.0 decoder and one with an unknown critical field is refused with `unsupported-critical-field`.

**3. Proposed for M5, not M1:** the local-stream substrate primitive (a pre-opened local stream with readiness delivered as tasks and handle passing, with a simulated implementation), which `hg-dbus`, the hand-off listener and the SSIP client need.

[MS-DP]: https://learn.microsoft.com/en-us/windows/win32/shell/default-programs
[MS-DEF]: https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/default-apps-platform
[MS-SET]: https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-default-apps-settings
[MS-URI]: https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/aa767914(v=vs.85)
[MS-AUMID]: https://learn.microsoft.com/en-us/windows/win32/shell/appids
[MS-PIN]: https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/pin-to-taskbar
[MS-SHARE]: https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/integrate-sharesheet-send
[MS-PRIN]: https://blogs.windows.com/windowsexperience/2023/03/17/a-principled-approach-to-app-pinning-and-app-defaults-in-windows/
[MS-DMA25]: https://blogs.windows.com/windows-insider/2025/06/02/updates-to-windows-for-the-digital-markets-act/
[AP-SETDEF]: https://developer.apple.com/documentation/appkit/nsworkspace/setdefaultapplication(at:toopenurlswithscheme:completion:)
[AP-URLFOR]: https://developer.apple.com/documentation/appkit/nsworkspace/urlforapplication(toopen:)-7qkzf
[SOX-264]: https://scriptingosx.com/2026/03/macos-26-4-brings-more-default-app-confirmation-prompts/
[XDG-DE]: https://specifications.freedesktop.org/desktop-entry/latest/
[XDG-EXEC]: https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
[XDG-DBUS]: https://specifications.freedesktop.org/desktop-entry/latest/dbus.html
[XDG-MIME]: https://specifications.freedesktop.org/mime-apps/latest/file.html
[XDG-NOTIFY]: https://specifications.freedesktop.org/notification/latest/
[MPRIS]: https://specifications.freedesktop.org/mpris/latest/
[XDP]: https://flatpak.github.io/xdg-desktop-portal/docs/
[XDP-WHY]: https://flatpak.github.io/xdg-desktop-portal/docs/reasons-to-use-portals.html
[XDP-REG]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html
[XDP-URI]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.OpenURI.html
[XDP-NOT]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Notification.html
[XDP-INH]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Inhibit.html
[XDP-SET]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html
[XDP-DL]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.DynamicLauncher.html
[XDP-SEC]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Secret.html
[DBUS]: https://dbus.freedesktop.org/doc/dbus-specification.html
[WS]: https://www.w3.org/TR/web-share/
[PS-URL]: https://positive.security/blog/url-open-rce
[FP-SF]: https://fingerprint.com/blog/external-protocol-flooding/
