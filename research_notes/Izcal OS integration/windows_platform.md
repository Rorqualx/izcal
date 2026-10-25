# Izcal OS integration: Windows platform today (2026)

Compiled 24 October 2026. Primary sources were preferred: Microsoft Learn conceptual and API reference pages, the Microsoft Lifecycle site, and Microsoft blogs. Vendor support notes and press were used only where Microsoft publishes nothing.

M-1 note: this note uses only published prose and vendor documentation. No third-party project's source code, headers, commits, diffs or code reviews were opened. Search results that pointed at browser source trees (a Firefox source browser, a Chromium source file on taskbar pinning) were not opened or used. A Firefox prose page on setting defaults, hosted next to code, was also left out under OD-7, because an outside source covers the same facts. A binary-analysis write-up of a Windows driver (UCPD.sys) was not used, because it reads like code analysis of an OS component. A Microsoft API reference describes a contract. It is not a template copied from an implementation.

Confidence: **High** = the primary text was read and the claim checked against it. **Medium** = an official page's summary or search snippet, a secondary source, or an established vendor behaviour not re-read this session. **Low** = secondary source only. **Gap** = no published evidence found. **Inference** = my conclusion from the cited rows.

**Method notes.**
- Scope: what an unpackaged Win32 browser can and cannot do with the Windows shell and OS services in 2026; which features need package identity; which need medium integrity, and so cannot run in an AppContainer renderer or utility process.
- Budget: about 12 web searches went to the top questions (EEA default-browser changes, UserChoice protection, notifications for unpackaged apps, share, taskbar pinning, Windows 10 support, speech). Everything else came from direct reads of known Microsoft Learn URLs. Rows not read in full this session are graded Medium or Low.
- Dates: a page's `ms.date`/`updated_at` where shown; "read 2026-10-24" means read today with no visible date.
- "Package identity" means MSIX or *packaging with external location* (formerly "sparse package"): a signed identity-only MSIX registered against an existing install directory.

---

## 0. Cross-cutting: package identity and integrity level

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 0.1 | **Package identity unlocks a named list of features unavailable to unpackaged apps**: "toast and push notifications, background tasks, app extensions, share targets, file associations, startup tasks, privacy consent prompts, and Windows AI Foundry APIs" | 8 feature families | [Microsoft: Grant package identity by packaging with external location manually](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps) | 2026-10-03 | High |
| 0.2 | **Identity without repackaging**: a signed identity MSIX with `uap10:AllowExternalContent=true`, `runFullTrust` + `unvirtualizedResources` capabilities, `TrustLevel="mediumIL"`, `RuntimeBehavior="win32App"`, `AppListEntry="none"`. The installer registers it with `Add-AppxPackage -ExternalLocation` (per user) or stage + provision (per machine). Each exe's side-by-side manifest carries a matching `<msix publisher packageName applicationId>` element | MinVersion 10.0.19041.0 (Windows 10 2004); MaxVersionTested 10.0.26100.0 | Same as 0.1 | 2026-10-03 | High |
| 0.3 | **Signing is mandatory**: the identity package must be signed with a certificate trusted on the target machine; a self-signed certificate must be in Trusted People. The same version cannot be registered twice. A mismatched ExternalLocation leaves the app without identity at runtime and raises no error | 5 documented failure codes | Same as 0.1 | 2026-10-03 | High |
| 0.4 | **"File associations" appear in the identity list (0.1), but classic registry associations need no identity.** The list refers to manifest-declared `uap:FileTypeAssociation`/`uap:Protocol`. Registry ProgIDs + Capabilities (section 1) remain the unpackaged route | — | 0.1; [Microsoft: Default Programs](https://learn.microsoft.com/en-us/windows/win32/shell/default-programs) | 2025-04-15 | Inference |
| 0.5 | **AppContainer processes always run at low integrity**; AppContainer is a second axis, orthogonal to IL. Unpackaged apps can use AppContainer by calling `CreateAppContainerProfile` themselves. The container isolates "hardware, files, registry, other apps, network connectivity, and network resources without specific permission" | IL = low, always | [Microsoft: AppContainer for legacy apps](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-) | 2025-04-15 | High |
| 0.6 | **Elevated processes cannot show app notifications**: "`Show` will fail silently" when the app runs as administrator | — | [Microsoft: Quickstart: app notifications](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-quickstart) | 2026-09-10 | High |

**Implications:** Izcal should plan for two Windows install modes: unpackaged (registry only) and unpackaged + identity package (an extra signed MSIX). Identity is the only route to share-target registration and the supported route to background-activated notifications. The broker runs at medium IL with the user's ordinary token and owns every shell integration. Renderers and AppContainer utilities own none.

---

## 1. Default browser

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Registration = application-specific ProgIDs + a `Capabilities` key + a `RegisteredApplications` value.** Capabilities holds `ApplicationDescription` (required, or the app is not listed), `ApplicationName`, `FileAssociations` (.htm/.html/.shtml/.xht/.xhtml → ProgID), `UrlAssociations` (http/https/ftp → per-scheme ProgID), `MIMEAssociations`, `Startmenu\StartMenuInternet`. `RegisteredApplications\<Name>` points at the Capabilities path | 1 ProgID per scheme recommended | [Microsoft: Default Programs](https://learn.microsoft.com/en-us/windows/win32/shell/default-programs) | 2025-04-15 (page notes it "does not apply for Windows 10" for the defaults *UI*) | High |
| 1.2 | **StartMenuInternet is vestigial**: "the registry data associated with the **Internet** entry is not used by Windows at all" since Windows 7. It is still the conventional client registration | — | Same as 1.1 | 2025-04-15 | High |
| 1.3 | **Install-time prompt**: after writing its keys, the installer calls `SHChangeNotify(SHCNE_ASSOCCHANGED, …)`. Windows may then show a one-time system notification offering the new http+https handler as default, if the user has not already seen it advertised | 1 notification per app | Same as 1.1 | 2025-04-15 | High |
| 1.4 | **The programmatic set/check APIs are gone**: `SetAppAsDefaultAll` and the restore APIs were deprecated in Windows 8; `QueryAppIsDefault(All)` is "no longer supported as of Windows 8". Microsoft's rule: "An application should never reclaim a default without asking the user" | Deprecated since Windows 8 (2012) | Same as 1.1 | 2025-04-15 | High |
| 1.5 | **Reading the current default still works**: `IApplicationAssociationRegistration::QueryCurrentDefault(L"https", AT_URLPROTOCOL, AL_EFFECTIVE, …)` returns the ProgID that `ShellExecute` would use. For machine-default protocols it can return a legacy command line instead of a ProgID | Windows Vista+ | [Microsoft: QueryCurrentDefault](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iapplicationassociationregistration-querycurrentdefault) | 2024-02-22 | High |
| 1.6 | **`AssocQueryString` (ASSOCSTR_PROGID/EXECUTABLE) is the other documented read path** | — | Not re-read this session | — | Medium |
| 1.7 | **Deep link to the app's own Default Apps page**: `ms-settings:defaultapps?registeredAppUser=<name>` (HKCU RegisteredApplications), `?registeredAppMachine=<name>` (HKLM), or `?registeredAUMID=<aumid>` (packaged). Requires Windows 11 21H2/22H2 with the 2023-04 cumulative update, or 23H2+. The value must be URI-escaped | 3 parameters | [Microsoft: Launch the Default Apps settings page](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-default-apps-settings) | 2025-01-11 (updated 2026-05-29) | High |
| 1.8 | **Why apps cannot just write UserChoice**: per-user choices live under each extension's or scheme's `UserChoice` key with a `Hash` value; a wrong hash makes Windows fall back to its default. Since February 2024 (KB5034763 for Windows 10, KB5034765 for Windows 11) a filter driver, the User Choice Protection Driver `UCPD.sys`, blocks direct writes to the http, https and .pdf associations | 3 protected associations; Feb 2024 | [Citrix: Unable to save default application … after February MS updates](https://support.citrix.com/external/article/691100/unable-to-save-default-application-on-up.html); [gHacks: UCPD](https://www.ghacks.net/?p=205343) | 2024 | Medium (no Microsoft doc found) |
| 1.9 | **EEA (DMA) changes, 2025**: in the EEA, a browser's "Set default" button in Settings > Apps > Default apps now also sets ftp, http, https and read, plus .htm, .html, .mht, .mhtml, .shtml, .svg, .xht, .xhtml, .xml and optionally .pdf. It pins the browser to the taskbar (and Start on Windows 11) unless the user opts out. Edge stopped prompting when the default changes. Insider Beta build 26120.4151 (May 2025); retail rollout from July 2025 | 4 schemes + 9–10 file types | [Windows Insider blog (via elevenforum/TechRepublic summaries)](https://www.techrepublic.com/article/news-windows-default-browser-dma-update-2025/); [The Register](https://www.theregister.com/2025/06/03/microsoft_stops_eu_edge_pestering/); [Microsoft EU policy blog](https://blogs.microsoft.com/eupolicy/2023/11/22/windows-updates-comply-dma-edge/) | 2025-05/07; 2023-11-22 | Medium (primary Insider post not fetched; URL guessed wrong) |
| 1.10 | **Outside the EEA**, Windows 11 has had a one-click "Set default" for a browser's http/https/.htm/.html since KB5011563 (build 22000.593, March 2022). Other types (.pdf, .svg, .webp, .shtml) and shell-internal links (Start search, Widgets) were not covered at that time | 1 button | [PCWorld](https://www.pcworld.com/article/627401/new-windows-11-update-restores-one-click-browser-choice-mostly.html); [gHacks](https://www.ghacks.net/?p=177742) | 2022 | Low (press) |
| 1.11 | **"UserChoiceLatest"** and any 2025–2026 change to the hash scheme: **no published source found** | — | Search 2026-10-24 | — | Gap |
| 1.12 | **A Microsoft "principles for default apps" post (2024/2025)**: not found in this budget | — | — | — | Gap |
| 1.13 | **Enterprise**: admins deploy a *default associations configuration XML* via DISM `/Export-DefaultAppAssociations` / `/Import-DefaultAppAssociations` and the Group Policy "Set a default associations configuration file". This is named only, not researched | — | — | — | Medium |

**Implications:** Izcal registers ProgIDs + Capabilities + RegisteredApplications (HKCU for per-user installs, HKLM for per-machine), calls `SHChangeNotify`, reads its own status with `QueryCurrentDefault`, and sends the user to `ms-settings:defaultapps?registeredAppUser=…` (or `registeredAppMachine`). It never writes UserChoice. Register .svg, .xml, .mht(ml), .pdf and `read:` under Capabilities so the EEA one-click button covers them. Black-box test to schedule: which associations the EEA button actually sets for a third-party browser, and whether `ms-settings:defaultapps` behaves the same in and outside the EEA.

---

## 2. Single instance, activation and argument safety

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Classic Win32 single-instancing uses a named mutex**: "Typically this is done using a single named mutex to indicate if an app is already running." Unpackaged apps are multi-instance by default | — | [Microsoft: App instancing with the app lifecycle API](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/applifecycle/applifecycle-instancing) | 2026-09-10 | High |
| 2.2 | **Windows App SDK `AppInstance` works for unpackaged apps** (after `MddBootstrapInitialize`): `FindOrRegisterForKey(key)` either claims a key or returns the instance that holds it; `RedirectActivationToAsync(args)` forwards Launch/File/Protocol activation arguments to the `Activated` event. Redirection is *not* terminal. Lists are per user and per app version. Waiting on it from an STA thread blocks, so run it on another thread | 1 key per instance | Same as 2.1 | 2026-09-10 | High |
| 2.3 | **Hand-off by hand (no Windows App SDK)**: the second process finds the first through a per-user, per-profile named object and passes its argv over a named pipe or `WM_COPYDATA`. This is standard Win32 practice, not re-read this session | — | — | — | Medium |
| 2.4 | **URL/file opens arrive on the command line**: the registered `shell\open\command` string with `%1` is expanded by `ShellExecute`; `DDEEXEC` is the legacy alternative | — | [Microsoft: Registering an Application to a URI Scheme](https://learn.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/aa767914(v=vs.85)) | 2016-07-13 (archived) | High |
| 2.5 | **Argument-injection risk is documented**: "any non-encoded spaces, quotes, and backslashes in the URI will be interpreted as part of the command line … Malicious parties could use additional quote or backslash characters to pass additional command line parameters." Quoting `"%1"` mitigates this "but doesn't completely solve the problem of quotes in the URI or a backslash at the end". Some callers percent-decode before launch and others do not. Handlers "should assume that any parameters on the command line could come from malicious parties" and confirm dangerous actions with the user | Example: `alert:"Hello%20World"` → 2 argv entries | Same as 2.4 | 2016-07-13 | High |
| 2.6 | **A `--` (end-of-options) separator before `%1`** is a convention, not a Windows rule. Microsoft's URI-scheme page does not mention it | — | Same as 2.4 (absence) | — | Inference |
| 2.7 | **Foreground rules**: `SetForegroundWindow` succeeds only if the caller is a desktop app, no `LockSetForegroundWindow` is in force, no menu is active, **and** one of these holds: the lock timeout has expired, the caller is the foreground process or was started by it, there is no foreground window, the caller received the last input event, or one side is being debugged. Otherwise the taskbar button flashes. The foreground process can pass the right on with `AllowSetForegroundWindow(pid)`; that grant lapses at the next user input not directed at that process | 6 qualifying conditions | [Microsoft: SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) | 2025-10-06 | High |
| 2.8 | **Restart after update or crash**: `RegisterApplicationRestart(cmdline, flags)` (flags `RESTART_NO_CRASH/HANG/PATCH/REBOOT`). Last chance to call is during `WM_QUERYENDSESSION`. Restart after a crash or hang asks the user; restart for an update is automatic. A 60-second minimum uptime prevents restart loops. Restart across a reboot requires the installer to use `EWX_RESTARTAPPS`/`SHUTDOWN_RESTARTAPPS`. The command line is limited to `RESTART_MAX_CMD_LINE` | 60 s; 4 flags | [Microsoft: RegisterApplicationRestart](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-registerapplicationrestart) | 2025-07-01 | High |

**Implications:** the broker owns the single-instance lock (one per profile directory) and the IPC endpoint that receives forwarded arguments. A second launch must hand over its argv *before* it starts any services. Its first action should be `AllowSetForegroundWindow(ASFW_ANY or the broker's PID)`, while it still holds the user's input grant. Izcal parses its command line itself and treats any argument after the protocol/file slot as untrusted. Register `"<exe>" --single-argument "%1"` (or an equivalent Izcal-defined marker), and accept exactly one URL or path after it, so that injected switches are inert. Black-box test: the exact argv Windows produces for crafted `izcal:`/`https:` URIs containing quotes, a trailing backslash and `%22`.

---

## 3. AppUserModelID, taskbar and jump lists

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **AUMID form**: `CompanyName.ProductName.SubProduct.VersionInformation`, ≤128 characters, no spaces, Pascal-cased; omit VersionInformation if upgrades should keep the identity | 128 chars | [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids) | 2025-04-15 | High |
| 3.2 | **Where it must be applied**: the shortcut (`System.AppUserModel.ID`), the process (`SetCurrentProcessExplicitAppUserModelID`, before any UI, jump-list call or `SHAddToRecentDocs`) or each window (`SHGetPropertyStoreForWindow`; the window value overrides the process value), the ProgID registration, and every jump-list and `SHAddToRecentDocs` call. Jump-list calls made from another process (such as the installer) without an explicit AUMID "will have no effect" | 5 places | Same as 3.1 | 2025-04-15 | High |
| 3.3 | **Per-profile taskbar identity is supported by design**: "A single executable … with multiple modes that appear to the user as separate applications should assign different AppUserModelIDs to each mode". Window-level AUMIDs can carry `RelaunchCommand`, `RelaunchDisplayNameResource` and `RelaunchIconResource`; `PreventPinning` must be set before the ID | — | Same as 3.1 | 2025-04-15 | High |
| 3.4 | **Per-window AUMIDs break the file dialog's automatic recent-docs**: the app must call `SHAddToRecentDocs` itself and set `FOS_DONTADDTORECENT` | — | Same as 3.1 | 2025-04-15 | High |
| 3.5 | **Shortcut names containing "Install", "Setup", "Help", "Readme" and similar, and hosts such as rundll32/dllhost, are excluded from pinning and the MFU list unless an explicit AUMID is set** | 11 strings; 22 exes | Same as 3.1 | 2025-04-15 | High |
| 3.6 | **Taskbar pinning API is open to Win32 apps from early 2026**: `TaskbarManager.RequestPinCurrentAppAsync` shows a user-consent dialog. It must run on a foreground UI thread, with the app in the foreground and holding a Start menu entry. A Win32 app checks for `ITaskbarManagerDesktopAppSupportStatics` first. It was a Limited Access Feature (unlock token by request); "The limited access restriction is being removed from Windows starting with KB5074105 (OS Builds 26200.7705 and 26100.7705)" (29 January 2026 preview). Microsoft says installers must not call it, and apps should not ask repeatedly | 1 prompt; KB5074105 | [Microsoft: Pin your app to the taskbar](https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/pin-to-taskbar) | 2026-07-16 | High |
| 3.7 | **Whether `TaskbarManager` needs package identity for a Win32 app is not stated on the page** | — | Same as 3.6 (absence) | — | Gap |
| 3.8 | **Jump lists (`ICustomDestinationList`: tasks, custom categories, known Recent/Frequent), `ITaskbarList3` (progress, overlay icon, thumbnail toolbar of ≤7 buttons) work for unpackaged apps that have an AUMID.** Badge notifications are a packaged/identity feature | ≤7 thumb buttons | Not re-read this session | — | Medium |

**Implications:** give each Izcal profile its own AUMID (`Izcal.Browser.Profile<id>`), set on every window. Create a per-profile Start-menu shortcut carrying the same AUMID, which notifications also need (section 4). Download progress uses `ITaskbarList3::SetProgressValue` from the broker. Offer "Pin to taskbar" only after explicit user action, on builds that have KB5074105.

---

## 4. Notifications

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **Windows App SDK `AppNotificationManager` supports unpackaged apps**: for them, `Register()` "automatically sets up the COM server registration that allows Windows to launch your app when a notification is clicked", and takes the display name and icon from the shell. Register the `NotificationInvoked` handler before `Register()`, or a new process is launched. Call `Unregister()` on exit and `UnregisterAll()` on uninstall | — | [Microsoft: app notifications console guide](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-console); [AppNotificationManager.Register](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.appnotifications.appnotificationmanager.register) | read 2026-10-24 | Medium (snippets) |
| 4.2 | **Packaged apps declare `windows.toastNotificationActivation` with a `ToastActivatorCLSID`, plus a `windows.comServer` ExeServer with `Arguments="----AppNotificationActivated:"`.** If the app was not running, it is COM-launched with activation kind `Launch`; the payload then arrives through `NotificationInvoked`. `activationType="background"` "is ignored for desktop apps" | 1 CLSID | [Microsoft: Quickstart: app notifications](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-quickstart) | 2026-09-10 | High |
| 4.3 | **Without the Windows App SDK, a classic unpackaged route exists**: an AUMID, a Start-menu shortcut carrying the AUMID and a `ToastActivatorCLSID`, and a registered `LocalServer32` COM activator (`INotificationActivationCallback`) ([Microsoft: send local toast from C++ WRL](https://learn.microsoft.com/en-us/windows/apps/design/shell/tiles-and-notifications/send-local-toast-desktop-cpp-wrl)). Not re-read this session | 1 shortcut + 1 CLSID | Same | — | Medium |
| 4.4 | **Not available when elevated** (row 0.6). Focus/Do-Not-Disturb suppresses banners into Notification Center. The documented visible-action limit is ≤5 buttons. Neither was re-read this session | ≤5 actions | — | — | Medium |
| 4.5 | *Removed in phase 8 (24 October 2026).* This row cited a GitHub pull request, which is a code review and is excluded under M-1 part 3. Its claim (that `AppNotificationManager.Register()` fails in one unpackaged configuration) is not used; Q-OI-8 measures unpackaged activation black-box instead | — | — | — | — |

**Implications:** web notifications are a broker service. Izcal can use its own COM activator (no Windows App SDK dependency, which fits the "build our own" policy), registered per user, with the profile's AUMID on its Start shortcut. Click-through on a cold start lands in the broker's activation path (section 2), not in a renderer. Black-box test: whether an unpackaged activator survives Focus mode and whether actions work on a cold start, on 24H2/25H2.

---

## 5. System media controls

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **`ISystemMediaTransportControlsInterop::GetForWindow(hwnd)`** returns a `SystemMediaTransportControls` for a desktop window; it is implemented by the SMTC activation factory. The page's legacy banner points at the old "Media Transport Controls"; SMTC itself is the current API | 1 method | [Microsoft: ISystemMediaTransportControlsInterop](https://learn.microsoft.com/en-us/windows/win32/api/systemmediatransportcontrolsinterop/nn-systemmediatransportcontrolsinterop-isystemmediatransportcontrolsinterop) | 2024-02-22 | High |
| 5.2 | **SMTC carries metadata (title, artist, artwork via `DisplayUpdater`), button enablement, playback status and a timeline (`UpdateTimelineProperties`)**. Hardware media keys are delivered to the active SMTC session; apps without SMTC get `WM_APPCOMMAND` (APPCOMMAND_MEDIA_PLAY_PAUSE …) only when focused | — | Not re-read this session | — | Medium |
| 5.3 | **No package identity requirement is documented for SMTC via the interop** | — | Absence in 5.1 | — | Inference |

**Implications:** the audio service reports sessions; the broker owns one SMTC object per top-level window (the HWND lives in the broker) and maps the Media Session API to it.

---

## 6. Share

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Desktop apps, "packaged or unpackaged", can be a share *source***: through `IDataTransferManagerInterop::GetForWindow(hwnd)`, `DataRequested`, and `ShowShareUIForWindow(hwnd)`. `GetForCurrentView`/`ShowShareUI` are UWP-only | — | [Microsoft: Share content from your app](https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/integrate-sharesheet-send) | 2026-10-19 | High |
| 6.2 | **Payloads**: `SetText`, `SetWebLink` (recommended for URLs, for rich previews), `SetApplicationLink`, `SetStorageItems` (files), plus Title (required), Description and Thumbnail. Source events: `TargetApplicationChosen`, `ShareCompleted`, `ShareCanceled` | 4 payload kinds | Same as 6.1 | 2026-10-19 | High |
| 6.3 | **The interop's API reference still says "Windows 8 [UWP apps only]"** and contradicts the 2026 guide | — | [Microsoft: IDataTransferManagerInterop](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idatatransfermanagerinterop) | 2024-02-22 | High (conflict noted) |
| 6.4 | **Being a share *target* requires package identity**: "for the default share target registration to work with Win32 apps, the app needs to have a package identity"; unpackaged apps get identity through packaging with external location | — | [Microsoft: integrate sharesheet (packaged)](https://learn.microsoft.com/windows/apps/develop/windows-integration/integrate-sharesheet-packaged); row 0.1 | read 2026-10-24 | Medium (snippet) + High (0.1) |
| 6.5 | **PWAs can be share targets through the manifest `share_target` member when installed** | — | Same as 6.1 | 2026-10-19 | High |

**Implications:** `navigator.share()` maps to the broker calling `ShowShareUIForWindow` on the tab's top-level HWND, with `SetWebLink` for URLs. Receiving shares (Izcal as a target, or installed PWAs as targets) depends on the identity package.

---

## 7. Power, wake and efficiency

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Power requests**: `PowerCreateRequest(REASON_CONTEXT)` + `PowerSetRequest` with `DisplayRequired` (must be paired with `SystemRequired` to stop sleep), `SystemRequired`, `AwayModeRequired` (S3 only) or `ExecutionRequired`. **On Modern Standby on battery, system and execution requests end 5 minutes after the sleep timeout.** All end on user-initiated sleep (power button, lid). Provide a localized reason string | 4 types; 5 min | [Microsoft: PowerSetRequest](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powersetrequest) | 2025-07-01 | High |
| 7.2 | **EcoQoS**: `SetProcessInformation(ProcessPowerThrottling)` with `PROCESS_POWER_THROTTLING_EXECUTION_SPEED` classifies the process as EcoQoS (lower frequency, efficient cores). "should not be used for performance critical or foreground user experiences". Without it, Windows infers QoS heuristically | Windows 11 adds EcoQoS (LowQoS before) | [Microsoft: SetProcessInformation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessinformation) | 2024-10-31 | High |
| 7.3 | **Timer resolution**: `PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION` opts out. "By default in Windows 11 if a window owning process becomes fully occluded, minimized, or otherwise non-visible … and non-audible, Windows may automatically ignore the timer resolution request" | — | Same as 7.2 | 2024-10-31 | High |
| 7.4 | **`SetProcessInformation` needs `PROCESS_SET_INFORMATION` on the target**, so the broker can set EcoQoS on child processes it created | 1 access right | Same as 7.2 | 2024-10-31 | High |
| 7.5 | **`SetThreadExecutionState`, `WM_POWERBROADCAST`, `RegisterSuspendResumeNotification` and `RegisterPowerSettingNotification` (energy saver / battery saver GUIDs) are the notification side**. Not re-read this session | — | — | — | Medium |

**Implications:** the Screen Wake Lock API and media playback map to broker-held power requests with a reason string. Background tabs' renderers get EcoQoS set by the broker. Izcal must expect its own occluded windows to lose high timer resolution on Windows 11, and test animation timing under it.

---

## 8. Launch at login and background mode

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **`StartupTask` needs a package manifest** (`windows.startupTask`, `EntryPoint="Windows.FullTrustApplication"`). Packaged desktop apps see no consent dialog and may set `Enabled="true"`, but the user must launch the app once. The user controls it afterwards in Settings > Startup or the Task Manager Startup tab. `DisabledByUser` cannot be overridden | Windows 10 1607+ (desktop) | [Microsoft: StartupTask](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask) | 2021-05-18 | High |
| 8.2 | **Unpackaged apps use HKCU `…\CurrentVersion\Run` or a Startup-folder shortcut**. Both are listed in Task Manager's Startup apps (with "startup impact") and in Settings > Apps > Startup, where the user can disable them | — | Not re-read this session | — | Medium |
| 8.3 | **The notification-area icon (`Shell_NotifyIcon`, NOTIFYICONDATA with a GUID) is the documented background affordance**. On Windows 11, icons sit in the overflow by default unless the user shows them | — | Not re-read this session | — | Medium |

**Implications:** use the Run key for unpackaged installs and StartupTask when the identity package is present. A "keep running in background" mode needs a broker-owned tray icon, and should be off by default.

---

## 9. Native file dialogs

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **`IFileOpenDialog`/`IFileSaveDialog` (Common Item Dialog) are created in-process** (`CLSCTX_INPROC_SERVER`) after `CoInitializeEx(COINIT_APARTMENTTHREADED)` on a UI thread. `IModalWindow::Show(hwndOwner)`; cancel = `HRESULT_FROM_WIN32(ERROR_CANCELLED)`. Folder picking uses `FOS_PICKFOLDERS`; `FOS_FORCEFILESYSTEM` restricts results to file-system items | 1 STA thread | [Microsoft: Common Item Dialog](https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog) | updated 2026-07-17 | High |
| 9.2 | **The dialog browses the full Shell namespace** (`IShellItem`). Shell namespace extensions, thumbnail/preview/property handlers and context-menu handlers therefore load into the calling process. This follows from the in-proc design; Microsoft does not state it on this page | — | Same as 9.1 | — | Inference |
| 9.3 | **`SetClientGuid` keeps separate last-folder state per dialog purpose** (Save vs Upload) | — | Same as 9.1 | — | High |

**Implications:** the dialog cannot run in an AppContainer renderer, because it needs user files and loads third-party shell DLLs. Running it in the broker exposes the broker to crashes and code from those extensions. Recommendation: a dedicated medium-IL, same-user *dialog helper* process. It owns an STA thread, is parented to the broker's HWND (cross-process owner), and returns only the chosen paths or open handles over Izcal IPC. A crash in a shell extension then kills the helper, not the browser. Black-box test: cross-process owner-window modality and focus return on 24H2/25H2.

---

## 10. DPI, multi-monitor and window chrome

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | **Per-Monitor v2** (manifest `dpiAwareness` = `PerMonitorV2`; Windows 10 1703+) gives: DPI-change notification to top-level *and child* HWNDs, raw pixels, no bitmap stretching, automatic non-client scaling, scaled `CreateDialog` dialogs, and theme bitmaps rendered at the right DPI. PMv1 is "very limited" | 1703 (April 2017) | [Microsoft: High DPI desktop application development](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows) | 2026-09-10 | High |
| 10.2 | **Use `WM_DPICHANGED`'s suggested rectangle**: it keeps the cursor in the same relative position while dragging between monitors and prevents a recursive DPI-change loop. Use `WM_GETDPISCALEDSIZE` to supply a custom size | 2 failure modes | Same as 10.1 | 2026-09-10 | High |
| 10.3 | **Cross-process parenting with mismatched DPI awareness forces a reset**: since 1703, a cross-process `CreateWindow` with a parent of a different awareness resets the *caller's* process; a cross-process `SetParent` resets the child window's process | 4-cell rule table | Same as 10.1 | 2026-09-10 | High |
| 10.4 | **Use the per-DPI APIs**: `GetDpiForWindow`, `GetSystemMetricsForDpi`, `AdjustWindowRectExForDpi`, `SystemParametersInfoForDpi`. Many legacy APIs return values virtualized to the thread's DPI context | 4 replacements | Same as 10.1 | 2026-09-10 | High |
| 10.5 | **Snap Layouts with a custom title bar**: return `HTMAXBUTTON` from `WM_NCHITTEST` over the maximize button. Support a minimum width of at most 500 epx (330 epx recommended) | 500 / 330 epx | [Microsoft: Support snap layouts](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-snap-layout-menu) | 2026-06-16 | High |
| 10.6 | **DWM attributes**: `DWMWA_USE_IMMERSIVE_DARK_MODE` (20), `WINDOW_CORNER_PREFERENCE` (33), `BORDER_COLOR`, `CAPTION_COLOR`, `TEXT_COLOR` (all build 22000+); `DWMWA_SYSTEMBACKDROP_TYPE` (Mica/Acrylic/Tabbed; build 22621+); `REDIRECTIONBITMAP_ALPHA` (26100+); `BORDER_MARGINS` ("upcoming update to 26100") | 22000 / 22621 / 26100 | [Microsoft: DWMWINDOWATTRIBUTE](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute) | updated 2026-03-09 | High |
| 10.7 | **Monitor enumeration** (`MonitorFromWindow`, `EnumDisplayMonitors`, `GetMonitorInfo`) and `WM_DISPLAYCHANGE` are the topology sources. Not re-read this session | — | — | — | Medium |

**Implications:** all top-level HWNDs live in the broker, which is PMv2. If the GPU process ever creates child HWNDs under a broker window, it must have the *same* DPI awareness, or row 10.3 resets a process. A custom tab strip must hit-test `HTMAXBUTTON` and `HTCAPTION` itself. Mica is available only on 22621+, with a fallback below that.

---

## 11. System appearance settings

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | **Dark mode detection, as Microsoft documents it for Win32**: `UISettings().GetColorValue(UIColorType::Foreground)`; a light foreground means dark mode. Watch `UISettings.ColorValuesChanged` | 1 heuristic | [Microsoft: Support Dark and Light themes in Win32 apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-windows-themes) | 2026-06-16 | High |
| 11.2 | **Title bars default to light "for compatibility reasons"**; opt in per window with `DWMWA_USE_IMMERSIVE_DARK_MODE` = TRUE, and reset on change | — | Same as 11.1; row 10.6 | 2026-06-16 | High |
| 11.3 | **The undocumented `HKCU\…\Themes\Personalize\AppsUseLightTheme` value and `WM_SETTINGCHANGE` with lParam "ImmersiveColorSet" are widely used but not the documented contract** | — | Absence in 11.1 | — | Medium |
| 11.4 | **Accent colour** (`UIColorType::Accent*`), **text scaling** (`UISettings.TextScaleFactor`, `TextScaleFactorChanged`), **animations** (`UISettings.AnimationsEnabled`, `SPI_GETCLIENTAREAANIMATION`), **high contrast** (`SPI_GETHIGHCONTRAST` / `HCF_HIGHCONTRASTON`), **change events** (`WM_SETTINGCHANGE`, `WM_THEMECHANGED`, `WM_SYSCOLORCHANGE`). Not re-read this session | — | — | — | Medium |

**Implications:** the broker reads these settings and pushes them to renderers as values: `prefers-color-scheme`, `prefers-contrast`, `forced-colors` with the system colour set, `prefers-reduced-motion`, and the text scale factor. Renderers never query them.

---

## 12. Credentials and speech

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | **DPAPI (`CryptProtectData`) encrypts to the user's logon credentials**; Credential Manager (`CredWriteW`, `CRED_TYPE_GENERIC`) stores per-user secrets, with a blob limit of `CRED_MAX_CREDENTIAL_BLOB_SIZE` (5×512 bytes). `UserConsentVerifier.RequestVerificationAsync` (or the interop `RequestVerificationForWindowAsync`) prompts for Windows Hello. None re-read this session | 2560-byte blob | — | — | Medium |
| 12.2 | **`Windows.Media.SpeechSynthesis.SpeechSynthesizer`**: `AllVoices`, `DefaultVoice`, SSML 1.1, `Options`. "Only Microsoft-signed voices installed on the system can be used." Windows 10 10240+; usable from desktop apps through WinRT activation | — | [Microsoft: SpeechSynthesizer](https://learn.microsoft.com/en-us/uwp/api/windows.media.speechsynthesis.speechsynthesizer) | updated 2025-11-21 | High |
| 12.3 | **SAPI 5 (`ISpVoice`) is the older COM engine with its own voice registry (third-party voices)**. OneCore voices are a separate set; the .NET `System.Speech` wrapper over SAPI "is not accepting new features" | — | [NuGet System.Speech mirror](https://nuget.rza.ru/packages/system.speech) | 2025 | Low |
| 12.4 | **Whether Narrator's "natural voices" (Windows 11) are exposed to third-party apps through `SpeechSynthesizer` or SAPI: no Microsoft statement found** | — | Search 2026-10-24 | — | Gap |

**Implications:** the Web Speech API's `speechSynthesis.getVoices()` should merge OneCore voices (WinRT) and SAPI 5 voices, from a broker-side or medium-IL speech service. Black-box test: natural-voice enumeration on 24H2/25H2 through both APIs.

---

## 13. Clipboard and drag-and-drop

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | **Clipboard history and cloud sync opt-outs are registered formats**: `ExcludeClipboardContentFromMonitorProcessing` (any data; excludes the item from history *and* cloud sync), `CanIncludeInClipboardHistory` (DWORD 0/1), `CanUploadToCloudClipboard` (DWORD 0/1) | 3 formats | [Microsoft: Clipboard Formats](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats) | 2025-04-15 | High |
| 13.2 | **Put the richest format first**; the system synthesizes `CF_UNICODETEXT`↔`CF_TEXT` and `CF_DIB`↔`CF_BITMAP`/`CF_DIBV5`. Prefer `CF_DIB`/`CF_DIBV5` for bitmaps; DIBV5 with colour-space information is converted to sRGB on request | 16 synthesized pairs | Same as 13.1 | 2025-04-15 | High |
| 13.3 | **Browser-relevant formats**: `CF_UNICODETEXT`; registered "HTML Format" (CF_HTML, with a header of byte offsets: Version/StartHTML/EndHTML/StartFragment/EndFragment/SourceURL); "PNG"; `CF_HDROP` (file paths); `CFSTR_FILEDESCRIPTORW` + `CFSTR_FILECONTENTS` for virtual (not-yet-on-disk) files in drag-out. None re-read this session | — | — | — | Medium |
| 13.4 | **UIPI blocks drag/drop and window messages from a lower-IL process to a higher-IL window**. A medium-IL browser cannot accept drops from, or post to, an elevated app's window, and AppContainer/low-IL processes cannot post to medium-IL windows. Not re-read this session | — | — | — | Medium |

**Implications:** the OLE clipboard and `RegisterDragDrop`/`DoDragDrop` live in the broker, on the thread that owns the HWND. Renderers send typed payloads. For "copy password" style actions, the broker adds `ExcludeClipboardContentFromMonitorProcessing`. Drag-out of a download uses FileGroupDescriptorW/FileContents, with content streamed from the network/storage service.

---

## 14. Download scanning and Mark of the Web

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 14.1 | **`IAttachmentExecute`** (CLSID_AttachmentServices, in-proc) is the shell's safe-download/attachment API: `SetClientGuid`, `SetLocalPath`, `SetSource` (URL), `SetReferrer` (zone), `CheckPolicy`, `Save`/`SaveWithUI` (applies the zone identifier, i.e. MOTW, and invokes the registered antivirus), `Prompt`, `Execute`. Its documented audience is "email and messaging attachments" | Windows XP SP2+ | [Microsoft: IAttachmentExecute](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iattachmentexecute) | 2024-02-22 | High (API); Medium (Save → AV scan + Zone.Identifier, not stated on this page) |
| 14.2 | **AMSI** is a vendor-neutral interface that lets apps and services submit "file and memory or stream" content and "content source URL/IP" to the installed antimalware product, with sessions to correlate fragments. Windows integrates it into UAC, PowerShell, WSH, JavaScript/VBScript and Office VBA. Since 1903, unsigned provider DLLs may not load | 5 integrated components | [Microsoft: AMSI](https://learn.microsoft.com/en-us/windows/win32/amsi/antimalware-scan-interface-portal) | 2025-07-09 | High |
| 14.3 | **For a browser, `IAttachmentExecute::Save` is the established route for completed downloads (MOTW + AV hook); AMSI is aimed at script/dynamic content scanning** | — | Rows 14.1–14.2 | — | Inference |

**Implications:** after a download finishes, the broker (or a medium-IL download helper, since `IAttachmentExecute` is in-proc and loads AV components) calls `SetSource`/`SetReferrer`/`Save`, and treats a failing HRESULT as "blocked by policy/AV". AMSI is optional for Izcal.

---

## 15. Where each API can run (sandbox placement)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 15.1 | **AppContainer = low IL + capability SIDs; no access to user files, most registry, other apps or network without a capability** (row 0.5). UIPI then stops messages to medium-IL windows (row 13.4) | — | Rows 0.5, 13.4 | — | High / Medium |
| 15.2 | **Placement by API**, derived from the rows above | — | Sections 1–14 | — | Inference |

| API family | Needs | Runs in |
|---|---|---|
| Default-app registration (HKCU/HKLM), `QueryCurrentDefault`, `ms-settings:` launch | User registry, ShellExecute | Installer (HKLM) / broker (HKCU) |
| Single-instance lock, `AllowSetForegroundWindow`, `RegisterApplicationRestart` | Medium IL, user session | Broker |
| AUMID, jump lists, `ITaskbarList3`, `TaskbarManager` | Shell COM, owner of HWNDs | Broker |
| Toast/app notifications, COM activator | User COM registration, not elevated | Broker |
| SMTC, share source, `UserConsentVerifier` | HWND owner | Broker |
| Share target, StartupTask | Package identity | Broker + identity package |
| Power requests, EcoQoS on children | `PROCESS_SET_INFORMATION` | Broker |
| Common Item Dialog | User files, in-proc shell extensions | Medium-IL dialog helper (recommended) |
| `IAttachmentExecute::Save` | User files, AV components | Broker or medium-IL download helper |
| OLE clipboard, drag-drop | HWND thread | Broker |
| DPI/DWM/theme reads | HWND owner | Broker → renderers as values |
| DPAPI, Credential Manager | User logon secrets | Broker or storage service at medium IL (not AppContainer, since DPAPI is tied to the user) |
| Speech (OneCore/SAPI) | Audio output, voice registry | Speech utility; capability set **to be measured** |

**Implications:** nothing in this note needs to run in a renderer. The broker holds every shell-facing API. Two medium-IL helpers (dialogs; optionally downloads) isolate in-process third-party shell code from the broker. Black-box test: whether `SpeechSynthesizer` and SAPI run inside an AppContainer with no capabilities.

---

## 16. Target OS versions and Windows on Arm

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 16.1 | **Windows 10 Home/Pro reached end of support on 14 October 2025**; 22H2 is the final version | 2025-10-14 | [Microsoft Lifecycle: Windows 10 Home and Pro](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro) | 2024-09-17 | High |
| 16.2 | **Consumer ESU (22H2 only) was to end on 13 October 2026; a June 2026 report says Microsoft extended it to 12 October 2027** | 2026-10-13 → 2027-10-12? | [Microsoft Q&A](https://learn.microsoft.com/en-ca/answers/questions/5805066/windows-esu-does-enrollment-end-on-13-oct-2026-or); [Pureinfotech](https://pureinfotech.com/windows-10-esu-extended-2027-microsoft-update/) | 2026 | Low (extension unverified at Microsoft) |
| 16.3 | **Several 2026 features above need 24H2/25H2 (26100/26200)**: taskbar pinning without a LAF token (KB5074105), `DWMWA_REDIRECTIONBITMAP_ALPHA`; Mica needs 22621. The identity package needs 19041+ | 19041 / 22621 / 26100 / 26200 | Rows 0.2, 3.6, 10.6 | — | High |
| 16.4 | **Windows on Arm**: the identity package is architecture-neutral (`ProcessorArchitecture="neutral"`) | — | Row 0.2 | — | High |
| 16.5 | **Arm64 share, x64 emulation (Prism) behaviour and native Arm64 requirements: not researched in this budget** | — | — | — | Gap |

**Implications:** recommendation: target Windows 11 (22H2+ minimum, 24H2/25H2 primary) on x64 and Arm64 natively. Run Windows 10 22H2 on a best-effort, feature-degraded path only while ESU lasts, with no Mica and no taskbar pin API. Re-check the ESU date at Microsoft before committing.

---

## Open questions for black-box measurement

1. Exact argv produced by `ShellExecute` for crafted `https:`/`izcal:` URIs (quotes, trailing backslash, `%22`, very long) under Explorer, Run, and another browser.
2. Which associations the EEA "Set default" button sets for a third-party browser on 25H2, and whether it pins the browser.
3. Whether a hand-registered COM toast activator (no Windows App SDK) activates a cold-started unpackaged app on 24H2/25H2, and its behaviour under Focus.
4. Whether `TaskbarManager.RequestPinCurrentAppAsync` works for an unpackaged Win32 app (no identity) after KB5074105.
5. Whether `SpeechSynthesizer.AllVoices` and SAPI list Narrator natural voices, and whether either runs inside an AppContainer.
6. Cross-process owner-window behaviour (modality, focus return, DPI) for a file dialog shown by a medium-IL helper over a broker HWND.
7. Hash or UserChoice changes in 2025–2026 (the "UserChoiceLatest" claim): observe the registry only; no reverse engineering.
