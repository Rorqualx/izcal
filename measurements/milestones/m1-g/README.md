# M1 phase G: the renderer-skeleton row, measured

Measured 26 October 2026 on the Mac and rig253; hueyj's rows are taken in its exclusive window from 20:37 (below). This is the M1 plan's phase G: the launched renderer skeleton (`hg-renderer-main`, started by the browser library through each OS's full launch path and sandbox) at rest after its round trips, on three OSes under exclusive bookings. The network partition service (`hg-net-main`) is measured beside it, the same way.

## Setup

| | |
|---|---|
| Build | `m1/integration`, `--release`; Mac at `04ca8a3`; rig253 at `406b473` plus `04ca8a3`'s `hg-launch-tool` (the hold mode; the measured children are unchanged by it) |
| Mac | Huey the Sixth: Apple M6 (Mac18,5), 12 cores, 32 GB, macOS 27.0.1 (26A434); DC-16, AArch64 |
| rig253 | Huey the Sojourn: AMD Ryzen AI Max+ 395, 32 threads, 28 GB, Windows 11 Pro build 26300; DC-16, x86-64 |
| hueyj | Huey the Destroyer: Linux x86-64, 8 threads, 31 GB, Ubuntu 24.04; DC-16 (rows pending) |
| Sandbox tier, by name (E-R5) | macOS `MacosSbpl` (deny-default SBPL); Windows `WindowsAppContainer` (AppContainer, lockdown token, job: the M1 subset, `windows-m1`); Linux `linux-restricted-userns` until the AppArmor profile is installed and the full tier probed (E-Q1), never claimed before |
| Tool | `hg-launch-tool hold SECS` (round trip, then idle with the children alive) and `measure-spawn N` |
| Harness | `harness/` here: `g-probe-mac.c`, `g-run-mac.sh`, `g-run-linux.sh`, `g-run-windows.ps1`, `g-spawn.sh`, `g-spawn-windows.ps1`; FP-M0's method (counters read from outside, by process) |

**Method, per FP 6.2 and 8.1.** Charged memory is FP 6.2's per-OS metric: macOS `phys_footprint`, Linux `RssAnon + VmSwap`, Windows commit charge (`PrivateMemorySize64`). Each run samples at 1 Hz over a 60 s quiet window; **settled** is the median of its last 30 s; the row is the **median of 5 runs**. Wakeups are counted over the same window with FP-M0's counters (macOS interrupt wakeups, Linux voluntary plus involuntary context switches over all threads, Windows raw thread context switches; on Windows the window starts 40 s after `ready`, as FP-M0 did, so the thread set is stable). Mappings (FP 6.20.6) are the process's own map entries on macOS (`PROC_PIDREGIONINFO`), VMAs on Linux (`/proc/PID/maps`) and allocations (VADs, distinct `AllocationBase`) on Windows. FP-27 is the time from the launch request to the accepted `ready`, on the substrate clock, 20 runs.

**Cold.** FP-27 asks for cold runs after reboot. On Linux the owner runs `../m1-e/fp27-cold.sh` (page cache dropped before each run, sudo; owner's decision E-Q6). On the Mac and rig253 a reboot per run was not possible in this session, so the cold-side row uses E-R12's fallback, a **fresh build directory**: each run copies the binaries into a new directory first. The copy is in the page cache, so this is not cold after reboot; it does include the first-exec costs of a new file (code-signature validation on macOS).

## Results: the renderer skeleton

| Target (DC-16) | macOS | Windows | Linux | Spec |
|---|---|---|---|---|
| Charged memory, settled, median of 5 | **4,656 KiB** (runs: 4,688, 4,672, 4,640, 4,640, 4,656) | **1,024 KiB** (1,020, 1,024, 1,036, 1,028, 1,020) | pending | FP-2 ≤ 4 MiB† |
| FP-2 ≤ 4,096 KiB | **FAIL** (+560 KiB) | **PASS** | pending | |
| Idle wakeups, 60 s | **0.000/s** (5 of 5 runs; package-idle 0.000/s) | **0.000/s** (5 of 5; 6 threads) | pending | FP-18 ≤ 1/s† whole browser |
| FP-18 | **PASS** | **PASS** | pending | |
| Mappings | **83** map entries (5 of 5) | **45** allocations (5 of 5; 143 regions) | pending | FP 6.20.6 ≤ 64† |
| ≤ 64 | **FAIL** | **PASS** | pending | |
| Spawn-to-ready, warm, median of 20 | 4.726 ms (min 4.623, max 6.536) | 28.630 ms (26.089–33.803) | pending | FP-27 ≤ 15 ms† |
| Spawn-to-ready, fresh build directory, median of 20 | 25.727 ms (21.458–84.326) | 98.129 ms (94.831–111.640) | owner's cold run | FP-27 ≤ 15 ms†, cold |
| FP-27 (cold side) | **FAIL** (fresh-directory fallback; not after reboot) | **FAIL** (warm already over) | owner's cold run | |

**After the root-cause fixes (below, same day):** macOS 4,176 KiB (still over), 83 map entries (unchanged); Windows warm spawn-to-ready 5.4 ms (PASS), fresh-directory 74.7 ms (still over).

The network partition service beside it: macOS 4,656 KiB settled (4,736, 4,656, 4,752, 4,656, 4,656), 0 wakeups, 83–86 map entries, warm spawn 4.699 ms, fresh 25.425 ms; Windows 1,036 KiB (1,036, 1,028, 1,040, 1,028, 1,048), 0 switches, 45 allocations, warm 31.275 ms, fresh 103.634 ms. Raw rows: `results/2026-10-26-{mac,windows}-*.tsv`.

**Reading.**
- **macOS memory.** FP-M0's empty sandboxed process was 4,096–4,144 KiB. The renderer skeleton, with the substrate's thread pools, the IPC transport, its bootstrap channels and the renderer library, is 4,656 KiB: about 560 KiB of Izcal line items over a floor that alone already sits at FP-2's 4 MiB. FP-2's rule ("the measured floor plus Izcal's line items", a decision-log entry between that and 4 MiB) needs these line items itemised; that is a decision for the owner, not a pass.
- **macOS mappings.** 83 entries against ≤ 64†. vmmap reports 1,931 regions for the same process because it also counts every region inside the dyld shared-cache submap; the 83 are the process's own entries. Which count FP 6.20.6 means on macOS is not fixed by the spec text ("VM metadata (a VMA on Linux, a VAD on Windows)"); both are recorded.
- **Windows spawn.** FP-M0's empty AppContainer process reached `ready` in 7.774 ms p50; the launched renderer takes 28.6 ms warm. The launch path adds a per-child AppContainer profile and SID, the job, the handle list, a substrate start in the child and the `ready` handshake over IPC. Where the 21 ms go is not measured yet; that is the next probe, not a guess.
- **macOS spawn.** 4.7 ms warm passes 15 ms; the first exec of a newly written binary costs about 20 ms more (code-signature validation), which is what the fresh-directory runs measure.
- **Wakeups.** Zero on both OSes for both children, as FP-M0's substrate-at-rest row predicted.

## Root causes of the three misses (26 October, afternoon)

Each miss was taken apart by measurement on the machine that showed it, under exclusive bookings (`rig check` before every timing run; the lines are in `results/2026-10-26-rootcause/{mac,windows}/evidence.txt`). Raw outputs are in `results/2026-10-26-rootcause/`; the probes are in `harness/` (`g-sbx-cost-mac.c`, `g-sbx-exec-mac.c`, `g-regions-mac.c`, `g-selfmap-mac.c`, `g-classify-regions-mac.py`, `g-phases.py`, `g-fresh-*-windows.ps1`, `g-conhost-windows.ps1`) and in `hg-launch-tool measure-phases` (feature `measure-launch`). Fixes: `659c317` (macOS), `b3a821e`, `9a99e44` (stamps only) and `94fe3f7` (Windows). No target was changed.

| Miss | Before | Cause, measured | After the fixes | Target |
|---|---|---|---|---|
| macOS FP-2 | 4,672 KiB | +464 KiB: CoreFoundation and Security loaded in every child; the rest is a floor of 4,096 KiB (2.8 MB of it is the sandbox's own) | **4,176 KiB**, still over | ≤ 4,096 KiB†; see the rule below |
| macOS mappings | 83 | 53 OS entries before any Izcal code; 25 from 6 threads; 5 the image | **83**, unchanged | ≤ 64† |
| Windows FP-27, warm | 29.0 ms | a conhost.exe per child (console subsystem): 18 ms; profile registration in the launch: 5–9 ms | **5.4 ms** | ≤ 15 ms† |
| Windows FP-27, fresh directory | 99.6 ms | first execution of a newly written .exe: 69 ms in `CreateProcess`, outside the launch path | **74.7 ms**, still over | ≤ 15 ms†, cold |

### Miss 1: macOS FP-2 (charged memory)

**Is the method the spec's?** Yes. FP 6.2 defines macOS charged memory as "`phys_footprint` (dirty + compressed, including owned graphics memory) minus owned graphics memory"; the sampler reads `ri_phys_footprint` and the children own no graphics memory. "Settled" is the "median over the last 30 s† of a 60 s† quiet period", median of 5 runs, as G does. **The comparison is where the spec reads two ways.** Section 8.2's row says "Empty site process (process part + first agent) ≤ 4 MiB† — FP-2"; Section 6.3 says "FP-2's ship blocker is then set as that measured floor plus Izcal's own line items (everything above except the margin), recorded in the decision log item by item, never as a lump", and FP-M0's decision log recorded the macOS floor as 4,096 KB "plus the line items as they land". Under 6.3's rule the renderer is now the floor plus 80 KiB of line items (below), to be itemised; under 8.2's flat 4 MiB it fails.

**Breakdown by region** (`footprint` categories, KiB; one process each, held idle; `results/2026-10-26-rootcause/mac/`):

| Category | FP-M0's empty process, built at `25f7591`, run today | Empty process at `709ff37` | Renderer at `709ff37` | Renderer, fixed |
|---|---|---|---|---|
| Malloc Small (dirty) | 2,736 | 2,848 | 2,944 | 2,800 |
| Malloc metadata | 480 | 480 | 480 | 480 |
| Page tables | 288 | 336 | 352 | 288 |
| Image and library data (`__DATA*`, `__TPRO_CONST`, `__OBJC_RW`, dyld cache) | 353 | 592 | 592 | 353 |
| Stacks | 64 | 96 | 144 | 112 |
| Malloc Tiny | 64 | 96 | 96 | 64 |
| Untagged, deferred reclaim, nano, OS alloc, activity tracing | 112 | 112 | 80 | 80 |
| **phys_footprint** | **4,096** | **4,560** | **4,688** | **4,176** |

- **The floor, not ours.** A one-thread C program that only calls `sandbox_init_with_parameters` with the renderer's profile goes from 1,200 to **4,032 KiB**; the call leaves **2,832 KiB** of freed memory dirty in the default malloc zone (`vmmap`: Malloc Small 2,720 KiB dirty with 16 KiB in use). `malloc_zone_pressure_relief` returns 0 bytes, with or without a live block in that region (`g-sbx-cost-mac.c`). FP-M0's empty process, rebuilt from `25f7591` and run today, is exactly **4,096 KiB**: the floor has not moved.
- **What changed since FP-M0, and the fix.** The launch path (phase E) added the broker's code-directory-hash check through Security and CoreFoundation, linked into `hg-substrate-os`, which every child links. Each sandboxed child loaded both frameworks and ran their initialisers (Foundation, Swift and XPC objects appear in the renderer's `heap`), never calling them: **+464 KiB** in the empty process (4,096 → 4,560), +496–512 KiB in the C probe. `659c317` loads them with `dlopen` in the broker only; the check is unchanged (`a_replaced_executable_never_runs` passes) and a test reads the children's load commands.
- **Ours, after the fix: +80 KiB over the floor.** Stacks +48 KiB (6 threads: main, two general workers, one may-block worker, the timer thread and the Mach IO thread, against FP-M0's 3; FP 6.3 and FP-32 say an idle site process has **two**); Malloc Small +64 KiB; untagged and activity tracing −32 KiB. The D4 IO thread's 80 KiB receive buffer is the one live block in the 4 MiB Malloc Small region the sandbox call freed (`heap -addresses`), so libmalloc cannot return that region as empty. FP-M0's empty process left it empty, and libmalloc returned it once (to 2,208 KiB) in one of four processes held 5–10 minutes; in one of G's ten runs today both children dropped to 2,496 and 2,480 KiB at the same moment, which points to a system-wide event, not to either process. Neither can be relied on. No D6 region appears in the idle footprint, and macOS binaries are not statically linked.
- **Before/after, G's harness** (5 runs, exclusive booking b-dfc312): renderer 2,496, 4,656, 4,672, 4,688, 4,688 → median **4,672**; after `659c317` 4,176, 4,176, 4,176, 4,192, 4,224 → median **4,176 KiB**. The network partition service: 4,672 → 4,144. Wakeups 0.000/s throughout.
- **Why 4 MiB flat cannot be met by a fix that keeps the rules.** The sandbox call's 2.8 MB is in-process compilation of the SBPL profile. Measured: a process that applies the profile and then `exec`s keeps the sandbox (`/etc/hosts` denied) and starts the new image at **1,184 KiB**. But dyld's start in the new image aborts unless the profile also grants `process-exec` of the binary and `file-read*` on `/`, `/System`, `/usr`, `/private/preboot` and `/dev`, plus all `file-read-metadata`; a second `sandbox_init` to narrow it again is refused (EPERM) and costs the 2.8 MB again (`g-sbx-exec-mac.c`). That widens the renderer's sandbox for its whole life, which is a security decision, not a fix. **Stopped here: evidence for the owner.**

### Miss 2: macOS mappings (FP 6.20.6)

**Is the method the spec's?** The spec says "An idle empty site process holds ≤ 64† mappings … Untouched reservations cost VM metadata (a VMA on Linux, a VAD on Windows)" and names no macOS unit. G's count (`PROC_PIDREGIONINFO` walk) equals the task's own top-level VM map entries: a process counting itself both ways gets 58 = 58, of which 19 are submap entries (`g-selfmap-mac.c`). That is the macOS VM metadata, so the method is consistent; the 1,931 of `vmmap` count the contents of the shared submaps as well.

**Breakdown** (`g-classify-regions-mac.py` over each process's entries):

| Kind | C, sandboxed, 1 thread | FP-M0 empty (3 threads) | Renderer (6 threads), before and after |
|---|---|---|---|
| dyld shared region (submap entries and their private copy-on-write data) | 31 | 31 | 31 |
| its guard reservations | 2 | 2 | 2 |
| malloc and libSystem (zones, metadata, reservations) | 20 | 20 | 19 |
| thread stacks and guards | 2 | 6 | 12 |
| Rust's alternate signal stacks and guards | 0 | 7 | 13 |
| the executable image | 5 | 6 | 5 |
| other | 1 | 1 | 1 |
| **Total** | **61** | **73** | **83** |

- **Ours:** 30 (25 from 6 threads, 5 from the image). **The system's:** 53, present in any process before Izcal's code runs. Unloading CoreFoundation and Security changed no top-level entry (83 before and after).
- **Not reachable under this count.** 53 OS entries plus the image (5) plus FP 6.3's two threads (about 4 entries each with their signal stacks) is about 66 > 64. Cutting the renderer from 6 threads to FP-32's two would remove about 16 entries, to about 67. **Stopped here: the owner decides whether the dyld shared region's entries count toward ≤ 64 on macOS (they are the same 33 in every process, and none is an Izcal reservation), or whether the macOS figure is set from the measured floor, as 6.3 sets FP-2.**

### Miss 3: Windows FP-27 (spawn-to-ready)

**Phases, measured.** `hg-launch-tool measure-phases` (feature `measure-launch`, `9a99e44`) stamps QPC in the broker at each launch step and in the child at each start step; the child publishes its stamps as its main thread's description, so the launch gains no handle or message. Renderer, median of 20 (first run dropped), ms:

| Step | `709ff37` (console) | + GUI subsystem | + profiles ahead (final) |
|---|---|---|---|
| Register the AppContainer profile | 9.42 | 5.21 | 0.00 |
| Lockdown token, job, three pipe pairs | 0.11 | 0.10 | 0.10 |
| `CreateProcessAsUserW` (suspended) | 1.69 | 1.63 | 1.67 |
| Job assignment, handle gives, broker worker, Hello, resume | 0.10 | 0.09 | 0.09 |
| Resume → the child's Rust entry (loader, CRT) | **15.67** | 3.25 | 2.99 |
| Child: Untrusted, link, entropy key, executor threads | 0.38 | 0.36 | 0.37 |
| Child transport start (Hello read, IO thread) | **4.77** | 0.09 | 0.08 |
| Child sends `ready` → broker accepts | 0.08 | 0.08 | 0.06 |
| **Spawn-to-ready** | **32.56** | **10.89** | **5.46** |

- **Cause 1: a console per child.** `hg-renderer-main.exe` and `hg-net-main.exe` were console-subsystem executables (PE `Subsystem` 3). Started with `CREATE_NO_WINDOW`, each got its own `conhost.exe`, observed as the child's child: 944 KiB private and 6.2 MB working set each (`windows/conhost-observation.txt`), outside G's per-process row. FP-M0 found and fixed exactly this for `fp-m0-empty` on 23 October (GUI subsystem); the M1 mains never got it. The console cost the loader phase 12.4 ms and the transport start 4.7 ms (the step medians do not sum exactly to the total median). Fix `b3a821e`: `windows_subsystem = "windows"` on both mains; a test reads the PE header.
- **Cause 2: profile registration in the launch.** `CreateAppContainerProfile` took 5.2–9.4 ms in today's runs of the same code. It costs 4–6 ms even when the profile already exists (it then returns "already exists"; measured by making the profiles first, or reusing one), so only keeping the SID from an earlier registration removes it. Fix `94fe3f7`: two fresh profiles registered at broker start and refilled on a thread after each launch takes one; a launch that finds none registers its own. Every child still gets a profile and SID no process had before it.
- **FP-M0's path, compared.** FP-M0's 7.774 ms p50 used a GUI-subsystem child, one persistent profile registered once (`izcal.fp-m0.empty`, never in the launch), two threads and a ready line on stdout. The M1 launch now matches it step for step and lands at 5.4 ms.
- **Before/after, G's harness** (`g-spawn-windows.ps1`, warm, median of 20, booking b-2ac71e): Renderer **29.04 → 11.42 (GUI) → 5.69 (profiles ahead) → 5.37** (final code); NetworkPartition **33.03 → 12.22 → 6.12 → 6.03**. Memory after both fixes (`g-run-windows.ps1`, 5 runs): renderer 1,036 KiB median (1,036, 1,028, 1,028, 1,036, 1,048), 0 switches, 45 allocations, 6 threads, as before; the conhost per child is gone.
- **D5 and reuse.** D5's rule is one AppContainer SID per **live** child. The transport enforces it with `sid_in_use`, which counts reserved SIDs, live children's SIDs and `creator_sids`; the last outlives a dead child while any channel it created is alive (item 5). So reusing a dead child's SID is refused until all its channels are gone, and within that the letter of D5 allows reuse. Whether a profile's storage (its `AC` folder, its registry key, its named objects) can carry anything from one holder to the next was **not probed**, because reuse gained nothing over fresh profiles made ahead: the loader phase was 14.2 ms reused against 13.7 ms fresh (console build), and the registration call costs the same either way. The fix reuses nothing. One reuse path did exist and is closed: profile names were per launcher, and "already exists" was taken as success, so two launchers in one process, or a broker reusing a crashed broker's pid, could hand a child a profile an earlier child had used. Spares are now numbered per process and must be newly registered; a stale one is removed and registered again empty (`a_stale_profile_is_registered_again_empty`).
- **The fresh-directory runs (E-R12's cold fallback) are still over: 74.7 ms.** With a fresh copy, `CreateProcessAsUserW` alone takes **70.3 ms** (1.7 ms warm); every other step is as warm. A Defender on-demand scan of the three copies first (52–94 ms per run) changes nothing (69.0 ms). One plain execution of each new file first, unsandboxed (it exits at argument parsing), brings the sandboxed launch to **6.74 ms** (`CreateProcess` 3.18 ms). So the 69 ms is Windows' first execution of a newly written executable, paid once per file, whatever launches it. FP-27's cold case is "after reboot"; the fallback measures a new file's first execution, which an installed browser pays at its first run after install or update. Whether a reboot brings any of it back needs a reboot on rig253, which this session did not do. **Stopped here: a method question for the owner, not a target change.**

**Also found.**
- **Windows profiles leak when a broker exits** (since found and fixed; see "Follow-up: Windows AppContainer profiles" below). On rig253, every `measure-spawn`, `hold` and `round-trip` run left its children's profiles registered: about 166 `izcal.izk1.*`/`izcal.izk4.*` from G's and today's base runs are still there. The launcher's `Drop`, which unregisters them (shown by `each_launch_takes_a_fresh_profile_made_ahead`), evidently does not run when the tool ends; the cause is not measured. With the profiles registered ahead, each broker also leaves its two spares. This session's 514 leftovers (`izcal.<pid>.<n>`, `izcal.pre.*`, `izcal.measure.*`) were unregistered; others' were left.
- **A Windows test fails at `709ff37` too:** `ipc::windows::tests::a_child_dying_mid_copy_stalls_no_one_and_keeps_no_copy` (`kept` 16, expected 8), unchanged by these commits.
- **Evidence notes.** `rig check` on rig253 intermittently reported "No route to host" while ssh worked; every run kept here followed an OK line (retried 12 s apart). The first phase pass (`phases-console-unchecked.tsv`) followed a failed check and is kept for its shape only. About 6 s of compiling on rig253 overlapped the Windows memory run; an idle child's commit and context switches do not depend on it, but it is recorded.

## Follow-up: Windows AppContainer profiles leaked when a broker ended

**Cause, measured.** Not drop order, not process exit, and no static launcher: a **reference cycle**. The browser's `Inner` holds each child's serving; the serving's handlers (`HostSide`, `on_end`) hold `Arc<Inner>`; `Running` had no `Drop`. In an instrumented build (macOS and rig253), `Inner` had 7 strong references when `Running` was dropped (`Running`, two handlers per child, two pending ready deadlines) and none was ever released, so neither `Inner` nor the launcher was dropped, even within one process: a single `measure-spawn 21` left `izcal.<pid>.1` to `.83`. Taking the children out of the browser at that point brought `Inner` to 3, then to 0 once the deadlines fired, and the launcher dropped with the substrate. `hg-browser-main` itself never ends normally: it parks until it is ended, and a crash runs no `Drop`.

**Fixes.**
- `9bbddc8`, `acc86c2` (browser): an `Owner` token shared by `Running`'s clones ends the browser when the last one goes. It marks the state closed (nothing more is launched, including a launch already in flight, whose child is killed) and takes both children, which closes their channels and releases the handlers. The deadline and relaunch tasks hold `Weak<Inner>`. DST: behaviour during a run is unchanged (steps, virtual time and crashes equal HEAD's; with the token disabled every trace equals HEAD's). Traces now include the children's channels closing at shutdown, so their hashes change; they stay identical across runs and across the three OS models, and every scenario's exit status is unchanged.
- `731d5db`, `553dc69` (Windows launcher): profiles are named `izcal.l.<broker pid>.<broker creation time>.<n>`. At start, before registering its spares, a launcher unregisters a profile of this scheme only if no running process has that pid **and** creation time (gone, reused or exited). A dead broker's children ended with it through their kill-on-close jobs. Live brokers' profiles and every other name are never touched. Closing the spares joins the refill thread, whose handle is stored under the lock `close` takes.

**Tests** (rig253): `the_sweep_removes_only_profiles_of_brokers_that_are_gone` (a reused pid and a gone pid removed; this process's, another scheme's and a malformed name kept); `a_broker_that_ends_leaves_no_profile`, which fails on HEAD's `hg-browser` with all four of the run's profiles left and passes now. 20 of 20 round trips leave no profile; the launch and tool tests pass 5 of 5. One batch of five runs, right after the negative control, left four profiles each time; three later batches (15 runs) and the instrumented runs (every unregister `S_OK`) did not reproduce it. Its cause is not known.

**G's leftovers removed.** Listed first (`windows/old-scheme-profiles-deleted.txt`, `harness/g-old-profiles-windows.ps1`): 160 profiles of the launcher's first scheme, `izcal.izk<k>.<pid>.<n>`, each deleted only because no process holds its pid now (the script would also accept a pid whose holder started after the profile's folder was created). All 160 unregistered. Kept, as another session's or another scheme's: 23 to 25 `izcal.d5test.*` (the D6 owner's tests were creating and removing them during the cleanup), 3 `izcal.d6probe.*`, `izcal.fp-m0.empty`. This session's own 514 experiment leftovers were removed earlier.

## Follow-up: six threads where the spec says two

**Every thread in an idle renderer child** (macOS by `sample` and per-region `vmmap`; Windows by `GetThreadDescription`; Linux from the code only, as hueyj was not used):

| Thread | Created by | Why it exists | Stack, reserved / touched (macOS) | Does the idle renderer need it? |
|---|---|---|---|---|
| main | the OS; `main` runs `os_child_substrate` and the renderer, then parks in `Running::wait` | process entry | 8 MiB / 32 KiB | exists anyway; idle after start-up, so it could run the IO loop or a worker instead of parking |
| `hg-worker-0`, `hg-worker-1` | `OsExecutor::new`, `Threads.general` (2 in `hg-renderer-main` and `hg-net-main`) | run sequences: the renderer's `control` sequence, where IPC deliveries are handled | 2 MiB / 16 KiB each | one, to run the sequence; the second only adds parallelism, which no M1 child uses |
| `hg-may-block-0` | `OsExecutor::new`, `max(may_block, 1)` | `MayBlock` sequences | 2 MiB / 16 KiB | no: nothing in a child posts `MayBlock` work (ast-grep finds `MayBlock` only in the executor and its test) |
| `hg-timers` | `OsExecutor::new` | delayed posts | 2 MiB / 16 KiB | not at M1: no delayed post in the child path (the ready deadline is the broker's); the broker relies on it for that deadline |
| `hg-ipc-io` | each OS transport's start (`ipc/{linux,macos,windows}.rs`) | the D3/D4/D5 IO thread: receives on the epoll set, port set or completion port with no timeout (0 idle wakeups); acceptance and identity checks run on receipt | 2 MiB / 16 KiB | yes: it carries the D3/D4/D5 security rules |

Every one comes through the substrate (CS-AS-3) with Rust's default stack size; none sets one. On Windows each child also starts with 3 unnamed OS threads (the loader's pool, FP-M0's "three the process did not create itself"); they exit on their own within 45 s (9 threads at 6 s, 6 at 45 s; `windows/threads-at-{6s,45s}.txt`). Broker-only threads, never in a child: `hg-ipc-own` and one `hg-ipc-child` per child (D5 M8, "relay work on its own broker thread"), `hg-ipc-copy` (D6), `hg-profile-spares` (94fe3f7).

**Against the spec and the plan.** FP 6.3 and FP-32: "An idle empty site process has two threads", the script thread (256 KiB reserved) and the control thread (128 KiB); the worker pool is "created on the first parallel phase … each worker exits after 10 s† idle"; on Windows, stacks are a reservation with a small initial commit. CS-AS-3: the substrate owns every thread, and "thread count after startup equals the substrate's" (`OsExecutor::thread_count` exists for that check). The plan gives each transport one IO thread (D3, D4, D5) and gives the broker its relay threads (D5 M8); it sets no executor pool for children, and the executor's fixed pool (2 general, at least 1 may-block, 1 timer) comes from the mains' `Threads`, not from a rule. Measured Windows commit (1,036 KiB for 6 threads) shows the stacks are reserved, not committed.

**What the idle renderer skeleton needs: two threads.** `hg-ipc-io` (FP's control thread, carrying D3/D4/D5's rules) and one thread to run its sequences (FP's script thread, which can be `main`). No thread is removed in this change.

**What removing each would save, measured** (one general worker fewer, `Threads.general` 2 → 1, 3 runs each with G's harness; `{mac,windows}/threads-{6,5}-memory.tsv`):

| Per thread | macOS | Windows |
|---|---|---|
| Charged memory | 16 KiB of stack, measured exactly; renderer median 4,192 → 4,144 KiB, within run-to-run noise (4,080–4,160) | about 40 KiB of commit: renderer median 1,036 → 1,000 KiB, network 1,036 → 984 |
| Mappings | **4 map entries** (stack, its guard, Rust's alternate signal stack, its guard): 83 → 79 in every run | 1 allocation (45 → 44 in two of three runs) |

| Thread | What removing it takes | Saves (macOS / Windows) |
|---|---|---|
| `hg-worker-1` | `Threads.general: 1` in the two child mains (configuration) | 16 KiB, 4 entries / ~40 KiB, 1 allocation |
| `hg-may-block-0` | the executor creating may-block workers on the first `MayBlock` sequence | same |
| `hg-timers` | the executor serving delayed posts from a timed wait in a worker, or creating the timer thread on the first delayed post (the broker keeps its use) | same |
| `main` parked | `main` running the worker loop (or the IO loop) instead of parking; the thread itself stays | the work of one thread, so one of the others goes: same |
| `hg-ipc-io` | keep: D3/D4/D5's receive-side rules and zero idle wakeups live there | — |

Going from 6 threads to FP-32's two would save about 64 KiB and 16 map entries on macOS (to about 4,112 KiB and 67 entries, still over 4,096 KiB and 64; see misses 1 and 2), and about 160 KiB of commit and 4 allocations on Windows.

## Evidence: bookings and checks

Every timing run began with `rig check --kind exclusive --session m1-substrate`; the lines are in `results/2026-10-26-{mac,windows}-evidence.txt`:
- Mac, booking b-b3f490 (exclusive): `rig check OK: local … load 0.29/cpu … 2026-10-26T14:50:24-06:00` (memory), `… 0.20/cpu … 14:56:46` (warm spawn), `… 14:56:46` (fresh spawn).
- rig253, booking b-6614c0 (exclusive): `rig check OK: rig253 … CPU% 1% … 2026-10-26T15:00:36-06:00` (memory), `… CPU% 2% … 15:13:18` (warm), `… 15:13:20` (fresh).

**Process note.** At 15:14 a 12-minute heavy booking on hueyj was refused (it overlapped the session's own Kani booking b-33275b from 15:27 with its grace). The release build of `m1/integration` on hueyj and one 6-second `hold` (to see the children's process names for the Linux sampler) ran anyway, without a booking. Neither was a timing run, and nothing measured here comes from it.

## Linux (hueyj), pending

hueyj is booked exclusive for the session from 20:37 (b-fe3c27), after the session's Kani bookings. In that window: `g-run-linux.sh` (memory, wakeups, VMAs, 5 runs), `g-spawn.sh warm` (FP-27 warm, 20 runs, through the spawner), each after `rig check`; then the owner's cold runs (below). The tree is `~/izcal-m1int-g` on hueyj (`m1/integration`, release binaries built).

## For the owner

1. **Linux cold FP-27:** in the hueyj exclusive window, on the Mac: `measurements/milestones/m1-e/fp27-cold.sh 20`. It checks the booking, then drops the page cache before each run (the only sudo) and logs to `m1-g/fp27-cold-<date>.log`.
2. **AppArmor full tier (E-Q1, approved):** install and probe as `research_notes/M1 E launch/apparmor/README.md` says (root for the install only), then remove the profile. Its attach paths name `~/izcal-m1e2`'s spawner builds; G's Linux rows run from `~/izcal-m1int-g`, so they stay on the restricted tier, reported by name, until the profile names the tree measured.
3. **Decisions the results ask for** (the evidence is in "Root causes" above):
   - **FP-2 on macOS:** the renderer is 4,176 KiB, FP-M0's floor (4,096 KiB, unchanged) plus 80 KiB of Izcal line items. Section 6.3's rule (floor plus itemised line items) or Section 8.2's flat 4 MiB? Meeting 4 MiB flat needs the sandbox's 2.8 MB gone, which was only measured by applying the profile before an `exec`, and that widens the profile for the renderer's life.
   - **Mappings on macOS:** 53 of the 83 entries are the OS's (33 the dyld shared region), so ≤ 64 is out of reach under the top-level count. Do the shared region's entries count, or is the macOS figure set from the measured floor?
   - **FP-27 cold on Windows:** warm now passes (5.4 ms). The fresh-directory fallback's 69 ms is a new file's first execution, not the launch. Measure cold by reboot on rig253, or accept the fallback as it is?
   - **Threads:** the renderer runs 6 threads at idle where FP 6.3 and FP-32 say two; each costs about 16 KiB and 4 map entries on macOS.
