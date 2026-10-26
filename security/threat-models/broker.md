# Threat model: Broker (browser process), including the Linux spawner

Owner: Substrate and IPC      Reviewed: self-approved, M1 phase E (26 October 2026); security owner review pending      Surface inventory: not generated yet (SEC-6). At M1 the endpoint set is [`process.hgidl`](../../crates/hg-ipc-gen/idl/process.hgidl) (`ProcessHost`, which the broker serves) plus the transports' control links

Filled at M1 (security scope 6.5, SEC-26) from the security scope's first pass and from what M1 built. This file states what the code on `substrate/m1e-faults` does today and what is designed but not built yet; each item says which. The Linux spawner is in the broker's trusted computing base and is covered here (security scope 6.3a).

## 1. Assets

- **Every grant.** Each child's bootstrap channel ends. The site-to-process map (at M1: one renderer and one network partition service, [`hg-browser`](../../crates/hg-browser/src/lib.rs)). Every region it allocated and the ledger that charges each region to an owner (D6). On Windows, the relay ledger and tickets for every channel end that moves (D5).
- **Launch authority.** The `Launcher` ([`hg-substrate/src/launch.rs`](../../crates/hg-substrate/src/launch.rs)): it starts children of the M1 kinds (renderer and the network system, container and partition services), kills them (`Launcher::kill`), and enters restart-required (DU-3).
- **The spawner** (Linux, designed, being built on `substrate/m1e-launch`): the ability to create namespaces and exec an Izcal binary from a pre-opened O_PATH descriptor. It holds no page data, profile keys or network.
- **Later milestones:** the permission store, profile and policy, consent records, agent origin sets, native windows and OS input, and the clipboard, drag and personal-dictionary services (security scope 6.5's first pass). None of these exists at M1.

## 2. Entry points

- **`ProcessHost` from each child**, one bound channel per child: `ready(kind, build, sandbox)`, `pong`, `ask` (renderer), `answer` (network). Each child's channel is bound to the `OsPeer` its launch returned. A record from any other process is a transport violation (IPC-R2).
- **Transport control links** (Windows, D5): relay, redeem, bind and `Rebind` requests from every child. Each child has its own broker worker and a cap on queued requests (`MAX_CHILD_REQUESTS`; M8).
- **Region requests:** `allocate_region` names an owner, and the broker charges it (D6).
- **The spawner's replies** (Linux, designed): `Spawned{id, outer_pid, ns_pid, tier}` with a pidfd, or `Failed{id, stage, errno}`. The format is fixed, decoded by [`hg-launch-plan`](../../crates/hg-launch-plan/src/lib.rs).
- **The spawner's own entry point** (designed): one `Spawn{id, kind}` request from the broker over a SEQPACKET socketpair, carrying exactly two descriptors.
- **The process's own command line and environment** (ATK-7). At M1 the mains parse nothing (S1-M1b; being built on `substrate/m1e-launch`).

## 3. Attacker capabilities assumed

- **ATK-2 from every child:** arbitrary records on its own channels, including ones the generated decoder refuses; floods; `ready` with a wrong kind or build; answers to work never sent; withholding `ready`; moving channel ends in hostile orders (Windows: leaving reads pending on an end it gives away, probe W3).
- **ATK-2 against the spawner, from the broker only:** the spawner has no other peer.
- **ATK-7:** local code running as the same user can signal, ptrace or replace the broker and the spawner, and edit the profile, the command line or the environment.

## 4. Mitigations (M1 status, code and tests)

| Mitigation | Where | Test |
|---|---|---|
| Typed, bounded messages; unknown ordinals and oversize fields refused (IPC-R3, R4) | `hg-ipc-codegen`, [`process.rs`](../../crates/hg-ipc-gen/src/generated/process.rs) (generated) | Golden and fresh-IDL tests; 74 Kani harnesses over the generated `process` codecs |
| Bound channels: a record from any process but the bound peer is `Violation(Identity)` (IPC-R2) | Linux `SCM_CREDENTIALS` (D3); macOS audit token (D4); Windows broker ledger (D5); simulator | `a_bound_receiver_hears_only_the_peer_it_names` (sim); per-OS transport tests |
| A flooding renderer cannot grow the broker's queue: the `MAX_PENDING`+1th ask ends its channel, and a child that violated is never relaunched | [`hg-browser`](../../crates/hg-browser/src/lib.rs) `ask`, `ended` | dst-sim `flood` (30 seeds, run twice) |
| **DU-3:** a child with another `BuildId` is told `exit(BuildMismatch)`, and restart-required is absorbing: no further launch of any kind | `hg-browser` `ready`; `Launcher::restart_required` | dst-sim `du3`: exactly one mismatch, 3 launches, 0 after it, no missed deadline |
| **Ready deadline (E-R3):** a child not ready within 5 s† on the substrate's clock is killed (`Launcher::kill` reaches only the child that launcher started, in that incarnation) and the spawn is a failure | `hg-browser` `deadline`; `Launcher::kill` | `kill_ends_the_named_child`, `kill_reaches_only_its_own_child`; dst-sim `faults` (ready stalls), `--stall-ppm` |
| Answers are routed under the broker's own work ids, never the renderer's. Each piece of work is tied to the renderer launch that asked, so a dead renderer's answers never reach its successor, and a renderer's answer cannot be taken for another's | `hg-browser` `Work`, `answer` | dst-sim `relaunch` and `faults`: renderer refused-answer count 0 over 60 seeds |
| Relaunch and retry are bounded per kind (restart budget); a failed launch (refused, spawner died) is retried within it | `hg-browser` `relaunch` | dst-sim `faults`, each fault class alone |
| Handles move only as their kind, a bound sender only once and only from its creator (IPC-R1) | Every transport; `ipc/unix/accept.rs` (Linux accepts only `SOCK_SEQPACKET` channel ends and the region kinds); `ipc/win/accept.rs` (Windows relay classification) | Kani: `unix/accept`, `unix/decide`, `win/accept`, `win/decide`, `mach/accept`, `mach/decide` |
| Regions: the broker allocates every region and charges it to a named owner, within per-owner quotas; a writer comes only from the broker (D6) | `hg-substrate/src/ipc/region.rs`; per-OS `ipc/shm.rs` | D6 cross-process tests; Kani region harnesses; dst-smoke `--regions` |
| **The spawner (designed, E-R2, review B2, M6):** a static-pie `[[bin]]` of `hg-substrate-os`. It parses no arguments, has no shell, passes no environment, accepts only fixed-layout `Spawn{kind}` records, refuses `MSG_CTRUNC`, receives with `MSG_CMSG_CLOEXEC`, and closes its copies of the child's ends before replying, on every path. The broker checks `outer_pid` against the pidfd's `Pid:` line and `ns_pid` against `NSpid:` | `hg-launch-plan` (the codec); the spawner is being built on `substrate/m1e-launch` | Kani: `request_decode_is_total_and_exact`, `reply_decode_is_total_and_exact`, `wrong_lengths_are_refused`, `requests_round_trip`, `kinds_map_to_their_executables` |
| Children's seccomp launch filter = steady list + `execveat` only (review B1) | [`sandbox/seccomp.rs`](../../crates/hg-substrate-os/src/sandbox/seccomp.rs) `launch_rules` | The seccomp proof and the table tests |
| `panic = "abort"` in shipping and simulation binaries (IPC-R5, CS-ER-2) | Workspace `[profile.release]` | — |
| Deterministic simulation of the broker under crash, IPC delay, reorder and drop, spawner-death, refused-launch and missed-deadline faults | [`hg-substrate-sim`](../../crates/hg-substrate-sim/src/world.rs), [`dst-sim`](../../crates/hg-dst-sim/src/bin/dst-sim.rs) | dst-sim scenarios; identical traces on the three OS models |

## 5. Residual risk

- **The broker runs unsandboxed** by necessity. Its OS-level hardening (Hardened Runtime and Enhanced Security on macOS; CET, CFG and a medium-integrity per-user install on Windows; CET or IBT on Linux) is not built at M1. Same-user code can inject into it (ATK-7).
- **Logic bugs in policy and handle relay** (security scope 3.2). At M1 the relay is D5's Windows ledger and D6's region registry.
- **Generations are the broker's labels, not the kernel's** (review M6). `OsPeer`'s generation is the launcher's own sequence number. The kernel attests only the pid (and, on macOS, the audit token's version). Pid reuse cannot forge a record on a bound channel: a bound sender cannot be transferred, and a dead child's channels close before its pid can be reused.
- **Windows: the move-steal limit (probe W3), decided but not implemented.** A compromised child can leave a `ReadFile` pending on an end it then gives away, and keep taking records sent on that channel afterwards. It is confined to channels that child already held. The owner decided on 26 October that **every move puts the moved end and its peer on a fresh pipe**. Until that lands, the D5 code documents the limit. After it lands, one limit remains: a compromised old receiver can take one record that was queued at the move, and the new holder then sees `Violation(Sequence)`, unattributed. That is detected, not silent.
- **Windows: pipes attest no sender per message** (probe P13). Attribution comes from the broker's ledger, so a violation a child reports is unattributed.
- **D6 residuals.** On macOS, enforcement is at acceptance only: a child can make memory entries itself, and the OS charges them to the child (Q2). The ledger charges a region to its owner until the owner exits. On macOS and Windows a read-only region is trusted to the process that filled it, unless its IDL field is `immutable` (macOS: a snapshot, whose hidden memory the region quota bounds; Windows: the broker's copy).
- **The spawner:** a compromise is a compromise of sandbox setup for future processes, not of existing ones. It holds no data.

## 6. Open items

- **The full Linux tier is unclaimed** until the owner answers E-Q1 (an AppArmor `userns` profile on the measurement host, as root). M1 builds and measures the restricted tier only (`linux-restricted-userns`: user, PID and net namespaces, no mount namespace, no empty root, no id map; launch facts E1–E5, E22).
- The OS launchers (Linux spawner, macOS `posix_spawn` with the code-directory check, Windows AppContainer with the job assigned while suspended) are on `substrate/m1e-launch`. When they land, this file gains their tests.
- M3: the broker checks each child's `SandboxReport` against the tier the launcher applied; a mismatch kills the child. Today the report is received and ignored.
- M7: `PeerAttest::Inherited` (a child attests its bootstrap channel, never a pid) is on `substrate/m1e-launch`.
- The generated surface inventory (SEC-6) does not exist yet. Section 2 lists the endpoints by hand.
- Proposed seccomp hardening for children: allow `sendto` only with a null destination address (launch facts E27). Forwarded to the seccomp table's owner.
- The security scope's first-pass open item stays: which Windows mitigations (CIG, extension-point blocking) the broker can carry while it owns every prompt window, measured at M5.
