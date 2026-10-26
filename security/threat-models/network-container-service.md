# Threat model: Network container service

Owner: Network and security      Reviewed: self-approved, M1 phase E (26 October 2026); security owner review pending      Surface inventory: not generated yet (SEC-6). At M1 the endpoint set is [`process.hgidl`](../../crates/hg-ipc-gen/idl/process.hgidl) (`ProcessControl`, which the service serves, with the container configuration (`ChildConfig::NetworkContainer{container}`))

Filled at M1 (security scope 6.5, SEC-26). At M1 the network services are skeletons: one library, launched as three kinds in the no-socket sandbox (requirements C3). Sockets, LNA tickets, the OS adapter and spares belong to NW-M0's own plan. This model states what M1 built, and keeps the security scope's first pass as the model NW-M0 must meet.

## 1. Assets

- **At M1:** nothing beyond its bootstrap channels. The M1 broker does not launch this kind yet; the kind, its configuration and its sandbox exist.
- **From NW-M0 (first pass):** one container's policy (credential mode, block-list set, GPC and permission defaults, bounce-tracking state) and its table of partition services.

## 2. Entry points

- **`ProcessControl` from the broker**, as for every M1 network kind.
- **From NW-M0:** container-policy requests from the broker and the system service; sealed content-blocking files, read only through `hg-cb-format` (G80). No page bytes and no connection state.

## 3. Attacker capabilities assumed

- ATK-2 from the broker or a partition service; ATK-5 if a policy file is served.
- Every peer process is assumed compromised (IPC-R17).

## 4. Mitigations (M1 status, code and tests)

| Mitigation | Where | Test or evidence |
|---|---|---|
| One binary, [`hg-net-service`](../../crates/hg-net-service/src/lib.rs) behind `hg-net-main`, launched by kind (`NetworkSystem`, `NetworkContainer`, `NetworkPartition`; [`hg-launch-plan`](../../crates/hg-launch-plan/src/lib.rs)). It refuses a `start` whose configuration is not its own kind's, a second `start`, and any `work` before `start` | `hg-net-service` `start`, `work` | dst-sim `roundtrip`, `netcrash`, `faults` |
| **No socket creation (NW-D4).** Linux: the D3 seccomp allowlist denies `socket`, `connect` and `bind`, and an empty network namespace leaves nothing to reach. macOS: the SBPL denies `network-outbound`. Windows: an AppContainer without `internetClient`. Sockets come later (NW-M0) only as typed handles the broker grants | [`sandbox/seccomp.rs`](../../crates/hg-substrate-os/src/sandbox/seccomp.rs); [`sandbox/macos.rs`](../../crates/hg-substrate-os/src/sandbox/macos.rs); [`sandbox/windows.rs`](../../crates/hg-substrate-os/src/sandbox/windows.rs) | Launch facts E3 (`connect` to 1.1.1.1 gives ENETUNREACH in a new netns), **E26**, E30; FP-M0 tier probes |
| **seccomp's socket denial is what keeps the service off the host's Unix sockets.** Landlock ABI 4 does **not**: under it, `connect` to the D-Bus session and system buses and to `/var/run/docker.sock` succeeded on hueyj. Kani harness 2 (E-R11) must keep "no `socket`, `connect` or `bind` for any kind at M1". When NW-M0 gives these kinds socket calls, Landlock scoping (a later ABI) or the full tier's empty root has to replace that guarantee | — | Launch facts **E25, E26** |
| Identity: the service trusts its bootstrap channel, never a pid (review M8); the broker binds its channel to the launch's `OsPeer` | [`launch.rs`](../../crates/hg-substrate/src/launch.rs) contract | Launch facts E12, E13 |
| A crash or a lost message ends the channel (`Closed`), and the broker relaunches the kind within its budget. The broker resends unanswered work for the live renderer, and drops a dead renderer's work | [`hg-browser`](../../crates/hg-browser/src/lib.rs) `forget`, `relaunch` | dst-sim `netcrash` (every answer arrives after the network is killed mid-conversation), `faults` (IPC drop) |
| `panic = "abort"`; the broker restarts a crashed service (IPC-R5) | Workspace profile | — |

## 5. Residual risk

- A compromise changes one container's blocking and credential policy, not another container's.
- **`newfstatat` reveals whether a path exists** under the full Linux sandbox (launch facts E28). The service can learn which files exist, but cannot open them.
- `sendto` with an address works on an unconnected datagram socket the process already holds (E27). None reaches it today; the proposed hardening is to allow `sendto` only with a null address.
- **Linux restricted tier:** no empty root and no id mapping; a non-reference configuration (`linux-restricted-userns`). **The full tier is unclaimed until E-Q1.**
- **macOS:** `sandbox.h` in SDK 27.0 is deprecated and marks `sandbox_init` and `sandbox_free_error` "No longer supported" (E33). Custom profiles work today (E30, E31); this is a deprecation risk to re-measure at each SDK (SEC-D6).
- **Windows M1 tier** (`windows-m1`): no alternate desktop, no win32k lockdown. SEC-D13's network-service tier (AppContainer with `internetClient`, or LPAC) applies at NW-M0.

## 6. Open items

- The broker launches this kind at NW-M0. That launch and its tests update this file.
- E-Q1, M3 and M7 as for the partition service.
- The generated surface inventory (SEC-6).
