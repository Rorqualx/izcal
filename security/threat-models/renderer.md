# Threat model: Renderer (per site, container), including the spare renderer

Owner: JS, bindings and DOM      Reviewed: self-approved, M1 phase E (26 October 2026); security owner review pending      Surface inventory: not generated yet (SEC-6). At M1 the endpoint set is [`process.hgidl`](../../crates/hg-ipc-gen/idl/process.hgidl) (`ProcessControl`, which the renderer serves)

Filled at M1 (security scope 6.5, SEC-26). At M1 the renderer is a skeleton, [`hg-renderer`](../../crates/hg-renderer/src/lib.rs). It has no web content, parser, engine or fonts. This model states what that skeleton and its sandbox do today, and keeps the security scope's first pass as the model the full renderer must meet. The spare renderer is covered here (security scope 6.3a); it is not built at M1.

## 1. Assets

- **At M1:** its bootstrap channels to the broker, and the round-trip state of the M1 exit test (ask ids and answers).
- **Full renderer:** one (site, container)'s DOM, JS heap, the session cookies it was handed, and form data (first pass).
- **Spare renderer, before adoption:** nothing (no document, no site key, no bound endpoints).

## 2. Entry points

- **`ProcessControl` from the broker**, on the bootstrap receiver the renderer was born with: `ping`, `exit`, `start(ChildConfig)` (once, only the renderer configuration), `work` (always refused), and `answered`. `answered` is accepted only in order, one per ask, and only if it carries the answer to the ask it names; anything else is refused and counted.
- **The OS's system-call, Mach and Windows service surface** that remains after lockdown (section 4).
- **Full renderer:** all web content, the parsers, the engine boundary (H-1, H-2) and the rest of the first pass's list.

## 3. Attacker capabilities assumed

- **ATK-1 fully** once it renders content; after a bug, **ATK-2 within the process**, including arbitrary read and write inside the engine cage (the premise of JS-10).
- **At M1:** a compromised renderer is the attacker against the broker and the OS. The broker is assumed honest toward it, but every record from the broker is still decoded by the generated, bounded decoder (IPC-R17).

## 4. Mitigations (M1 status, code and tests)

| Mitigation | Where | Test or evidence |
|---|---|---|
| The renderer **never attests a peer by pid** (review M8). It trusts its bootstrap channel because the channel came at birth and only the broker holds the other end | `hg-renderer` `run`; contract doc in [`launch.rs`](../../crates/hg-substrate/src/launch.rs) | Launch facts E12, E13, E23 (in a PID namespace every outside process reads as pid 0). `PeerAttest::Inherited` (M7, 4fd9db7); the simulator models it (`a_namespaced_child_*` tests in `hg-substrate-sim`) |
| `ready` reports the build and a `SandboxReport` (tier plus missing 6.7 features); the broker kills a child with another build (DU-3) | `hg-renderer`; `hg-browser` | dst-sim `du3` |
| Answers are checked: out of order, duplicated or misrouted answers are refused | `hg-renderer` `answered` | dst-sim `faults` and `relaunch`: 0 refused over 60 seeds each |
| **Linux:** the D3 seccomp allowlist with an arch check. It denies `socket`, `connect`, `bind`, process and namespace creation (`clone` threads only; `clone3` gives ENOSYS), `PROT_EXEC` maps, `memfd_create` and `ioctl` TIOCSTI. It is installed with TSYNC; the launch filter is the same list plus `execveat` (review B1) | [`sandbox/seccomp.rs`](../../crates/hg-substrate-os/src/sandbox/seccomp.rs), [`sandbox/linux.rs`](../../crates/hg-substrate-os/src/sandbox/linux.rs) | Table tests and the seccomp Kani proof; FP-M0 tier probe; launch facts E26 |
| **Linux restricted tier:** user, PID and net namespaces (no mount namespace), Landlock ABI 4 handling every filesystem right and both TCP rights, the executable exec'd from the spawner's pre-opened descriptor, and **static-pie** binaries (designed: E-R4, E-Q5) | Spawner on `substrate/m1e-launch` | Launch facts E3, E6, E7, E22 |
| **seccomp's socket denial carries the weight against host Unix sockets.** Landlock ABI 4 does **not** stop `connect` to path-named Unix sockets: under it, the D-Bus session and system buses and `/var/run/docker.sock` accepted a connect on hueyj. With the D3 filter, `socket` and `connect` give EPERM. The new network namespace hides abstract sockets | — | Launch facts **E25, E26, E29**; Kani harness 2 (E-R11) must keep "no `socket`, `connect` or `bind` for any kind at M1" |
| **macOS:** custom deny-default SBPL (`(deny default) (allow sysctl-read)`) through `sandbox_init_with_parameters`. Never a built-in named profile | [`sandbox/macos.rs`](../../crates/hg-substrate-os/src/sandbox/macos.rs) | **Launch facts E30–E32: in a real SDK 27.0 build (Xcode 27.1, macOS 27.0.1) the profile applies and enforces, allow rules hold, and a named profile is SIGKILLed** |
| **Windows:** AppContainer without capabilities, a lockdown token, Untrusted integrity, its own kill-on-close job; one AppContainer SID per live child (D5 item 10) | [`sandbox/windows.rs`](../../crates/hg-substrate-os/src/sandbox/windows.rs) | FP-M0 tier probe on rig253 |
| No handle a child is given can be one its sandbox would deny: IPC acceptance takes only channel ends and region kinds (Linux: `SOCK_SEQPACKET` only) | `ipc/unix/accept.rs`, `ipc/mach/accept.rs`, `ipc/win/accept.rs` | Kani acceptance harnesses |

## 5. Residual risk

- **`newfstatat` reveals whether a path exists** (launch facts E28). Under the full Linux sandbox, `stat` of any path succeeds: Landlock ABI 4 has no metadata right, and the D3 filter allows `newfstatat` for std's descriptor queries. A compromised renderer learns whether files exist, and their metadata. It cannot open them or connect to them.
- **`sendto` with an address** works on an unconnected datagram socket the process already holds (launch facts E27). No path gives a renderer one today: `socket` and `socketpair(SOCK_DGRAM)` are denied, IPC acceptance takes only seqpacket, and at exec only stdio and the bootstrap ends survive. Proposed hardening: allow `sendto` only with a null address.
- **Linux restricted tier:** no empty root and no id mapping (launch facts E2). This is a named, non-reference configuration (`linux-restricted-userns`, review M4). **The full tier stays unclaimed until E-Q1** (the owner's AppArmor `userns` profile). Kernel bugs reachable through the allowed syscalls remain.
- **macOS: the API is deprecated.** In the SDK 27.0 `sandbox.h` (Xcode 27.1), the header "is deprecated and may be removed in a future release". It marks `sandbox_init` and `sandbox_free_error` `API_DEPRECATED("No longer supported", macos(10.5, 10.8))`, and no longer declares the `kSBXProfile*` constants. `sandbox_init_with_parameters` and `sandbox_check` are not public (launch facts E33). Custom profiles work today, but Apple does not call them supported. **Deprecation risk, recorded with SDK 27.0; re-measured with each new SDK (SEC-D6).**
- **Windows M1 tier:** no alternate desktop and no win32k lockdown (`windows-m1`, until M5).
- **First-pass residuals for the full renderer:** engine logic bugs are contained, not absent. Persistence through engine images is bounded by FP-D16. Spectre within the process. Subdomains of one site share a renderer below the origin-keying threshold. A scriptless top-level SVG can read the files in its granted directory.

## 6. Open items

- E-Q1 (full Linux tier), M3 (the broker's cross-check of the `SandboxReport`), and the launch paths on macOS and Windows: on `substrate/m1e-launch`.
- Re-run the macOS SBPL probe (`research_notes/M1 E launch/probes/macos`) at each new SDK.
- Sandbox extensions on macOS 27 (SEC-D6) and the Windows token combination (M5) remain from the first pass.
- The generated surface inventory (SEC-6).
- Spare renderer: built only if FP-M0's numbers justify it (FP 6.3). Before adoption it gets the strictest profile, and it is bound and given its endpoints in one transaction.
