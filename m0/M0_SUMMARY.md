# M0 summary (14 October 2026)

Combined results of milestone M0 on macOS arm64 (this Mac), Linux x86-64 (huey) and Windows x86-64 (laptop). Per-platform details: `M0_RESULTS.md` (macOS), `linux/M0_LINUX_RESULTS.md`, `windows/M0_WINDOWS_RESULTS.md`, `determinism_crossplatform.md`, `drm-spike/README.md`.

## Exit criteria (Stage 1 design, Section 8.1)

| Criterion | Status |
|---|---|
| Fork of `rusty_v8` with the `cppgc` APIs and the predictable platform | **Done, locally.** Branch `hgba` on each host: v152.2.0 + `cppgc` patch + predictable platform (+ `disarm` submodule fix on macOS). Not yet pushed to a shared remote |
| HGBA-2.1 start condition (i): spike items 1–5 under the sandbox on three platforms | **Met**, in release and debug V8 |
| (ii) exposure route recorded | **Met**: the fork is the plan of record (HGBA-2.1.1); `v8-cppgc-ext` is retired; the APIs are compiled only inside the fork (HGBA-2.1.6 S-4e) |
| (iii) debug-V8 CI configuration live | **Partly.** Debug V8 builds and catches an omitted barrier on all three platforms; the CI workflow (`rusty_v8/.github/workflows/izcal-v8-build.yml`, 17 October 2026) uses the recipes that worked by hand on all three hosts, but has not run on GitHub, because the fork has no remote yet. Its Windows release and debug jobs passed end to end on the laptop through a local runner on 17 October 2026 (`ci/README.md`) |
| V8 build farm producing sandbox archives for three platforms | **Partly.** Archives built by hand on each host (gzipped release: 42 MB macOS, 41 MB Linux; Windows `.lib` 239 MB); no farm or mirror yet |
| `rusty_v8` owner named | **Open** |
| Team for M1–M2 staffed | **Blocked** on D-S8 (funding) |
| DRM spikes DRM-1 and DRM-2 | **Built; probe results only.** Signed-in playback tests deferred by the owner |

## Results

**Spike (sandbox on, all three platforms, release and debug).** Identical outcomes to the sandbox-off original:
- **Item 1:** the no-GC scope does not stop V8-initiated GC. Under the blocked-owner guard: 0 finalizations (macOS 15,006 phases, Linux 7,380, Windows 8,891 in release).
- **Item 2:** 0 errors on all three platforms.
- **Items 3 and 4:** pass.
- **Item 5:** debug catches the omitted barrier and release misses it.
- **`cppgc` APIs:** compiled natively in the fork; companion tests pass (macOS 28, Linux 25, Windows 30). The counts differ only in which suites were run: all three ran the same 23 functional tests; macOS adds the 5 `abi` tests (the patch's 2 `upstream_api` tests ran separately in `fork-api-verify/`), Linux adds the 2 `upstream_api` tests, and Windows adds both.

**Determinism (HGBA-3.1 Q-3, answered).**
- Under the fork's predictable platform: 1,000 of 1,000 identical hashes in-process and 50 of 50 across fresh processes, in release and debug.
- **Hashes are identical across macOS, Linux and Windows.**
- With V8's stock platform: 848, 912 and 989 distinct hashes in 1,000 runs without flags, and occasional divergence even with `--predictable --single-threaded`.
- The worker-task mode changes the hash in release builds, so recordings carry it as a mandatory field.

**DRM probe (system web engines).**
- WebView2 (Windows): Widevine (software level), PlayReady including the hardware level, and Clear Key.
- WKWebView (macOS): FairPlay and Clear Key.
- Per-container storage and blocking work on both.

## Findings carried into the specs

| Finding | Where it landed |
|---|---|
| The fork's platform, not the flags, makes V8 deterministic; cross-OS identical hashes; mandatory worker-mode field | HGBA-3.1.4 Q-3, D10; design Section 3.3 |
| `v8-cppgc-ext` cannot sit on the fork unchanged. 2.1.4 made it an ABI-check layer; 2.1.6 retires it, because every build is now from source | HGBA-2.1.6 S-4e |
| Debug V8 works on Windows from source (the earlier "no debug on Windows" finding was wrong) | HGBA-2.1.4 S-4c; design Section 7.1 |
| Windows release crashes when the V8 platform is destroyed at exit (stock platform too) | HGBA-2.1.4 S-4f; design Section 7.1 |
| `test_fast_calls_pointer` crashes under the sandbox | Design Section 5.3 fast-call rule (golden-file plus full-IDL check) |
| Build requirements per platform (libclang 21+ and `CLANG_PATH`; native arm64 Python; VS path, SDK Debugging Tools, autocrlf off, job cap; `fadec` and `disarm` submodules; cgroup-aware memory guard) | Design Section 7.1 |
| TSAN on Linux works under the sandbox, but needs suppressions for V8's uninstrumented code | Design Section 7.1 |

## Open items

1. Push the fork to a shared remote and stand up the build farm and `RUSTY_V8_MIRROR`.
2. Root-cause the Windows release teardown crash (S-4f) and the fast-call crash before M2. Until then, the fork's platform tests keep the platform alive on Windows (both `test_predictable_platform` and `test_single_threaded_default_platform` hit the crash; confirmed 17 October 2026).
3. Add `fadec` and `disarm` as tracked submodules in the fork. `disarm` is committed (6d065ee); `fadec` (c9f78f5, the V8 DEPS pin) is staged on the Mac fork as of 17 October 2026, not yet committed.
4. Admin action on the Windows laptop: add "Debugging Tools for Windows" to Windows SDK 10.0.26100.
5. Owner decisions: D-S8 (funding), the `rusty_v8` owner. The name is decided: **Izcal** (D-S9, 17 October 2026); trademark clearance is still to do.
6. Deferred by the owner: signed-in DRM playback tests.

## Disk

| Host | Usage |
|---|---|
| Mac `m0/` | About 2 GB, plus the DRM spike |
| huey `~/hgba-m0` | About 15 GB, of which roughly 7 GB of test and probe build folders can be deleted |
| Laptop `C:\hgba-m0` | About 16 GB at M0's end. **Gone as of 17 October 2026**: the laptop was reinstalled on 16 October (C: now has 1.6 TB free, Rust is not installed). The Windows fork branch can be rebuilt from the Mac fork, and the Windows results and logs copied to `m0/windows/` survive |
