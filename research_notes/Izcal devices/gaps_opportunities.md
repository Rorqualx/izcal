# Izcal devices: gaps and opportunities

Compiled 21 October 2026. Primary sources were preferred (specifications, standards-body status pages, maintainers' posts, vendor documentation, issue trackers).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

Scope: user and developer gaps, engine support, Izcal's opportunities and unowned surfaces; other strands own history, semantics, security, OS interfaces, fingerprinting and recording. Products are evidence, not dependencies; ESP Web Tools and Hardless are Apache-2.0 and KlipperFleet GPL-3.0, evidence only.

## 1. What users cannot do today

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | **Firmware flashing and consoles** run over Web Serial in ESP Web Tools, Arduino Create, Betaflight Configurator, Espruino Web IDE and MakeCode; ESP lists Firefox, Chrome and Edge, Android is unimplemented, and Serial launched on Chrome 89 desktop only | 5 tools; 3 browsers; 1 platform unimplemented | [ESP Web Tools](https://esphome.github.io/esp-web-tools/); [Chrome, Web Serial](https://developer.chrome.com/docs/capabilities/serial) | 2020-08-12 | High |
| 1.2 | **3D printers**: a FLY tool flashes controller boards over Web Serial and needs Chrome or Edge; KlipperFleet is an alpha web UI for Klipper/Kalico firmware | 1 vendor tool; 1 alpha project | [FLY tool](https://www.klipper.cn/en/docs/ToolsDoc/fly-ams-tool/); [KlipperFleet, GPL-3.0](https://github.com/JohnBaumb/KlipperFleet) | 2026-01-02 | Medium |
| 1.3 | **Hardware wallets**: Trezor Suite for web is USB-only over WebUSB, works in Chrome, Edge, Brave and Opera, needs Trezor Bridge on Firefox, and has no iOS path | 4 Chromium browsers; 2 engines excluded | [Trezor](https://trezor.io/guides/trezor-suite/connecting-to-trezor-suite-for-web) | 2026-10-21 (accessed) | High |
| 1.4 | **Keyboard configuration**: VIA configures QMK keyboards over WebHID and is Chromium-desktop only; Firefox, Safari and mobile are unsupported, so a desktop wrapper exists | 5 browsers; 0 mobile | [Pulsar](https://support.pulsar.gg/hc/en-us/articles/58539951860761-What-web-browsers-support-VIA) | 2026-10-21 (accessed) | Medium |
| 1.5 | **MIDI editors**: Novation Components runs on Web MIDI; Safari and iOS are unsupported, and Firefox rejects `requestMIDIAccess()` without a site-specific add-on, so an app exists | 1 add-on gate; 2 platforms excluded | [Novation](https://support.novationmusic.com/hc/de/articles/208258585-My-web-browser-is-not-compatible-with-Novation-Components); [Firefox 108](https://www.firefox.com/en-US/firefox/108.0/releasenotes/) | 2022-12-13 | High |
| 1.6 | **Bluetooth wearables and medical devices**: Web Bluetooth is Chromium desktop and Android only; all iOS browsers are WebKit, so users install Bluefy or WebBLE | 2 workaround browsers; 0 native iOS | [Chrome, Web Bluetooth](https://developer.chrome.com/docs/capabilities/bluetooth); [Bluefy](https://apps.apple.com/us/app/bluefy-web-ble-browser/id1492822055); [Pybricks](https://github.com/pybricks/support/issues/2074) | 2020-01 | High |
| 1.7 | **NFC tags**: Web NFC is Android Chrome only and NDEF-only, with no low-level I/O, peer-to-peer or host-card emulation; tasks are museum cards, inventory, badges and provisioning | 1 platform; 3 excluded modes | [Chrome, Web NFC](https://developer.chrome.com/docs/capabilities/nfc) | 2020-02-12 | High |
| 1.8 | **Manufacturing at scale** fell back on heuristics: a tool for 2,000 wearable units could not tell identical units apart because Serial exposes vendor and product IDs, not the serial number; it probes with WebUSB and Serial | 2,000 units; 2 APIs; 1 identifier missing | [HackerNoon](https://hackernoon.com/how-i-fixed-manufacturing-for-ai-robotics-with-webserial) | 2026-02-11 | Medium |

**Implications:** Every listed task has a shipped browser tool, nearly all confined to Chromium or one platform with a native bridge or another browser as the fallback. Izcal can make the existing tasks work everywhere under one consent model.

## 2. The engine-support gap

Global-usage figures are Can I Use's share of users on a supporting browser, not API adoption.

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | **Chromium-only trio**: WebUSB Chrome 61+ and Android (76.47%); WebHID Chrome 89+, desktop only (30.2%); Web Bluetooth Chrome 56+ and Android (76.52%); Edge matches Chrome, Firefox and Safari never | 3 APIs; 30.2–76.52% global | [WebUSB](https://caniuse.com/webusb); [WebHID](https://caniuse.com/webhid); [Web Bluetooth](https://caniuse.com/web-bluetooth) | 2026-09 data | Medium |
| 2.2 | **Web Serial**: Chrome 89–158 and Edge 89–155 desktop; Firefox 151+ desktop from 19 May 2026, add-on-gated per site and port and off by default in enterprise policy; Firefox Android and Safari never | 74.58% global; 2 engines gated | [Can I Use](https://caniuse.com/web-serial); [Firefox 151](https://www.firefox.com/en-US/firefox/151.0/releasenotes/) | 2026-05-19 | High |
| 2.3 | **Web MIDI**: Chrome 43–158, Edge 79–155 and Chrome Android; Firefox 108–161 from 13 December 2022, add-on-gated; Safari never | 78.96% global; 3 engines | [Can I Use](https://caniuse.com/midi); [Firefox 108](https://www.firefox.com/en-US/firefox/108.0/releasenotes/) | 2022-12-13 | High |
| 2.4 | **Web NFC**: Chrome for Android 89+ only; desktop Chrome, Edge, Firefox and Safari never | 42.79% global; 1 platform | [Can I Use](https://caniuse.com/webnfc) | 2026-09 data | Medium |
| 2.5 | **Vendor positions and net shape**: Apple declined all six in 2020 (16 APIs); Mozilla recorded WebUSB and WebHID harmful and Serial neutral before shipping it gated. Net: 3 Chromium-only, 1 Android-only, 1 cross-engine without Safari, 1 add-on-gated; 0 in Safari | 6 APIs; 0 Safari | [WebKit](https://webkit.org/tracking-prevention/); [InfoQ](https://www.infoq.com/news/2020/07/apple-fingerprinting-privacy/); [WebHID issue](https://github.com/mozilla/standards-positions/issues/459) | 2020-07-04; 2021-08-26 | High |

**Implications:** Both non-Chromium engines that ship a device API gate it behind an add-on, and Safari ships none; sites target Chromium or ship a native fallback. Izcal can be the first engine to implement all six behind one auditable consent model.

## 3. The developer-experience gap

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | **Virtual devices are absent or test-only**: Chrome has `about://usb-internals` connect/disconnect simulation and `about://device-log`, but no scriptable Serial/HID/MIDI/NFC emulator; Bluetooth's fake adapter is not in stable builds, and Serial's tests need real hardware | 0 of 6 scriptable; 0 fakes in stable | [WebUSB](https://developer.chrome.com/docs/capabilities/usb); [WPT Bluetooth](https://raw.githubusercontent.com/web-platform-tests/wpt/master/bluetooth/README.md); [WPT Serial](https://raw.githubusercontent.com/web-platform-tests/wpt/master/serial/README.md) (README prose; no code opened) | 2026-10-21 (accessed) | Medium |
| 3.2 | **CI and mocks**: BrowserStack documents mobile tunnelling for Appium, not USB/HID/Serial passthrough; teams self-host hardware or build emulators such as Hardless, which simulates four POS classes with fault injection | 0 hosted options; 4 classes | [BrowserStack](https://www.browserstack.com/docs/custom-device-lab/app-automate/appium/device-tunneling/getting-started); [PlatformIO](https://docs.platformio.org/en/latest/plus/pio-remote.html); [Hardless, Apache-2.0](https://github.com/hardlessdev/hardless) | 2026-10-18 | Medium |
| 3.3 | **Reconnects are flaky**: GATT attributes go invalid after disconnect and must be re-fetched; parallel GATT operations may fail and must be queued; a fatal serial error nulls `port.readable` | 3 failure modes | [Bluetooth](https://developer.chrome.com/docs/capabilities/bluetooth); [Serial](https://developer.chrome.com/docs/capabilities/serial) | 2026-10-21 (accessed) | High |
| 3.4 | **Permission friction**: Firefox installs a site-permission add-on before the port picker for Serial and MIDI; `DefaultSerialGuardSetting` disables Serial in enterprise Firefox; Chrome needs `forget()` | 2 gates; 1 enterprise default | [The Register](https://www.theregister.com/2026/04/14/firefox_nightly_web_serial/); [WebUSB](https://developer.chrome.com/docs/capabilities/usb) | 2026-04-14 | Medium |
| 3.5 | **OS driver setup differs**: Windows needs WinUSB, and libusbK/libusb0 devices stay invisible until rebound (Zadig); Linux needs udev rules and `plugdev`; Android Serial needs an unclaimed device | 3 OS paths | [WebUSB](https://developer.chrome.com/docs/capabilities/usb); [ODrive](https://docs.odriverobotics.com/v/latest/interfaces/gui.html#frequent-issues) | 2026-10-21 (accessed) | High |
| 3.6 | **No device inspector or replay**: DevTools has no USB, HID, Serial, MIDI, Bluetooth or NFC domain; visibility is `about://device-log`, with no captured stream to replay | 0 domains; 0 replay formats | [WebUSB](https://developer.chrome.com/docs/capabilities/usb); [WebHID](https://developer.chrome.com/docs/capabilities/hid) | 2016-03-30; 2020-09-15 | High |

**Implications:** Device code is tested by hand, on hardware, in one browser, and the only fakes are engine test adapters absent from stable builds. Izcal's deterministic substrate makes virtual devices and recorded sessions first-class facilities.

## 4. What Izcal's architecture makes possible

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | **Broker-owned device permission ledger**: each grant is a broker record (site, container, device, class, interface or usage, purpose, expiry); the renderer holds a typed handle, never the grant | 1 record per grant; 7 fields | [Security scope][SEC] broker store; CON-R1 | 2026-10-21 | High |
| 4.2 | **Per-site, per-container, per-device grants**: no class-wide MIDI grant; revocation is destructive and durable; disuse expiry follows the permission store | 3 scopes; 0 class-wide | [Extensions](../../reports/Izcal%20extensions.md) EX-D5; [shell](../../reports/Izcal%20shell.md) "Revoke" | 2026-10-21 | High |
| 4.3 | **Transparency in both places**: the ledger answers "which sites used which devices, when" with filters and a use count; the site panel gets a device row with broker-formatted names | 4 filters; 1 site row | [Shell](../../reports/Izcal%20shell.md) 6.4 and 6.6, SH-D11; CON-R4 | 2026-10-21 | High |
| 4.4 | **No ambient enumeration**: USB, HID, Serial and Bluetooth return only granted devices; Web MIDI lists every port. Per-device grants keep `getDevices` exact | 4 of 5 permission-scoped; 1 ambient | [USB](https://developer.mozilla.org/en-US/docs/Web/API/USB/getDevices), [HID](https://developer.mozilla.org/en-US/docs/Web/API/HID/getDevices), [Serial](https://developer.mozilla.org/en-US/docs/Web/API/Serial/getPorts), [Bluetooth](https://developer.mozilla.org/en-US/docs/Web/API/Bluetooth/getDevices), [MIDI](https://developer.mozilla.org/en-US/docs/Web/API/MIDIAccess) | 2026-10-21 (accessed) | High |
| 4.5 | **Recorded sessions and deterministic replay**: device bytes become B1 substrate inputs on the recorded clock, so a bug report replays, steps, diffs and bisects | 1 recorded stream; 1 clock | [Capabilities](../../.claude/skills/first-principles-spec/references/izcal-capabilities.md) B1/DET-1 | 2026-10-17 | High |
| 4.6 | **Virtual-device oracle**: an engine-hosted device chosen through the same chooser, labelled virtual, scriptable and fault-injecting; headless CI needs no hardware | 0 hardware needed | [DevTools](../../reports/Izcal%20DevTools%20spec.md) C-19 replay host; B1/B2 | 2026-10-21 | High |
| 4.7 | **One device-access process kind**: all six behind one sandboxed service, with no OS device calls in renderers, typed B2 channels and one threat model | 36 process kinds and 38 threat models today; 0 device services | [Security scope][SEC] 6.3/6.5 | 2026-10-21 | High |
| 4.8 | **Protocol-owned devices stay owned**: FIDO/CTAP stays in the credential subsystem, never raw HID; agents need an explicit device capability and cannot inherit a user grant | 0 raw FIDO paths; 1 agent gate | [WebHID](https://developer.chrome.com/docs/capabilities/hid); B9/B10, AGT-R | 2020-09-15 | High |

**Implications:** Existing broker, ledger, replay and process machinery covers devices without a new trust model: a grant is a record, a device is a typed channel, a session is a recording. The design work is one process shape and one grant shape, not six.

## 5. The unowned-surface gap

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | **Unowned and unrecorded**: DOM 6.18's 11 web API rows omit all six despite its rule that an unlisted API "is recorded in the ledger as a gap before it is admitted"; the Phase 3 ledger's 25 gap rows name no device API, service or permission | 11 API rows; 25 gap rows; 0 device | [DOM spec](../../reports/Izcal%20DOM,%20bindings%20and%20event%20loop.md) 6.18; [ledger](../Phase%203%20amendments/ledger.md) | 2026-10-21 | High |
| 5.2 | **The security scope** 3.5 gap table has 23 rows, none device; its process map has 36 kinds and 38 threat models, with device authority only in the capture, audio and hardware-decode exceptions | 23 rows; 36 kinds; 38 models; 0 device | [Security scope][SEC] 3.5, 6.3, 6.5 | 2026-10-21 | High |
| 5.3 | **Extensions Ring C** names `captivePortal`, `power`, `fontSettings` and `input.ime` as "OS and vendor surfaces without an Izcal owner yet", but names none of the six device APIs | 4 surfaces named; 0 of 6 | [Extensions](../../reports/Izcal%20extensions.md) Ring C | 2026-10-21 | High |
| 5.4 | **Transparency and tooling are unpopulated**: shell 6.4 records prompts with a use count and History link and 6.6 is the site panel, but no device grant class exists; DevTools has no device domain and no crate table names a device crate | 0 grant classes; 0 domains; 0 crates | [Shell](../../reports/Izcal%20shell.md); [DevTools](../../reports/Izcal%20DevTools%20spec.md) | 2026-10-21 | High |

**Implications:** Section 9.5 can name a precise gap: six script-facing APIs, their permissions, process, records and threat model have no owner, and the rule meant to record them leads to a ledger with no row for them.

## 6. Questions this strand could not answer

- Does Chrome for Android 155 ship Web Serial natively, or does Can I Use's "supported" reflect the WebUSB polyfill?
- Does Firefox 151's Serial gate bind per site, per port, or both, and what consent copy ships?
- How many users install Bluefy, WebBLE or a native bridge, and how many production sites call each API? No vendor publishes numbers, and no public crawl probes device APIs.
- Can Android Chrome's `neverForLocation` filtering be avoided for iBeacon/Eddystone, or is that class permanently invisible?
- Is there an MIT corpus of USB/HID descriptors, GATT profiles or NDEF records for a virtual-device oracle, or must Izcal build its own?
- Can one device-access process meet isochronous USB audio and low-latency MIDI timing, or does each class need its own process shape?
[SEC]: ../../../security-scope/reports/Izcal security scope.md
