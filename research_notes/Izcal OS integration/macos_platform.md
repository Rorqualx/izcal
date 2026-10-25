# Izcal OS integration: macOS platform today

Compiled 24 October 2026. Primary sources were preferred (Apple developer documentation, Apple release notes, WWDC session transcripts, Apple support and deployment guides, the `open(1)` man page).

M-1 note: this note uses only published prose, vendor documentation, man pages and blog posts. No third-party project's source code, headers, commits, diffs or code reviews were opened, and no SDK header was read. Search results that pointed at browser engine source trees were discarded unread. A documented API describes a contract; it is not a template to copy from an implementation.

Confidence: **High** = the primary documentation text was read and the claim checked in it; **Medium** = an official page's summary, a secondary source, or a claim from long-standing platform behaviour not re-read today; **Low** = secondary source only; **Gap** = no published evidence found; **Inference** = my conclusion from the cited rows, not a published statement.

**Method notes.**
- Apple documentation was read through its published JSON alternates (`developer.apple.com/tutorials/data/documentation/<path>.json`), which carry the same text and the availability metadata of the rendered page. Availability versions in the Numbers column come from that metadata.
- The machine used for the man-page checks runs macOS 27.0.1. macOS 27 ("Golden Gate") shipped in September 2026; macOS 26 Tahoe is the previous release and the last one for Intel Macs (row 16.6).
- About ten web searches were used, on the items the brief ranked highest (default-browser prompts, pasteboard privacy, cooperative activation, Intel end, Gatekeeper). Later rows are direct reads of Apple pages. Rows not re-read at the source are graded Medium or Low.
- "Read 2026-10-24" means the page was read that day and shows no date of its own.

---

## 1. Default browser

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Capability is declared, not granted.** An app can only be offered as a handler for schemes it lists in Info.plist `CFBundleURLTypes`; `LSSetDefaultHandlerForURLScheme`'s page says "URL handling capability is determined according to the value of the CFBundleURLTypes key". Document types (`public.html`, `public.xhtml`, …) are declared in `CFBundleDocumentTypes` | macOS 10.0+ (both keys) | [Apple: CFBundleURLTypes](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleurltypes); [LSSetDefaultHandlerForURLScheme](https://developer.apple.com/documentation/coreservices/1447760-lssetdefaulthandlerforurlscheme) | read 2026-10-24 | High |
| 1.2 | **The current API is `NSWorkspace.setDefaultApplication(at:toOpenURLsWithScheme:completion:)`** (and `…toOpen: UTType` for content types). Apple: "If a change requires user consent, the system asks the user for consent asynchronously before invoking the completion handler." The page does not say which changes need consent or what error a refusal returns | macOS 12.0+ | [Apple: setDefaultApplication(at:toOpenURLsWithScheme:completion:)](https://developer.apple.com/documentation/appkit/nsworkspace/setdefaultapplication(at:toopenurlswithscheme:completion:)); [setDefaultApplication(at:toOpen:completion:)](https://developer.apple.com/documentation/appkit/nsworkspace/setdefaultapplication(at:toopen:completion:)) | read 2026-10-24 | High (text); Gap (error code) |
| 1.3 | **The legacy Launch Services setters are deprecated**: `LSSetDefaultHandlerForURLScheme` is deprecated in macOS 12.0 (available since 10.4); `LSCopyDefaultHandlerForURLScheme` is its getter | deprecated 12.0 | [Apple: LSSetDefaultHandlerForURLScheme](https://developer.apple.com/documentation/coreservices/1447760-lssetdefaulthandlerforurlscheme) | read 2026-10-24 | High |
| 1.4 | **Reading the default is unprompted**: `NSWorkspace.urlForApplication(toOpen:)` returns the app that would open a URL ("the programmatic equivalent of double-clicking a document in the Finder"), safe from any thread; nil if none | macOS 10.6+ | [Apple: urlForApplication(toOpen:)](https://developer.apple.com/documentation/appkit/nsworkspace/urlforapplication(toopen:)-7qkzf) | read 2026-10-24 | High |
| 1.5 | **Changing the `http` handler (the default browser) has prompted the user since about macOS 10.13**; the dialog offers **"Use"** or **"Keep"**; "Keep" leaves the old browser. An unanswered dialog leaves the caller waiting indefinitely | 2 buttons; since ~10.13 (author hedges) | [Scripting OS X: macOS 26.4 brings more default app confirmation prompts](https://scriptingosx.com/2026/03/macos-26-4-brings-more-default-app-confirmation-prompts/) | 2026-03-24 | Medium |
| 1.6 | **macOS 26.4 extended the prompt to all file-type/UTI default changes**; through 26.3, file types and non-http schemes could be changed via the API silently. Each type is confirmed individually; Finder's Get Info change is not prompted. Apple offers no MDM/profile control for macOS default apps (the default-browser MDM command is iOS/visionOS only) | 26.4; 1 prompt per type | Same as 1.5 | 2026-03-24 | Medium |
| 1.7 | **http and https cannot be set to different browsers** in the user-facing flow; the change is made as one browser choice | 1 choice | [Jamf Community: managing the default browser](https://community.jamf.com/tech-thoughts-180/managing-the-default-browser-in-macos-53418) | read 2026-10-24 | Low |
| 1.8 | **The user-facing control is System Settings > Desktop & Dock > Default web browser**, which lists apps Launch Services knows can handle http/https. When Launch Services registers a newly copied app (on first launch, or when Finder notices it in /Applications) is not documented in a current Apple page; `lsregister` is an undocumented tool | — | Gap (no current Apple page found) | — | Medium (settings path); Gap (timing) |

**Implications:** Izcal's broker asks once, at a moment the user chose ("Make Izcal default"), calls `setDefaultApplication` for `http` and lets the OS dialog be the only consent; it must treat "no completion yet" as a normal state, never block on it, and re-read the truth with `urlForApplication(toOpen:)` rather than trust the completion. Because macOS 26.4 prompts per UTI, Izcal must not loop over `public.html`, `.webarchive`, `.pdf` etc. at first run: one http prompt only, file types left to the user. The refusal error code is a black-box measurement item.

---

## 2. URL and file opens, single instance, activation

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **`application(_:open:)` receives every URL and document open** for declared URL and document types; if implemented, AppKit does not call `application(_:openFile:)`/`openFiles:` | macOS 10.13+ | [Apple: application(_:open:)](https://developer.apple.com/documentation/appkit/nsapplicationdelegate/application(_:open:)) | read 2026-10-24 | High |
| 2.2 | **Opens arrive as Apple Events** (`kAEGetURL` "GURL", `kAEOpenDocuments` "odoc") through `NSAppleEventManager.setEventHandler(_:andSelector:forEventClass:andEventID:)`. A launch-time open can be delivered before `applicationDidFinishLaunching`, so a custom handler is installed in `applicationWillFinishLaunching(_:)` | macOS 10.0+ / 10.10+ | [Apple: setEventHandler](https://developer.apple.com/documentation/foundation/nsappleeventmanager/seteventhandler(_:andselector:foreventclass:andeventid:)); [applicationWillFinishLaunching](https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationwillfinishlaunching(_:)) | read 2026-10-24 | High (APIs); Medium (ordering) |
| 2.3 | **Launch Services reuses a running instance by default.** `NSWorkspace.OpenConfiguration.createsNewApplicationInstance` defaults to false, "which causes the system to open the already running app when present"; `open -n` forces a new instance | macOS 10.15+ | [Apple: createsNewApplicationInstance](https://developer.apple.com/documentation/appkit/nsworkspace/openconfiguration/createsnewapplicationinstance); `open(1)` man page, macOS 27.0.1 | read 2026-10-24 | High |
| 2.4 | **`LSMultipleInstancesProhibited` is about users, not instances**: "whether more than one user can launch the app simultaneously" (fast user switching). It is not the single-instance switch for one user | macOS 10.0+ | [Apple: LSMultipleInstancesProhibited](https://developer.apple.com/documentation/bundleresources/information-property-list/lsmultipleinstancesprohibited) | read 2026-10-24 | High |
| 2.5 | **Executing the binary inside the bundle directly (a shell, a script, `exec`) bypasses Launch Services** and starts a second process with no message to the first. A browser that accepts `izcal https://…` from a terminal needs its own "already running?" check and handoff | — | Inference from 2.3 (reuse is a Launch Services behaviour) | — | Inference |
| 2.6 | **macOS 14 made activation cooperative**: "Activate is now a request, as opposed to a command"; `activate(ignoringOtherApps:)` and `activateIgnoringOtherApps` are deprecated and the flag is ignored; an app hands focus on with `yieldActivation(to:)`; "NSWorkspace automatically handles this for you when opening URLs or applications" | macOS 14.0+ | [WWDC23 10054: What's new in AppKit](https://developer.apple.com/videos/play/wwdc2023/10054/); [Apple: activate()](https://developer.apple.com/documentation/appkit/nsapplication/activate()); [yieldActivation(to:)](https://developer.apple.com/documentation/appkit/nsapplication/yieldactivation(to:)) | 2023-06; read 2026-10-24 | High |
| 2.7 | **`activate()` "doesn't guarantee app activation"**; `NSRunningApplication.activate(from:options:)` returns whether the system allowed it. Yielding does not deactivate the yielder; the target must still request activation | macOS 14.0+ | [Apple: activate(from:options:)](https://developer.apple.com/documentation/appkit/nsrunningapplication/activate(from:options:)) | read 2026-10-24 | High |
| 2.8 | **Activation policy** (`NSApplication.ActivationPolicy`: regular / accessory / prohibited) and `LSUIElement` ("an agent app that runs in the background and doesn't appear in the Dock") decide whether a process shows in the Dock and can be activated | macOS 10.0+ | [Apple: ActivationPolicy](https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum); [LSUIElement](https://developer.apple.com/documentation/bundleresources/information-property-list/lsuielement) | read 2026-10-24 | High |

**Implications:** Only the broker is a Launch Services "app"; it installs the GURL/odoc handlers in `applicationWillFinishLaunching` and queues opens until a profile is loaded. Izcal needs its own single-instance channel (a per-user socket or Mach service owned by the broker) for command-line launches, and that channel must be authenticated as same-build, like the IPC. When a link opened in another app arrives, Izcal requests activation and accepts that macOS 14+ may refuse; when Izcal opens a file in another app it goes through `NSWorkspace`, which yields correctly.

---

## 3. Dock

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Dock menu** comes from `applicationDockMenu(_:)` (or a nib named by `AppleDockMenu`); on selection "the dock messages your application", so the items act in the app process | — | [Apple: applicationDockMenu(_:)](https://developer.apple.com/documentation/appkit/nsapplicationdelegate/applicationdockmenu(_:)) | read 2026-10-24 | High |
| 3.2 | **`NSDockTile` gives a badge string and custom drawing, not progress.** App tile: "may be badged with a short custom string"; drawing your own content needs "a custom content view". There is no progress API; a download bar is drawn in that view. Window tiles cannot carry a custom badge on the default image | macOS 10.5+ | [Apple: NSDockTile](https://developer.apple.com/documentation/appkit/nsdocktile); [badgeLabel](https://developer.apple.com/documentation/appkit/nsdocktile/badgelabel) | read 2026-10-24 | High |
| 3.3 | **Recent documents**: `NSDocumentController.noteNewRecentDocumentURL(_:)` adds an Open Recent item; non-NSDocument apps must also handle the reopen via the app delegate | — | [Apple: noteNewRecentDocumentURL](https://developer.apple.com/documentation/appkit/nsdocumentcontroller/notenewrecentdocumenturl(_:)) | read 2026-10-24 | High |
| 3.4 | **System window tabs**: `NSWindow.tabbingMode` (automatic/preferred/disallowed), default automatic, which follows the user's tabbing preference; set before showing the window | macOS 10.12+ | [Apple: tabbingMode](https://developer.apple.com/documentation/appkit/nswindow/tabbingmode-swift.property) | read 2026-10-24 | High |
| 3.5 | **A browser with its own tab strip sets `.disallowed`**, otherwise the user's "prefer tabs" setting nests browser windows in system tabs (tabs inside tabs) | — | Inference from 3.4 | — | Inference |
| 3.6 | **Full screen and Spaces bugs are still being fixed** in macOS 27 (window lost when exiting full screen via Mission Control; Dock persisting in full screen) | 2 fixes in 27.0 | [Apple: macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) (Window Management, Dock) | 2026-09 | High |

**Implications:** The broker owns the Dock tile: badge for unread web-app notifications, a custom content view for download progress, and a Dock menu (New Window, New Private Window, profiles). Windows set `tabbingMode = .disallowed`.

---

## 4. Notifications

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **`UNUserNotificationCenter` is the macOS API** for alerts, sounds and badges, with categories and actions, delivered-notification management and settings queries | macOS 10.14+ | [Apple: UNUserNotificationCenter](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter) | read 2026-10-24 | High |
| 4.2 | **One prompt per app, ever**: "The first time your app makes this authorization request, the system prompts … and records that response. Subsequent authorization requests don't prompt." Provisional authorization delivers quietly to Notification Center history without a prompt | 1 prompt | [Apple: Asking permission to use notifications](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications) | read 2026-10-24 | High |
| 4.3 | **Actions** are declared as `UNNotificationCategory` objects with action buttons and registered with `setNotificationCategories(_:)` | macOS 10.14+ | [Apple: UNNotificationCategory](https://developer.apple.com/documentation/usernotifications/unnotificationcategory) | read 2026-10-24 | High |
| 4.4 | **Identity is the bundle.** Authorization, the Notifications settings row and the badge belong to the app bundle. Apple's own web apps (Safari "Add to Dock", macOS 14+) are separate apps: each "appears in Notifications settings" after granting, shows its unread count "as a red badge on the app's icon in the Dock", and "shares no browsing history, cookies, website data, or settings with Safari" | 1 identity per bundle | [Apple Support: Use Safari web apps on Mac](https://support.apple.com/en-us/104996); [WebKit: Safari 17.0 features](https://webkit.org/blog/14445/webkit-features-in-safari-17-0/) | 2026-05-27; 2023-09 | High |
| 4.5 | **Focus filters** use App Intents `SetFocusFilterIntent`, which the system runs when a Focus engages, so the app can change configuration and filter notifications | macOS 13.0+ | [Apple: Focus](https://developer.apple.com/documentation/appintents/focus); [SetFocusFilterIntent](https://developer.apple.com/documentation/appintents/setfocusfilterintent) | read 2026-10-24 | High |
| 4.6 | **Delivery when the app is not running**: scheduled local notifications are shown by the system without the app. Web Push for a not-running browser needs the browser's own push channel or APNs (`aps-environment` entitlement via a provisioning profile, which Developer ID supports) | — | [Apple: Asking permission…](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications) (system shows them "when your app isn't running") | read 2026-10-24 | High (local); Medium (APNs for Developer ID) |
| 4.7 | **Developer ID apps can use the API**; there is no App-Store-only gate in Apple's pages | — | Gap (no explicit statement; none of the pages restrict it) | — | Medium |

**Implications:** Notifications are posted by the broker, the only bundle with the user-visible identity. Per-site permission is Izcal's own layer on top of one OS grant. If Izcal ships installed web apps with their own Dock icon, Focus row and notification switch, each needs its own small signed app bundle (Safari's model); a profile alone cannot have a separate OS notification identity.

---

## 5. Media controls

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **`MPNowPlayingInfoCenter`** publishes the now-playing metadata the system shows in Control Center media controls | macOS 10.12.2+ | [Apple: MPNowPlayingInfoCenter](https://developer.apple.com/documentation/mediaplayer/mpnowplayinginfocenter) | read 2026-10-24 | High |
| 5.2 | **On macOS the app must set `playbackState` itself**: "This property only applies to macOS. You must set this property every time the app begins or halts playback, otherwise remote control functionality may not work as expected" | macOS 10.12.2+ | [Apple: playbackState](https://developer.apple.com/documentation/mediaplayer/mpnowplayinginfocenter/playbackstate) | read 2026-10-24 | High |
| 5.3 | **`MPRemoteCommandCenter.shared()`** delivers play/pause/next/seek "sent by external accessories and system controls" | macOS 10.12.2+ | [Apple: MPRemoteCommandCenter](https://developer.apple.com/documentation/mediaplayer/mpremotecommandcenter) | read 2026-10-24 | High |
| 5.4 | **Media keys (F7–F9) and headset buttons go to the current now-playing app**, which is the app that most recently set playing state. Apple documents no arbitration rule when several apps play | — | [Hashrocket TIL](https://til.hashrocket.com/posts/fcdnesxsev-control-video-playback-with-keyboard-controls); [Apple Developer Forums 685333](https://developer.apple.com/forums/thread/685333) | read 2026-10-24 | Low |

**Implications:** One process, the broker (or the audio service if it has an app identity; see 16), owns the now-playing session and maps it to the single "active media session" Izcal picks among tabs. Routing between Izcal and other players is the OS's choice and must be measured.

---

## 6. Share, Services, Handoff

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **`NSSharingServicePicker`** shows the share UI; macOS 13+ shows "a popover with a preview of the item and the list of services" (a menu in 12 and earlier) | macOS 10.8+ | [Apple: NSSharingServicePicker](https://developer.apple.com/documentation/appkit/nssharingservicepicker) | read 2026-10-24 | High |
| 6.2 | **`NSSharingService`** shares URLs, strings, images, file URLs or any `NSPasteboardWriting` item to Mail, Messages, AirDrop and installed share extensions | macOS 10.8+ | [Apple: NSSharingService](https://developer.apple.com/documentation/appkit/nssharingservice) | read 2026-10-24 | High |
| 6.3 | **Being a share target needs an app extension.** Apple's guide: macOS extension templates "include the App Sandbox … by default"; extensions ship inside a containing app; outside the Mac App Store "Gatekeeper prevents the extension from running until the user opens and approves the containing app". Submitting to the store is "recommended … but not required" | 1 extension bundle | [Apple: App Extension Programming Guide, Creating an App Extension](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/ExtensionCreation.html) | archived guide | High (text); Medium (sandbox mandatory) |
| 6.4 | **Services menu** is declared in Info.plist `NSServices` (`NSMessage`, `NSSendTypes`, `NSReturnTypes`, `NSMenuItem`, `NSRequiredContext`); without `NSRequiredContext` a service is registered but not shown. Enablement in Keyboard settings is not described on that page | macOS 10.0+ | [Apple: Services Properties](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/SysServices/Articles/properties.html) | archived guide | High (keys); Gap (enablement) |
| 6.5 | **`NSUserActivity`** carries state for Handoff and Quick Notes | macOS 10.10+ | [Apple: NSUserActivity](https://developer.apple.com/documentation/foundation/nsuseractivity) | read 2026-10-24 | High |

**Implications:** Sharing out is a broker call with the page's URL and title. "Share to Izcal" from other apps is an optional sandboxed share extension that only forwards a URL to the broker; it is a separate signed bundle and its own attack surface, so it is a Stage 1 "maybe", not a requirement. A Services entry ("Open URL in Izcal", "Search with Izcal") is cheaper.

---

## 7. Power

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **`IOPMAssertionCreateWithName`**: "No special privileges are necessary … any process may activate a power assertion"; a name is required. Types include `PreventUserIdleDisplaySleep` (video) and `PreventUserIdleSystemSleep` (downloads, audio) | macOS 10.6+ | [Apple: IOPMAssertionCreateWithName](https://developer.apple.com/documentation/iokit/1557134-iopmassertioncreatewithname) | read 2026-10-24 | High |
| 7.2 | **`ProcessInfo.beginActivity(options:reason:)`** returns a token ended with `endActivity`; options include `userInitiated` ("exporting or downloading a user-specified file"), `idleSystemSleepDisabled`, `idleDisplaySleepDisabled` and `latencyCritical` ("Very few applications should need to use this") | macOS 10.9+ | [Apple: beginActivity](https://developer.apple.com/documentation/foundation/processinfo/beginactivity(options:reason:)); [ActivityOptions](https://developer.apple.com/documentation/foundation/processinfo/activityoptions) | read 2026-10-24 | High |
| 7.3 | **Sleep/wake**: `NSWorkspace.willSleepNotification` (observers "can delay sleep for up to 30 seconds"), `screensDidSleepNotification`; both only on `NSWorkspace.shared.notificationCenter` | 30 s max delay | [Apple: willSleepNotification](https://developer.apple.com/documentation/appkit/nsworkspace/willsleepnotification); [screensDidSleepNotification](https://developer.apple.com/documentation/appkit/nsworkspace/screensdidsleepnotification) | read 2026-10-24 | High |
| 7.4 | **Low Power Mode is on the Mac**: `ProcessInfo.isLowPowerModeEnabled` plus `NSProcessInfoPowerStateDidChange`; it reduces CPU/GPU performance and pauses discretionary work | macOS 12.0+ | [Apple: isLowPowerModeEnabled](https://developer.apple.com/documentation/foundation/processinfo/islowpowermodeenabled) | read 2026-10-24 | High |
| 7.5 | **`ProcessInfo.thermalState`**: "At higher thermal states your app should reduce usage of system resources" | macOS 10.10.3+ | [Apple: thermalState](https://developer.apple.com/documentation/foundation/processinfo/thermalstate-swift.property) | read 2026-10-24 | High |

**Implications:** Power assertions are per-process and unprivileged, so the media or download service can hold its own; the broker still needs to know them for the UI. Low Power Mode and thermal state are inputs to Izcal's scheduler (timer throttling, frame rate), read once in the broker and pushed to services.

---

## 8. Launch at login and background items

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **`SMAppService`** registers the main app, a LoginItem helper, a LaunchAgent or a LaunchDaemon that live inside the app bundle; it replaces `SMLoginItemSetEnabled` and hand-installed launchd plists | macOS 13.0+ | [Apple: SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice) | read 2026-10-24 | High |
| 8.2 | **`register()` semantics**: main app → launches at subsequent logins; LoginItem → starts now and is relaunched if it crashes or exits non-zero; LaunchAgent → bootstrapped now; LaunchDaemon → waits for admin approval. Errors `kSMErrorAlreadyRegistered`, `kSMErrorLaunchDeniedByUser`; `openSystemSettingsLoginItems()` opens the pane | macOS 13.0+ | [Apple: register()](https://developer.apple.com/documentation/servicemanagement/smappservice/register()); [openSystemSettingsLoginItems()](https://developer.apple.com/documentation/servicemanagement/smappservice/openSystemSettingsLoginItems()) | read 2026-10-24 | High |
| 8.3 | **Users see and control items in System Settings > General > Login Items**; registration posts a user notification; **macOS 26 adds a prompt to allow background tasks that stay active after the app quits**. MDM can pre-approve by bundle ID, team ID or label | macOS 13 / 26 | [Apple Platform Deployment: Manage login items and background tasks](https://support.apple.com/guide/deployment/manage-login-items-background-tasks-mac-depdca572563/web) | 2025-12-17 | High |
| 8.4 | **launchd no longer loads plists carrying the quarantine attribute** (macOS 27) | macOS 27.0 | [Apple: macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) (Launch Daemons and Agents) | 2026-09 | High |
| 8.5 | **Menu bar extras** are `NSStatusItem`s from `NSStatusBar.statusItem(withLength:)`, owned by an AppKit process | — | [Apple: NSStatusItem](https://developer.apple.com/documentation/appkit/nsstatusitem) | read 2026-10-24 | High |

**Implications:** "Open at login" is `SMAppService.mainApp` and nothing else. A background agent that keeps web push alive after quit would trigger the macOS 26 background-task prompt; Izcal should not ship one by default. A web-app agent, if ever needed, is a LoginItem bundle inside Izcal.app.

---

## 9. Native file dialogs

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 9.1 | **Open and Save panels always run out of process since macOS 10.15**, "regardless of whether the app is sandboxed"; the chosen file is added to the app's sandbox. Before 10.15 this was only for sandboxed apps | macOS 10.15+ | [Apple: NSOpenPanel](https://developer.apple.com/documentation/appkit/nsopenpanel); [NSSavePanel](https://developer.apple.com/documentation/appkit/nssavepanel) | read 2026-10-24 | High |
| 9.2 | **Persistent access across launches uses security-scoped bookmarks** (`withSecurityScope`, "for use in an app that adopts App Sandbox") | macOS 10.7+ | [Apple: withSecurityScope](https://developer.apple.com/documentation/foundation/nsurl/bookmarkcreationoptions/withsecurityscope); [Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox) | read 2026-10-24 | High |
| 9.3 | **macOS 14+ ties a sandbox container to the code signature** and prompts if another app (or an unsandboxed one) enters it; macOS 27 denies cross-team container access without a bundle ID | macOS 14 / 27 | [Apple: Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox); [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | read 2026-10-24 | High |
| 9.4 | **Panels from a helper process**: Apple documents panels for apps; whether a sandboxed, windowless helper may present one is not documented | — | Gap | — | Gap |

**Implications:** File pickers (`<input type=file>`, downloads "Save as", File System Access) are shown by the broker, attached to the tab's window, and the chosen path or an opened file descriptor is passed to the renderer or storage service. Renderers never ask the OS. The panel is already a system process, which is the Powerbox model Izcal wants.

---

## 10. Displays and DPI

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 10.1 | **`NSScreen.backingScaleFactor`** gives backing pixels per point; Apple prefers the view convert-to-backing methods | macOS 10.7+ | [Apple: backingScaleFactor](https://developer.apple.com/documentation/appkit/nsscreen/backingscalefactor) | read 2026-10-24 | High |
| 10.2 | **`NSWindow.didChangeBackingPropertiesNotification`** fires per window with backing scale and color space in userInfo | macOS 10.7+ | [Apple: didChangeBackingPropertiesNotification](https://developer.apple.com/documentation/appkit/nswindow/didchangebackingpropertiesnotification) | read 2026-10-24 | High |
| 10.3 | **`CGDisplayRegisterReconfigurationCallback`** is called twice per display (before and after) on add, remove or mode change | macOS 10.3+ | [Apple: CGDisplayRegisterReconfigurationCallback](https://developer.apple.com/documentation/coregraphics/cgdisplayregisterreconfigurationcallback(_:_:)) | read 2026-10-24 | High |
| 10.4 | **Per-display refresh**: `NSScreen.maximumFramesPerSecond` | macOS 12.0+ | [Apple: maximumFramesPerSecond](https://developer.apple.com/documentation/appkit/nsscreen/maximumframespersecond) | read 2026-10-24 | High |

**Implications:** Window-level scale changes are observed in the broker (it owns `NSWindow`) and forwarded to the GPU process and the renderer as a typed message; the graphics spec owns frame pacing.

---

## 11. System appearance and accessibility settings

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 11.1 | **Dark mode**: `NSApplication.effectiveAppearance` (observable); **accent**: `NSColor.controlAccentColor` (do not assume its color space) | macOS 10.14+ | [Apple: effectiveAppearance](https://developer.apple.com/documentation/appkit/nsapplication/effectiveappearance); [controlAccentColor](https://developer.apple.com/documentation/appkit/nscolor/controlaccentcolor) | read 2026-10-24 | High |
| 11.2 | **Display accessibility flags** on `NSWorkspace`: `accessibilityDisplayShouldIncreaseContrast`, `…ShouldDifferentiateWithoutColor` (10.10+), `…ShouldReduceMotion` (10.12+), `…ShouldReduceTransparency`; one change notification, `accessibilityDisplayOptionsDidChangeNotification`, only on the workspace notification center | macOS 10.10–10.12+ | [Apple: ShouldReduceMotion](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducemotion); [IncreaseContrast](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldincreasecontrast); [DifferentiateWithoutColor](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshoulddifferentiatewithoutcolor); [OptionsDidChange](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayoptionsdidchangenotification) | read 2026-10-24 | High |
| 11.3 | **There is no system-wide text-size API.** macOS 14+ has Accessibility > Display > Text size with per-app dropdowns for a few Apple apps; an Apple reply on the forums (macOS 14) said there is no API or change notification for third parties | 0 APIs | [Apple Developer Forums 746501](https://developer.apple.com/forums/thread/746501); [AbilityNet: Text size in macOS 26](https://mcmw.abilitynet.org.uk/how-to-make-the-text-larger-on-your-apple-mac-computer-using-the-text-size-options-in-macos-26-tahoe) | 2024; read 2026-10-24 | Medium |

**Implications:** The broker reads these once and on change, and pushes them as a typed "system preferences" message that the style engine maps to `prefers-color-scheme`, `prefers-contrast`, `prefers-reduced-motion`, `prefers-reduced-transparency` and the accent color. Renderers never call AppKit for them. Text zoom stays Izcal's own setting.

---

## 12. Scripting and automation

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 12.1 | **A scriptable app declares its dictionary** with Info.plist `OSAScriptingDefinition` (an `.sdef`) and handles the events in its main process | macOS 10.0+ | [Apple: OSAScriptingDefinition](https://developer.apple.com/documentation/bundleresources/information-property-list/osascriptingdefinition) | read 2026-10-24 | High (key); Medium (handling) |
| 12.2 | **Senders need consent, the recipient does not grant it.** `NSAppleEventsUsageDescription` is "required if your app uses APIs that send Apple events" (10.14+); the system lists sender→target pairs in Privacy & Security > **Automation** ("Allow apps to access and control other apps") | macOS 10.14+ | [Apple: NSAppleEventsUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsappleeventsusagedescription); [Apple Support: Privacy & Security settings (macOS 27)](https://support.apple.com/guide/mac-help/mchl211c911f/mac) | read 2026-10-24 | High |
| 12.3 | **Hardened-runtime senders need `com.apple.security.automation.apple-events`**, except for events to themselves or to "other processes signed with the same team ID" | macOS 10.7+ | [Apple: Apple Events Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.automation.apple-events) | read 2026-10-24 | High |
| 12.4 | **Safari gates script-into-page behind its own switch**: Developer settings > Automation > "Allow JavaScript from Apple Events" (also "Allow remote automation" for WebDriver). An Apple DTS answer (macOS 26.5.1) says the script fails while it is off and the calling app also needs Automation permission | 2 gates (TCC + browser switch) | [Apple: Safari developer settings](https://developer.apple.com/documentation/safari-developer-tools/developer-settings); [Apple Developer Forums 836299](https://developer.apple.com/forums/thread/836299) | read 2026-10-24 | High (switch); Medium (DTS answer) |
| 12.5 | **WebDriver enablement requires the user's password**: `safaridriver --enable` turns on remote automation "The user must authenticate via password" | 1 auth | `safaridriver(1)` man page, macOS 27.0.1 | read 2026-10-24 | High |
| 12.6 | **App Intents** expose actions and entities to Shortcuts, Siri, Spotlight and Apple Intelligence | macOS 13.0+ | [Apple: App Intents](https://developer.apple.com/documentation/appintents) | read 2026-10-24 | High |
| 12.7 | **macOS 27: apps can no longer read the local TCC database directly** | 27.0 | [Apple: macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) (TCC) | 2026-09 | High |

**Implications:** TCC protects the *target* only from unknown senders; once a user allows "App X → Izcal", X can do whatever Izcal's dictionary offers. So Izcal's dictionary must be split: windows/tabs/URLs (safe, on by default) and "run JavaScript in a page" (off by default, a per-profile user switch like Safari's, never available in private windows). App Intents are the preferred modern surface; both run in the broker.

---

## 13. Credentials and speech

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 13.1 | **Use SecItem with the data protection keychain.** macOS has two implementations; SecItem defaults to the file-based one; set `kSecUseDataProtectionKeychain`. The file-based keychain is "on the road to deprecation" | macOS 10.15+ (key) | [Apple: TN3137 On Mac keychain APIs](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains); [kSecUseDataProtectionKeychain](https://developer.apple.com/documentation/security/ksecusedataprotectionkeychain) | read 2026-10-24 | High |
| 13.2 | **Data protection keychain needs a user context and provisioned entitlements**: access groups come from code-signing entitlements that "must be authorized by a provisioning profile", which needs an app-like bundle; library code inherits the host executable's entitlements | — | Same as 13.1 | read 2026-10-24 | High |
| 13.3 | **`NSSpeechSynthesizer` is deprecated (macOS 14.0)**; `AVSpeechSynthesizer` (macOS 10.14+) is its replacement; Personal Voice needs `requestPersonalVoiceAuthorization` (macOS 14.0+) | 1 prompt (Personal Voice) | [Apple: NSSpeechSynthesizer](https://developer.apple.com/documentation/appkit/nsspeechsynthesizer); [AVSpeechSynthesizer](https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer); [requestPersonalVoiceAuthorization](https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer/requestpersonalvoiceauthorization(completionhandler:)) | read 2026-10-24 | High |
| 13.4 | **Where synthesis runs** (in-process or a system service) and whether a sandboxed helper can use it are not documented | — | Gap | — | Gap |

**Implications:** Password-store keys live in the data protection keychain under one access group owned by Izcal's signed bundle; only the broker (or a storage service built as a bundled, provisioned helper) touches it. Speech synthesis for `speechSynthesis` runs in a utility service; whether that service can be sandboxed is a measurement item.

---

## 14. Clipboard and drag and drop

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 14.1 | **Pasteboard privacy API exists since macOS 15.4**: `NSPasteboard.AccessBehavior` = `default` / `ask` / `alwaysAllow` / `alwaysDeny`. For the general pasteboard the default is to ask on programmatic access; "access that is both user originated and paste related will always be allowed, and will not result in a notification"; after the first alert the app appears in System Settings | macOS 15.4+ | [Apple: NSPasteboard.AccessBehavior](https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum) | read 2026-10-24 | High |
| 14.2 | **Detect-without-reading APIs**: `detectPatterns(for:)`, `detectMetadata(for:)` determine what the first item is "without notifying the person"; `detectValues` reads matched contents | macOS 15.4+ | [Apple: NSPasteboard](https://developer.apple.com/documentation/appkit/nspasteboard) | read 2026-10-24 | High |
| 14.3 | **Enforcement has not shipped by default.** Apple's 2025 notice described an alert when an app "programmatically reads the general pasteboard" and a `EnablePasteboardPrivacyDeveloperPreview` default; it was absent in macOS 26 unless enabled; a September 2026 comment cites 27.2 beta release notes saying the default is deprecated and no longer works. The macOS 27 Privacy & Security guide has no paste pane | preview since 15.4 | [Michael Tsai: Pasteboard Privacy Preview in macOS 15.4](https://mjtsai.com/blog/2025/05/12/pasteboard-privacy-preview-in-macos-15-4); [Apple Support: Privacy & Security (macOS 27)](https://support.apple.com/guide/mac-help/mchl211c911f/mac) | 2025-05-12 … 2026-09-21 | Medium |
| 14.4 | **File promises** (`NSFilePromiseProvider`, a UTI conforming to data or directory, delegate writes the file) carry virtual files out by drag; macOS 27 asserts that `receivePromisedFiles…` is only called during a drag operation | macOS 10.12+; 27.0 change | [Apple: NSFilePromiseProvider](https://developer.apple.com/documentation/appkit/nsfilepromiseprovider); [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) (AppKit) | read 2026-10-24 | High |
| 14.5 | **Password convention**: `org.nspasteboard.ConcealedType` marks confidential content (obfuscate, do not record); `TransientType` (momentary) and `AutoGeneratedType` (not user-copied) are siblings. This is a community convention, not an Apple API | 3 marker types | [nspasteboard.org](http://nspasteboard.org/) | read 2026-10-24 | High (convention text) |

**Implications:** All pasteboard reads happen in the broker in response to a real paste (menu, Cmd-V), which is the exempt path; `navigator.clipboard.readText()` without a gesture must use the detect APIs or ask, so Izcal never causes an OS alert. Copies of passwords from Izcal's password manager add `ConcealedType` and `TransientType`.

---

## 15. Download quarantine and Gatekeeper

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 15.1 | **`LSFileQuarantineEnabled`** makes every file the app creates quarantined by default; otherwise the app sets `com.apple.quarantine` (via the URL resource key `quarantinePropertiesKey`) on downloads with the origin URL | macOS 10.0+ | [Apple: LSFileQuarantineEnabled](https://developer.apple.com/documentation/bundleresources/information-property-list/lsfilequarantineenabled) | read 2026-10-24 | High (key); Medium (resource-key route) |
| 15.2 | **macOS 15 removed the Control-click Gatekeeper override**; users must go to System Settings > Privacy & Security to open software that is not signed and notarized | macOS 15 | [Apple Developer News: Updates to runtime protection in macOS Sequoia](https://developer.apple.com/news/?id=saqachfa) | 2024-08-06 | High |
| 15.3 | **macOS 27 adds `spctl --purge`** to clear Gatekeeper records for unmounted apps | 27.0 | [Apple: macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) (Gatekeeper) | 2026-09 | High |

**Implications:** The download service writes the quarantine attribute itself (with origin URL and Izcal's bundle as agent) before the file is visible; `LSFileQuarantineEnabled` in the broker would also quarantine profile files and is wrong for a browser. Which process may set the attribute under Izcal's sandbox profile is a measurement item.

---

## 16. Process placement, sandbox and runtime

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 16.1 | **Notarization requires the Hardened Runtime**; entitlements go on executables only, and frameworks/plug-ins inherit the host's | — | [Apple: Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime) | read 2026-10-24 | High |
| 16.2 | **JIT needs `com.apple.security.cs.allow-jit`** for `MAP_JIT` memory; without it JIT code "may fall back to an interpreter … or crash". `allow-unsigned-executable-memory` is the broader, discouraged exception | macOS 10.7+ | [Apple: allow-jit](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-jit); [allow-unsigned-executable-memory](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-unsigned-executable-memory) | read 2026-10-24 | High |
| 16.3 | **App Sandbox is mandatory only for the Mac App Store** ("To distribute a macOS app through the Mac App Store, you must enable the App Sandbox capability") | — | [Apple: App Sandbox](https://developer.apple.com/documentation/security/app-sandbox) | read 2026-10-24 | High |
| 16.4 | **XPC services are launchd-started bundles inside the app**, addressed by bundle identifier | — | [Apple: Creating XPC services](https://developer.apple.com/documentation/xpc/creating-xpc-services) | read 2026-10-24 | High |
| 16.5 | **Main-process-only by construction**: everything delivered through `NSApplication` and its delegate (URL/file opens, Dock menu, Dock tile, activation, Services, scripting, window backing changes), `NSWindow`/`NSStatusItem`, and the bundle-identity features (default handler, notification authorization, now-playing, login item `mainApp`). Process-wide and unprivileged: IOPM assertions, `ProcessInfo` power/thermal state | — | Inference from rows 2–12 (all these APIs are `NSApplication` delegate methods or bundle-scoped) | — | Inference |
| 16.6 | **Supported OS window**: macOS 26 Tahoe is the last release for Intel Macs; Rosetta app support ends after macOS 27; macOS 27 lists Intel apps that will stop working in macOS 28 | 26 = last Intel; 27 = last Rosetta | [Apple: macOS 26.4 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-26_4-release-notes) (Rosetta); [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | 2026-03; 2026-09 | High |

**Implications:** The broker is the one `NSApplication`: it owns every row in sections 1–6, 8–12 and 14, and is the only process with Apple-event handlers and user-visible bundle identity. Services may hold power assertions and read power state themselves. Only the JS-running processes (renderers, and any worker host) get `allow-jit`; the broker, network and storage services do not. Izcal can target arm64 only for macOS 27+, with an x86_64 build only if macOS 26 on Intel is a supported floor.

---

## 17. Open questions for black-box measurement

1. Error domain and code returned by `setDefaultApplication(at:toOpenURLsWithScheme:)` when the user clicks "Keep", and whether the completion is ever called if the dialog is ignored.
2. When Launch Services registers a freshly copied Izcal.app (before first launch? only after?), so it appears in Desktop & Dock > Default web browser.
3. Whether an `NSOpenPanel` can be shown by a windowless, sandboxed helper process (expected: no; the broker shows it).
4. Media-key arbitration between Izcal and another player (Music, Safari) after each starts and pauses playback.
5. Whether `AVSpeechSynthesizer` works inside a process running Izcal's sandbox profile.
6. Whether the download service can set `com.apple.quarantine` under its sandbox profile, and what Gatekeeper shows for the origin.
7. Whether pasteboard alerts appear in macOS 27.x for a gesture-less read, with and without `EnablePasteboardPrivacyDeveloperPreview`.

