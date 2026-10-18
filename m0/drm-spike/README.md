# DRM-1 / DRM-2 spike: protected media mode host

This is a minimal Rust host built on `wry` 0.57 and `tao` 0.37. It opens streaming sites in the operating system's own licensed web engine, unmodified:

- **Windows:** WebView2, the Edge engine. It provides Widevine and PlayReady under Microsoft's license.
- **macOS:** WKWebView, the system WebKit. It provides FairPlay under Apple's license.

Its purpose is to test whether DRM playback works through **protected media mode**. The plan is at `~/.claude/plans/cheerful-spinning-reef.md`, section 1 and work items 1–2.

Ground rules (no-circumvention rule, design section 13.1):

- **No spoofing.** The user agent is the engine's own. The only option is the honest macOS application-name suffix `HGBA-ProtectedMedia/0.1`, which is appended to WebKit's UA. Nothing impersonates Chrome or Safari.
- **No tampering.** Nothing touches the CDM or handles keys. There are no extra engine flags.
- **No credentials.** You sign in manually in the window. The program never sees or stores credentials. Cookies and other site data live only in the container's data store.
- **Rejections are recorded, not worked around.** If a service rejects the embedded webview, record it. It goes on the licensing and allow-list outreach list.

## What it does

| Piece | Windows (WebView2) | macOS (WKWebView) |
|---|---|---|
| Per-container storage | WebView2 user-data folder `profiles/<container>/` (via `WebContext`) | `WKWebsiteDataStore(forIdentifier:)`, using a UUID derived from the container name (macOS 14+). Data lives under `~/Library/WebKit/drm-spike/WebsiteDataStore/<uuid>`. |
| UA mode `appname` | not implemented (WebView2 has no application-name API); prints a note and uses the default | `WKWebViewConfiguration.applicationNameForUserAgent = "HGBA-ProtectedMedia/0.1"` |
| Content blocking | `WebResourceRequested` with filter `*`. Blocked requests get a 403, and each one is **counted and logged** | `WKContentRuleList`, compiled before the first load. WebKit has **no per-request callback**, so blocked requests can't be counted. The probe page's self-test shows that blocking works. |
| Observer script | injected into every frame | injected into every frame |

**Observer script** (`assets/observer.js`). It observes only and doesn't change behaviour; the header comment lists the one documented side effect. It records:

- `requestMediaKeySystemAccess` calls, with the requested and granted configuration, including robustness;
- `MediaKeys` creation and attachment;
- `encrypted` events;
- license requests (`message` events) and key statuses;
- every 2 s, the largest playing `<video>`'s `videoWidth x videoHeight`, frame counts, and whether it is encrypted.

**Results.** The host appends JSON lines to `results/<os>-<container>.jsonl`. Each line has `ts`, `os`, `container`, `ua_mode` and the sender's `origin`. Origins only are logged, never full URLs.

**Auto-trigger check.** For each top-level origin, the host logs a `trigger_evaluation` record the first time one of these happens: an `encrypted` event, `MediaKeys` creation, or a license request. Bare `requestMediaKeySystemAccess` probes never produce one, and an event inside a cross-origin iframe is logged as "cannot switch top-level page".

**Bundled pages.** The bundled home and probe pages are served from a custom protocol: `hgba://localhost/` on macOS and `https://hgba.localhost/` on Windows. Both are secure contexts, which EME requires.

**Popups.** Sign-in popups open in an engine-default window that shares the container's storage. They don't get the observer script, and on Windows they don't get the blocking hook.

## Command line

```
drm-spike --container <name> [--url <url>|home|probe] [--ua-mode default|appname]
          [--results-dir <dir>] [--exit-after <secs>]
```

`<name>` is `[A-Za-z0-9_-]{1,64}`. With no `--url`, the bundled home page opens. It has buttons for Netflix, Disney+, Prime Video, Spotify and Max, and for the probe-only page.

## Running the tests

### macOS (this Mac)

```sh
cd ~/Documents/Cline/code/browser/m0/drm-spike
cargo build --release
./target/release/drm-spike --container A                      # default UA
./target/release/drm-spike --container A --ua-mode appname    # UA + "HGBA-ProtectedMedia/0.1"
./target/release/drm-spike --container B                      # isolation check
./target/release/drm-spike --container A --url probe          # probe-only page
```

### Windows laptop (sit at the laptop)

Open a Command Prompt on the laptop's desktop:

```bat
cd /d C:\hgba-m0\drm-spike
target\release\drm-spike.exe --container A
target\release\drm-spike.exe --container B
target\release\drm-spike.exe --container A --url probe
```

To rebuild: `cargo build --release` in the same folder.

To launch it on the laptop's desktop from the Mac (ssh sessions are non-interactive):

```sh
ssh joe@192.168.50.253 'schtasks /create /tn drm_spike /tr "C:\hgba-m0\drm-spike\target\release\drm-spike.exe --container A" /sc once /st 00:00 /it /f'
ssh joe@192.168.50.253 'schtasks /run /tn drm_spike'
# when finished, close the window, then:
ssh joe@192.168.50.253 'schtasks /delete /tn drm_spike /f'
```

To copy the results back to the Mac:

```sh
scp 'joe@192.168.50.253:C:/hgba-m0/drm-spike/results/*.jsonl' ~/Documents/Cline/code/browser/m0/drm-spike/results/
```

## What to do in each session

1. Launch with `--container A`. On the home page, click a service and **sign in yourself**.
2. Play any title for at least 60 seconds. Use full screen if you like.
3. Open the service's stats overlay and note the resolution and anything it says about DRM or HDCP:
   - **Netflix:** `Ctrl+Shift+Alt+D` on Windows, `Ctrl+Shift+Option+D` on Mac. Shows playing bitrate, resolution and the DRM type.
   - **Prime Video:** the gear or settings menu shows the quality tier.
   - **Max:** the settings menu shows quality. There's no stats overlay.
   - **Disney+:** there's no public overlay. Rely on the decoded size in the JSONL, plus the quality settings.
   - **Spotify:** audio only. Note whether playback starts and keeps playing for more than 30 seconds.
4. If the site blocks the browser, write down the exact message ("unsupported browser", an error code, a blank player). **Don't work around it.**
5. Close the window. The session is saved in `results/<os>-A.jsonl`.
6. Repeat on macOS with `--ua-mode appname`. If one UA mode works and the other doesn't, note that.

### Container-isolation check

1. Sign in to the service under `--container A` and confirm you're signed in. Close the window.
2. Launch `--container B` and open the same service. **It must show you as signed out.** If it does, isolation held.
3. Optionally, sign in under B with a different profile or account and confirm A is unchanged.
4. Record "isolation held? yes/no" for that service and OS in `results/manual.csv` (see below).

### Auto-trigger check (probe-only page)

Run `--url probe`. The page lists which key systems the engine reports, and it never creates `MediaKeys`. Afterwards, the JSONL must contain **no** `trigger_evaluation` record for the probe origin. The summarizer reports this as "auto-trigger fired on probe page (must be no)". A real playback session should produce `trigger_evaluation` with `"would_switch": true`.

## Summarising results

```sh
cargo run --release --bin summarize                 # reads ./results/*.jsonl
cargo run --release --bin summarize -- /path/to/results
```

It prints the tables below as Markdown. It fills these columns automatically:

- **Plays?**: whether encrypted video was decoded, or keys became usable;
- **Max resolution**: the largest decoded `videoWidth x videoHeight` seen while playing;
- **Key system**.

You can add facts you observed to `results/manual.csv`. A non-empty field there overrides the automatic column:

```
# service,os,ua_mode,plays,max_resolution,isolation_held,notes
Netflix,windows,default,yes,1080p (overlay),yes,overlay shows PlayReady SL3000
Netflix,macos,appname,,,yes,
```

The `service` must match the summarizer's names: `Netflix`, `Disney+`, `Prime Video`, `Spotify`, `Max`. The `os` is `windows` or `macos`.

### Results table template

| Service | OS | Plays? | Max resolution | Key system | Isolation held? | Notes |
|---|---|---|---|---|---|---|
| Netflix | Windows (WebView2) | | | | | |
| Netflix | macOS (WKWebView) | | | | | |
| Disney+ | Windows | | | | | |
| Disney+ | macOS | | | | | |
| Prime Video | Windows | | | | | |
| Prime Video | macOS | | | | | |
| Spotify | Windows | | | | | |
| Spotify | macOS | | | | | |
| Max | Windows | | | | | |
| Max | macOS | | | | | |

**Pass criterion (plan, Verification):** Netflix and at least one other service play on each of Windows and macOS, with no UA spoofing beyond the engine's own identity.

## Known limitations

- **Per-container storage on macOS** uses `WKWebsiteDataStore(forIdentifier:)`, which needs macOS 14 or later; on macOS 13 and older, wry would silently fall back to the shared default store. The host builds its own `WKWebViewConfiguration` so that it can set the application-name UA and element full screen. It therefore sets the store itself, because wry uses the configuration's store when a configuration is supplied.
- **Request interception:** wry 0.57 only exposes custom-protocol interception through its own API. The spike reaches the native objects through wry's public escape hatches instead:
  - **Windows:** `WebViewExtWindows::webview()` and `environment()`, then `AddWebResourceRequestedFilter("*")` and `add_WebResourceRequested`. This is WebView2's documented API.
  - **macOS:** `WebViewExtMacOS::manager()`, then `WKContentRuleList`.
  - WKWebView can't intercept remote http(s) requests per request; `WKURLSchemeHandler` can't be registered for http or https. So on macOS, blocking is declarative-only and uncounted.
- **`--ua-mode appname` on Windows** isn't implemented. The plan names it for macOS only.
- **Popups** don't get the observer or the Windows blocking hook (see above).
