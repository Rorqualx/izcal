# M1 phase E launch: facts for the spawn path

Compiled 2026-10-26. Probes ran on:
- **hueyj:** Linux 6.8.0-147-generic, Ubuntu 24.04.5, x86_64, `kernel.apparmor_restrict_unprivileged_userns=1`, `kernel.unprivileged_userns_clone=1`, `user.max_user_namespaces=127001`, Landlock ABI 4, as user `joe` (uid 1000), never as root.
- **The Mac** and **rig253:** no new probe this round. Their spawn paths are D4's and D5's, already tested with sandboxed children; their spawn timings are FP-M0's (cited below).

Confidence: **High** means a probe on this hardware showed it, or a primary document was read and the claim checked in its text. **Medium** means a primary document only. **Low** means memory.

**M-1 statement.** Prose documents only: the man pages `clone(2)` (clone3), `user_namespaces(7)`, `pid_namespaces(7)`, `mount_namespaces(7)`, `pivot_root(2)`, `landlock(7)`, `seccomp(2)`, `execveat(2)`, `close_range(2)`, `unix(7)`, `pidfd_open(2)` and `proc(5)`, as installed on hueyj. No kernel, libc, AppArmor or third-party source was opened. Constants come from the `libc` crate as compiled; the Landlock access bits are the ones `landlock(7)` lists.

Probe program: `probes/linux` (one binary; `probe <child> [runs] [case-filter]` and `pidfd`). It ran as a dynamic and as a static (`-C target-feature=+crt-static`) build. Logs: `probes/logs/linux-hueyj-2026-10-26.log`, and the kernel's AppArmor lines for the same runs in `probes/logs/linux-hueyj-apparmor-denials.log`.

Spawn-to-ready below is warm: from `clone3` in the parent to the exec'd child's first record, median of 20. Cold runs need `drop_caches`, which needs root (open question E-Q6).

## 1. Linux: user namespaces under Ubuntu's AppArmor restriction

| # | Finding | Evidence | Conf. |
|---|---|---|---|
| E1 | An unprivileged `clone3(CLONE_NEWUSER | NEWPID | NEWNET | NEWNS)` **succeeds**. The kernel logs `apparmor="AUDIT" operation="userns_create" … target="unprivileged_userns"`: the child is moved into AppArmor's `unprivileged_userns` profile. | log, every namespace case | High |
| E2 | In that profile **every capability use is denied**, although `CapEff` reads `000001ffffffffff`. `setgroups` gives EACCES and `uid_map`/`gid_map` writes give EPERM, so the child's ids stay unmapped (uid 65534). `mount(MS_PRIVATE)`, a tmpfs mount, `pivot_root` and `umount2` all fail; `/etc/hosts` stays visible. The kernel logs `apparmor="DENIED" operation="capable" … capname="sys_admin"`. | `pivot:` rows; denials log | High |
| E3 | A **PID namespace works**: the child is pid 1. A **network namespace works**: `connect` to 1.1.1.1 gives ENETUNREACH (101). A PID namespace without a user namespace fails with EPERM, as `pid_namespaces(7)` says for an unprivileged caller. | `user+pid`, `user+net`, `pid only` | High |
| E4 | **`execveat` of an O_PATH fd opened before `CLONE_NEWNS` fails with EACCES** in the new mount namespace. The kernel logs `apparmor="DENIED" operation="exec" info="Failed name lookup - disconnected path" profile="unprivileged_userns" name="/dev/fd/100"`. Exec by path works there, and so does `execveat` of an O_PATH fd the child opens **after** the clone. With a user, PID or net namespace and no new mount namespace, the pre-opened fd works. | `user+mnt`, `user+mnt exec-by-path`, `user+mnt exec-fd-opened-in-child`; denials log | High |
| E4b | Reopening the pre-opened fd through `/proc/self/fd/N` inside the new mount namespace succeeds as an `open`, but its `execveat` fails the same way ("disconnected path", `name="/dev/fd/200"`). A procfs reopen does not reconnect the path. | `user+mnt exec-procfs-reopen`; log's added run | High |
| E5 | The kernel attributes E4 to AppArmor's path mediation, not to the exec rules of `execveat(2)`. Whether a host without AppArmor, or one with a profile that allows `userns` and disconnected paths, behaves differently is **not measured**: it needs root to install a profile (E-Q1). | E4's log line | High (cause on this host) |

**Implications:** on a stock Ubuntu 24.04 host, Izcal gets user, PID and network namespaces plus Landlock and seccomp, but **no empty root** and **no id mapping**. That is a named reduced tier for SEC-8, not the full floor. The full floor needs the AppArmor profile that DU's metapackage ships (DU 6.3, Q-DU-13). The spawner must open the executable fd inside the child's mount namespace, never before the clone.

## 2. Linux: Landlock and seccomp before exec

| # | Finding | Evidence | Conf. |
|---|---|---|---|
| E6 | Landlock **ABI 4** on 6.8 (filesystem rights v1–v3, TCP bind/connect). Handling every filesystem right and both TCP rights with no rule: opening `/etc/hosts` gives EACCES, and TCP `connect` gives EACCES. Ruleset creation plus `restrict_self` takes **4–7 µs**. | `landlock abi=4 restrict ok=0 … setup_us=6` | High |
| E7 | Landlock applied **before exec** with one rule (EXECUTE and READ_FILE on the executable's O_PATH fd, `LANDLOCK_RULE_PATH_BENEATH`) lets a **static** child exec and run. A **dynamic** child fails with EACCES, because its loader and libraries are outside the rule. | `FULL` case, static versus dynamic | High |
| E8 | Installing a 202-instruction seccomp filter with TSYNC in the child before exec takes **0.45–0.55 ms**. A filter that allows `execveat` survives exec, and the exec'd process starts under it. | `seccomp(200) ok=0 us=…` | High |
| E9 | `close_range(4, ~0, CLOSE_RANGE_CLOEXEC)` and then clearing CLOEXEC on fd 3 leaves exactly fds 0, 1, 2 and 3 in the exec'd child. A stray fd opened without CLOEXEC before the clone is gone. | `fds=0,1,2,3` | High |
| E10 | A **static-pie** binary builds from the stock `x86_64-unknown-linux-gnu` target with `-C target-feature=+crt-static`. No musl target is needed, and none is installed on hueyj. Size: 1.47 MB static versus 0.51 MB dynamic. | `file` rows | High |

**Implications:** the spawner's child can lock down filesystem and network access (Landlock) and the syscall surface (seccomp) **before** exec, provided the executable is **static** or its loader and libraries are covered by rules. One seccomp install is about 3% of FP-27's 15 ms† budget.

## 3. Linux: identities across the PID namespace (OsPeer, `SCM_CREDENTIALS`, `SO_PEERCRED`)

The socketpair was made by the parent before the clone; fd 3 in the child is one end of it.

| # | Who looks | What it sees | Evidence | Conf. |
|---|---|---|---|---|
| E11 | Parent receives the child's record (`SCM_CREDENTIALS`) | The child's **outer pid** (equal to `clone3`'s return value) and the real uid/gid (1000), in every namespace case | `parent got … creds[pid=<outer> uid=1000]` | High |
| E12 | Child in a **PID namespace** receives the parent's record | **pid 0**, uid and gid 65534 | `child-saw … creds[pid=0 uid=65534 gid=65534]` | High |
| E13 | Child in a PID namespace reads `SO_PEERCRED` on its end | **pid 0**, uid 65534. The creator is the parent, which is not visible in the child's namespace | `peercred=pid=0 uid=65534` | High |
| E14 | Child with a user namespace but **no** PID namespace | The parent's real pid, uid 65534 (`SO_PEERCRED` and `SCM_CREDENTIALS`) | `user only`, `user+net`, `user+mnt …` | High |
| E15 | Child's own `getpid` | 1 in a PID namespace | `pre-exec getpid=1`, `ready pid=1` | High |

**Implications:**
- **The broker's side works unchanged:** it gets the child's true outer pid from every record, whoever created the child.
- **The child's side cannot attest anyone by pid.** Every process outside its PID namespace reads as pid 0, siblings included: they each have their own namespace. D3's bootstrap check (`creator_pid`, which refuses pid 0) therefore fails, and so does D6's "writer only from the broker" check by pid.
- **The child must attest the broker by the channel, not the pid** (design rule E-R6).

## 4. Linux: the broker's handle on a grandchild

| # | Finding | Evidence | Conf. |
|---|---|---|---|
| E16 | `clone3(CLONE_PIDFD)` in the spawner returns a pidfd. Sent over `SCM_RIGHTS` to the broker, which is not the parent, it gives the broker the child's pid from `/proc/self/fdinfo/<fd>` (`Pid: <outer>`, `NSpid: <outer> 1`). | `pidfd via SCM_RIGHTS` row | High |
| E17 | The broker's `poll` on that pidfd reports POLLIN when the child exits (here 300 ms, the child's sleep). `waitid(P_PIDFD)` from the broker gives ECHILD, so only the spawner can reap. `pidfd_send_signal(…, 0)` works. | same row | High |

**Implications:** the spawner hands the broker a pidfd and the outer pid for every child. The broker gets a race-free identity (no pid reuse), exit notice and kill, all through the pidfd. The spawner reaps.

## 5. Linux: spawn-to-ready (warm, median of 20)

| Path | Median | Max |
|---|---|---|
| No namespaces, dynamic child | 0.97 ms | 1.15 ms |
| User namespace | 1.06 ms | 1.15 ms |
| User + PID | 1.09 ms | 1.33 ms |
| User + net | 1.62 ms | 1.75 ms |
| User + mount, fd opened in child | 1.17 ms | 1.39 ms |
| **Full: user + PID + net + mount, Landlock with exe rule, seccomp 202, static child** | **2.16 ms** | **2.38 ms** |

A network namespace adds about 0.55 ms, and seccomp about 0.5 ms. The full path is about 14% of FP-27's 15 ms† at the median. This is warm and on one host: FP-27 itself is measured cold, on the reference devices (G).

## 6. macOS and Windows: what the existing code and FP-M0 already show

| # | Finding | Source | Conf. |
|---|---|---|---|
| E18 | macOS: D4's `sys::spawn` uses `POSIX_SPAWN_START_SUSPENDED` with `POSIX_SPAWN_CLOEXEC_DEFAULT` and `posix_spawnattr_setspecialport_np` (one bootstrap port per child), reads the child's audit token while it is suspended, then resumes it. A child whose token cannot be read is killed and reaped (D4 finding 7). | `crates/hg-substrate-os/src/ipc/macos/sys.rs` | High |
| E19 | macOS: sandboxed spawn-to-ready on FP-M0's path is p50 4.42 ms and p99 11.20 ms (100 runs, 23 October). The SBPL apply adds about 9.5 ms at p99, over the 10 ms† spare gate, so the rule is to keep a spare. | `measurements/milestones/fp-m0/results/decision-log.md` | High |
| E20 | Windows: `sandbox::windows` builds the AppContainer (no capabilities), the restricted lockdown token and a job with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. D5's `launch` adds the handle list (one inherited bootstrap pipe, B4) and one AppContainer SID per live child (item 10). D5's launch does **not** yet put the child in a job: the two paths are separate today. | `sandbox/windows.rs`, `ipc/windows.rs` | High |
| E21 | Windows: AppContainer spawn-to-ready on FP-M0's path is p50 7.77 ms and p99 10.42 ms (100 runs). The Rust child needed the loader fix for 0xC0000142. | FP-M0 decision log | High |

## 7. Questions this round could not settle

- **E-Q1:** Does a host with an AppArmor profile granting `userns` (and `attach_disconnected`) give the full floor: id maps, an empty root, a pre-opened fd exec? Installing a profile needs root on hueyj. This is a question for the owner.
- **E-Q6:** Cold spawn-to-ready on Linux needs `drop_caches` (root). G measures FP-27 cold on the reference devices.
- macOS and Windows each need **one combined run**: D4's or D5's launch with the FP-M0 sandbox and Izcal's own bootstrap, measured spawn-to-ready. That is a phase E deliverable (FP-27 hook), not a probe.

## 8. Round 2 (2026-10-26): the probes E-R4 and E-R6 rest on

Probe: `probes/linux/src/bin/er46.rs`, run as a static build on hueyj (same host and settings as above) under the light booking b-3320da. Log: `probes/logs/linux-hueyj-er46-2026-10-26.log`, with the kernel's AppArmor lines for the run.

| # | Finding | Evidence | Conf. |
|---|---|---|---|
| E22 | **A directory fd does not help either.** The parent opens the build directory `O_PATH | O_DIRECTORY` before the clone. Then `openat(dir_fd, name, O_PATH)` succeeds in the child, but `execveat` of the result fails with EACCES once the child has a new mount namespace, with or without PID and net namespaces. `openat(…, O_RDONLY)` fails as well. The kernel logs `operation="exec" … "Failed name lookup - disconnected path"` and `operation="open" … "disconnected path" name="home/joe/izcal-eprobe/er46"`. With a user namespace only, the same `openat` plus `execveat` works. | `E-R4 …` rows | High |
| E23 | **Grandchild credentials.** Broker → spawner (plain child) → grandchild in new user and PID namespaces: <br>• The grandchild reads **pid 0, uid 65534** in `SO_PEERCRED` on both the broker-made pair and the spawner-made pair, and in `SCM_CREDENTIALS` from both the broker and the spawner. <br>• The broker and the spawner both read the grandchild's `SCM_CREDENTIALS` as its **outer pid** and uid 1000, equal to `clone3`'s return value. <br>• The broker's `SO_PEERCRED` on its own end of a pair it made names **the broker itself**: a socketpair's peer credentials are its creator's on both ends. | `E-R6a` rows | High |
| E24 | **A pidfd's inode is not a generation on 6.8.** Pidfds of two live children have the same `st_dev`/`st_ino` (15/69), and a pidfd made after both exited has it too. `/proc/self/fd` shows `anon_inode:[pidfd]`: every pidfd shares one anonymous inode. Because the inode is identical across distinct processes, there was no need to churn for pid reuse; the question is answered without it. Newer kernels were not measured. | `E-R6b` rows | High |

**Implications:**
- **E-R4:** under the AppArmor userns restriction, **no form of exec by handle works in a new mount namespace**: pre-opened file fd (E4), procfs reopen (E4b) and directory fd (E22). On those hosts the child gets no new mount namespace. That costs nothing, since E2 already showed it can't mount anything there.
- **E-R6:** `SO_PEERCRED` identifies only the creator, so a pair's peer credentials identify neither side's later holder. Only `SCM_CREDENTIALS` identifies a sender, and only from outside the sender's PID namespace. The generation has to come from somewhere other than the pidfd inode.
