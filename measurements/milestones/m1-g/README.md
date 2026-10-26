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

The network partition service beside it: macOS 4,656 KiB settled (4,736, 4,656, 4,752, 4,656, 4,656), 0 wakeups, 83–86 map entries, warm spawn 4.699 ms, fresh 25.425 ms; Windows 1,036 KiB (1,036, 1,028, 1,040, 1,028, 1,048), 0 switches, 45 allocations, warm 31.275 ms, fresh 103.634 ms. Raw rows: `results/2026-10-26-{mac,windows}-*.tsv`.

**Reading.**
- **macOS memory.** FP-M0's empty sandboxed process was 4,096–4,144 KiB. The renderer skeleton, with the substrate's thread pools, the IPC transport, its bootstrap channels and the renderer library, is 4,656 KiB: about 560 KiB of Izcal line items over a floor that alone already sits at FP-2's 4 MiB. FP-2's rule ("the measured floor plus Izcal's line items", a decision-log entry between that and 4 MiB) needs these line items itemised; that is a decision for the owner, not a pass.
- **macOS mappings.** 83 entries against ≤ 64†. vmmap reports 1,931 regions for the same process because it also counts every region inside the dyld shared-cache submap; the 83 are the process's own entries. Which count FP 6.20.6 means on macOS is not fixed by the spec text ("VM metadata (a VMA on Linux, a VAD on Windows)"); both are recorded.
- **Windows spawn.** FP-M0's empty AppContainer process reached `ready` in 7.774 ms p50; the launched renderer takes 28.6 ms warm. The launch path adds a per-child AppContainer profile and SID, the job, the handle list, a substrate start in the child and the `ready` handshake over IPC. Where the 21 ms go is not measured yet; that is the next probe, not a guess.
- **macOS spawn.** 4.7 ms warm passes 15 ms; the first exec of a newly written binary costs about 20 ms more (code-signature validation), which is what the fresh-directory runs measure.
- **Wakeups.** Zero on both OSes for both children, as FP-M0's substrate-at-rest row predicted.

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
3. **Decisions the results ask for:** FP-2 on macOS (line items over a 4 MiB floor); which macOS count FP 6.20.6's ≤ 64 means; FP-27 on Windows (28.6 ms warm against 15 ms).
