# SEC-D6 measurement: Seatbelt (SBPL) on macOS 27 and SDK 27

Run on 18 October 2026, 15:21 to 15:23 MDT, on rig `local` (Huey the Sixth), under booking b-87b60c (light, session browser-61; released afterwards). The poll started at 13:46. The run went ahead as soon as `rig check local --kind light` returned OK, at 15:21 (oot-default-s4's exclusive window ended early).

Probe source, scripts and raw logs: this folder (`src/`, `build.sh`, `run.sh`, `logs/`; the built binaries in `bin/` are not committed, `build.sh` recreates them)
- `src/probe.c`: one C probe with modes. Its SBPL comes from `sandbox_init(3)`, `sandbox-exec(1)` and black-box tests only, and no other browser's profile was read.
- `build.sh` and `run.sh`
- `logs/build-20261018-152145.log`, `logs/run-20261018-152150.log` (first full run) and `logs/run-20261018-152247.log` (rerun with corrected sandbox-exec profiles; the in-process matrix is identical, checked by diff)
- Crash reports: `~/Library/Logs/DiagnosticReports/probe-sdk2*-2026-10-18-1521*.ips` and `sandbox-exec-2026-10-18-152150.00010.ips`

## Versions

| | |
|---|---|
| OS | macOS 27.0.1 (26A434), Darwin 27.0.0, xnu-13432.1.9~1/RELEASE_ARM64_T8152, Mac18,5, SIP enabled, developerMode 1 |
| Xcode | 26.3 (17C528), the only Xcode in /Applications, with no Xcode 27 or beta present |
| SDKs installed | Xcode has MacOSX26.2.sdk (the default, `xcrun --show-sdk-version` = 26.2). CommandLineTools has MacOSX13.3, 13, 14.4 and 14 (MacOSX.sdk → 14.4). **No SDK 27 on the host.** |
| clang / ld | Apple clang 17.0.0 (clang-1700.6.4.2); ld-1230.1 |
| `/usr/bin/sandbox-exec` | LC_BUILD_VERSION minos 27.0, sdk 27.0. This is an Apple binary really built against SDK 27. |

## Builds (one source, only LC_BUILD_VERSION varies)

All builds use `xcrun clang -isysroot <MacOSX26.2.sdk> -arch arm64 -O1 -Wall -Wextra` and are ad-hoc signed:

| Build | Extra flags | `vtool -show-build` | dyld at runtime (`dyld_get_program_sdk_version`) |
|---|---|---|---|
| sdk262 | `-mmacosx-version-min=14.0` | minos 14.0, sdk 26.2 | 26.2.0 |
| sdk27-ld | `-mmacosx-version-min=14.0 -Wl,-platform_version,macos,14.0,27.0` | minos 14.0, sdk 27.0 | 27.0.0 |
| sdk27-min27 | `-mmacosx-version-min=27.0 -Wl,-platform_version,macos,27.0,27.0` | minos 27.0, sdk 27.0 | 27.0.0 |
| sdk27-vtool | the sdk262 binary, `vtool -set-build-version macos 14.0 27.0 -replace`, then `codesign -s - -f` | minos 14.0, sdk 27.0 | 27.0.0 |
| sdk28-ld | `-Wl,-platform_version,macos,14.0,28.0` | minos 14.0, sdk 28.0 | 28.0.0 |

The vtool build has byte-identical code to sdk262 and differs only in the load command. Any change in behavior is therefore keyed off the Mach-O SDK field.

## What each test does

Each mode runs as a child process of `run.sh`. Before applying the sandbox, the probe opens a loopback TCP listener, so a later connect has a peer (EPERM then means policy, not "no peer"). After the sandbox is applied it checks the following:
- `sandbox_check(getpid(), NULL, 0)`
- reading the allowed fixture and the denied fixture
- creating a file in the fixtures directory
- `connect()` to the listener
- `bootstrap_look_up` of `com.apple.system.notification_center` (the "allowed" service) and `com.apple.cfprefsd.daemon` (the "denied" service)
- a compute loop

The custom profiles are:
- **T1 custom-init**: `sandbox_init("(version 1)(deny default)(allow file-read* (literal \"<allowed>\"))", 0, &err)`
- **T2 custom-params**: `sandbox_init_with_parameters("(version 1)(deny default)(allow file-read* (literal (param \"ALLOWED_FILE\")))", 0, {"ALLOWED_FILE", <allowed>, NULL}, &err)`. This is declared by hand because it is not in the public header.
- **T3 named-pure**: `sandbox_init(kSBXProfilePureComputation, SANDBOX_NAMED, &err)`. **T3b named-nonet** does the same with `kSBXProfileNoNetwork` (extra).
- **T5 mach**: T1 plus `(allow mach-lookup (global-name "com.apple.system.notification_center"))`

The baseline (no sandbox) succeeded on every check in every build.

## Results matrix (in-process, macOS 27.0.1 runtime)

| Test | sdk 26.2 (real SDK) | sdk 27.0 via ld | sdk 27.0 + minos 27 | sdk 27.0 via vtool | sdk 28.0 via ld |
|---|---|---|---|---|---|
| T1 custom profile, `sandbox_init(…, 0, …)` | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** |
| T2 custom profile, `sandbox_init_with_parameters` | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** |
| T3 `kSBXProfilePureComputation`, `SANDBOX_NAMED` | applied + enforced | **process killed (SIGKILL)** | **killed** | **killed** | **killed** |
| T3b `kSBXProfileNoNetwork`, `SANDBOX_NAMED` | applied + enforced (network only) | **killed** | **killed** | **killed** | **killed** |
| T5 custom profile + one `mach-lookup` allowed | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** | **applied + enforced** |

"Applied + enforced" for T1, T2 and T5 means all of the following, identically in every build:
- rc=0, err=(null), `sandboxed=1`
- `read_allowed=OK`
- `read_denied=FAIL errno=1 (EPERM)` and `write_tmp=FAIL errno=1`
- `net_connect=FAIL errno=1 (EPERM)`, with a live listener present
- `compute=OK` and `reached_end=1`, exit 0
- T1 and T2: both Mach lookups fail with `kr=0x44c (Permission denied)`. T5: the allowed service returns `OK` and the other fails with `0x44c`.

launchd logs each Mach denial as `denied lookup: name = com.apple.cfprefsd.daemon … error = 159: Sandbox restriction`. That covers deny-by-default Mach lookup with a single-service allowlist (HGBA-2's Mach-port-under-Seatbelt assumption) for these Mach lookups. Port handoff between peers was not tested.

For the named profiles under sdk 26.2, PureComputation denied every check, including reading the "allowed" file and both Mach lookups, while compute still worked. NoNetwork denied only the connect.

**How the kill happens (T3/T3b, SDK field ≥ 27.0):**
- The process gets SIGKILL inside the `sandbox_init` call, before it returns. stdout stops after the `dyld_sdk=` line, and the shell reports `Killed: 9` (exit 137).
- Crash report (bug_type 309): `exception EXC_CRASH SIGKILL`, `termination {"namespace": "SANDBOX", "flags": 4162, "code": 1}`
- Faulting stack: `__mac_syscall` ← `libsandbox.1.dylib sandbox_apply_container` ← `libsystem_sandbox.dylib sandbox_init_with_parameters` ← `main`. The named path goes through `sandbox_init_with_parameters` internally, and the kill happens in the kernel apply syscall.
- No error string is returned, and nothing came from sender "Sandbox" in `log show`. The only log lines were osanalyticshelper writing the report.

The same named-profile call under the real 26.2 SDK field works. The gate is "SDK ≥ 27" (28.0 is also killed), not a check on the minos field.

## sandbox-exec cross-check (T4)

`/usr/bin/sandbox-exec` is itself built against SDK 27.0.

| Command | Outcome |
|---|---|
| `sandbox-exec -p '(version 1)(allow default)(deny file-read* (literal "<denied>"))' /bin/cat <denied>` | `Operation not permitted`, exit 1, so the custom profile was enforced |
| the same profile, `cat <allowed>` | prints the file, exit 0 |
| `sandbox-exec -n pure-computation /usr/bin/true` | **sandbox-exec itself SIGKILLed** (exit 137). The report shows `termination namespace SANDBOX, code 1`, with procPath `/usr/bin/sandbox-exec` |
| `sandbox-exec -p '<deny default; allow process-exec of bin/; allow file-read* minus one literal; sysctl-read; localhost bind/inbound; one mach-lookup>' bin/probe-{sdk262,sdk27-ld,sdk27-min27} checks …` | in all 3 builds: sandboxed=1, allowed read OK, denied read EPERM, connect EPERM, allowed Mach service OK, other service 0x44c, exit 0 |
| the same with a narrow read allowlist (`/usr/lib`, `/System`, `/private/preboot`, `/private/var/db/dyld`, `bin/`, the fixture, plus `file-read-metadata`) | SIGABRT from dyld (`dyld4::CacheFinder` → `abort_with_reason`) **in both sdk262 and sdk27 builds**. My allowlist is missing a path dyld needs at exec time. This has nothing to do with the SDK, and the path was not tracked down further. |

An Apple binary really linked against SDK 27 (sandbox-exec) applies custom SBPL from `-p`, and the profile is enforced. The same binary is killed for a named profile. This matches the in-process results.

## Desk check

- **The man page text (third-party quote, not checked against an SDK 27 copy).** Sevan Janiyan, openssh-unix-dev, 15 August 2026, quotes the SDK 27 `sandbox_init(3)`: "These profiles must not be used when building against macOS SDK >= 27.0. Processes opting into them will be killed on attempt to do so." In context, "these profiles" are the named `kSBXProfile*` profiles. The same thread reports an Xcode 27 beta build failure in OpenSSH's sandbox-darwin.c, where `kSBXProfilePureComputation` is apparently undeclared or unavailable, and Darren Tucker's patch gates on `AC_CHECK_DECLS(kSBXProfilePureComputation…)`. https://lists.mindrot.org/pipermail/openssh-unix-dev/2026-August/042690.html, https://lists.mindrot.org/pipermail/openssh-unix-dev/2026-August/042691.html. Ingo Schwarze removed mandoc's `sandbox_init` use, calling it unsupported (his opinion, untested on 27): https://lists.mindrot.org/pipermail/openssh-unix-dev/2026-August/042695.html. **The local SDK 26.2 man page (dated 9 March 2017) does not have this text.** I could not read the SDK 27 page itself.
- **Apple's position.** Apple DTS (Quinn, March 2025) says SBPL "has never been documented for third-party use, and thus is not considered API. That's why everything within `<sandbox.h>` is deprecated". Later follow-ups in the same thread, dated only relatively (about 3 and 1 weeks before capture), say: "macOS 27 introduced an alternative approach for third-party developers to sandbox child processes, namely the `es_new_descendants_client` mechanism within Endpoint Security" and "the supported options are … ES, App Sandbox, and Virtualization." https://developer.apple.com/forums/thread/777509. **Apple does not state that custom profiles are supported. It says the opposite.**
- **`es_new_descendants_client`** (Endpoint Security). Per Apple DTS (Kevin Elliott) and an Apple security engineer, June 2026: it needs no root and no TCC prompt, it covers the caller's process subtree, and it can block operations ("effectively a new feature for creating dynamic sandboxes"). It needs an entitlement that the page does not name. The original poster in 777509 says it is the privileged `com.apple.developer.endpoint-security.client` (unverified). https://developer.apple.com/forums/thread/832204. This is a third candidate backend that SEC-D6 does not currently list.
- **macOS 27 and Xcode 27 release notes** do not mention `sandbox_init`, `sandbox.h`, `kSBXProfile*`, SBPL or Seatbelt. Xcode 27 ships the macOS 27 SDK and requires macOS Tahoe 26.6 or later. https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes, https://developer.apple.com/documentation/xcode-release-notes/xcode-27-release-notes. Their Enhanced Security entries cover arm64e.x1 / CPA2 / pointer-arithmetic checks only.
- **Enhanced Security helper extensions.** Per Apple's docs they are available on iOS, iPadOS, macOS and watchOS. They are launched with ExtensionFoundation (`AppExtensionPoint` with the `EnhancedSecurity()` attribute, `AppExtensionProcess`), and the only documented channel is `XPCSession`. The system runs them "in a sandbox that restricts access to most system services and frameworks", they cannot present UI, and the policy is Apple's, not the host's. **The docs say nothing about JIT, instance or process-count limits, Mach-port handoff, or a host-supplied policy.** Q-SEC-3 stays open. https://developer.apple.com/documentation/xcode/creating-enhanced-security-helper-extensions, https://developer.apple.com/documentation/xcode/enabling-enhanced-security-for-your-app. The Enhanced Security capability's "additional runtime platform restrictions" (`hardened-process.platform-restrictions-string`) turn some Mach IPC trap patterns into crashes. That matters for HGBA-2's raw Mach-port design, but I have not tested it.
- **Related, not shown to be the same mechanism.** Firefox on the macOS 27 beta (26A5388g), launched from the CLI with a fresh profile: `sandbox_extension_issue_file_to_process failed … Operation not permitted`, then the child was killed with signal 9. https://github.com/microsoft/playwright/issues/42082 (Mozilla bug 2060476). Garmin ConnectIQ hit the same error on 27.0.1 (26A434): https://forums.garmin.com/developer/connect-iq/i/bug-reports/simulator-sdk-9-2-0-exits-immediately-on-macos-27-sandbox_extension_issue_file_to_process-operation-not-permitted. These involve sandbox *extensions* (tokens), not `sandbox_init`, and I did not test them. They are a warning that a broker which issues file extensions to children may hit a separate macOS 27 change.

## Caveats

1. **No real SDK 27 was available.** The SDK-27 columns use 26.2 headers and libraries with the LC_BUILD_VERSION sdk field set to 27.0 (or 28.0). The run-time gate clearly keys off that field: the vtool build has identical code and changed behavior. A real SDK 27 can still differ at **compile time**. The headers may mark `sandbox_init` / `kSBXProfile*` unavailable (OpenSSH saw an undeclared constant). `sandbox_init_with_parameters` was never in the public header, so it is unaffected by header changes. A test with real SDK 27 needs Xcode 27 or its Command Line Tools installed. Xcode 27 requires macOS 26.6 or later, which this host meets. I did not install it.
2. sandbox-exec is real SDK-27 evidence that custom `-p` profiles work. However, it is an Apple platform binary, and a third-party exemption could differ. Its kill on `-n` shows it is not exempt from the named-profile gate.
3. Only one OS build was measured (27.0.1 / 26A434). The behavior is undocumented and could change in a point release (DTS calls it "an implementation detail").
4. The profiles are minimal: deny default plus one literal read, plus one mach-lookup. Profiles at renderer or GPU scale were not exercised under the SDK-27 field: iokit, shared memory, `(import …)`, filters beyond `param`, and JIT/`dynamic-code-signing` together with Hardened Runtime. Mach-port **handoff** between peers (as opposed to lookup) was not tested.
5. The narrow-read sandbox-exec profile failed to exec because dyld aborted, under both SDKs. That is a gap in my allowlist, not an SDK effect, but it means I did not demonstrate a narrow pre-exec profile.
6. Kills leave no message from "Sandbox". The only signal is SIGKILL plus a `namespace SANDBOX` termination in the crash report.

## Conclusion

On macOS 27.0.1, custom deny-default SBPL profiles still apply and enforce, whether applied with `sandbox_init(profile, 0, …)`, `sandbox_init_with_parameters` or `sandbox-exec -p`. They enforce file, network and per-service Mach-lookup rules whether the binary says SDK 26.2, 27.0 or 28.0. The kill described in the man page is real but limited to the built-in named profiles (`kSBXProfile*` / `-n`): a process whose SDK field is 27.0 or higher is SIGKILLed inside the `sandbox_init` call (termination namespace SANDBOX, code 1).

That meets the first half of SEC-D6's reversal trigger ("custom profiles still apply"), with the caveat that only a simulated SDK-27 link was available. It does not meet the second half. Apple has not said custom profiles are supported. DTS says SBPL is not API, and it now points third parties to Endpoint Security's `es_new_descendants_client`, App Sandbox or Virtualization.

The data therefore **keeps (b)**: Seatbelt as the working backend, plus an evaluation of an alternative from M1. It does **not** justify dropping the extension evaluation, and it does **not** point to a reduced tier: nothing measured forces the renderer below the custom-profile floor. Two adjustments for the owner to consider:
- Row 638 of the tier table should not rely on named profiles. Its text already says this.
- The (b) evaluation could add `es_new_descendants_client` as a candidate alongside Enhanced Security extensions, because Apple now names it as the supported route for sandboxing child processes.

Before M1 sign-off, the result should be confirmed once with a real Xcode 27 / SDK 27 build, to rule out a compile-time block.
