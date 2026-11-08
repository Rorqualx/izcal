# M1 D5 Windows IPC: the AppContainer SID budget

Measured 7–8 November 2026 on rig253 (Windows 11 build 26100.9711, Ryzen AI
Max+ 395), as the standard user `izcal-run` except where noted. Black-box
only: the probes call documented Win32 APIs and the native calls Windows
exports, and read the kernel's pool-tag counters. Probes and the run log are
in `probes/sid-budget/`.

## What happened

On 7 November, about 21:45, every AppContainer launch on rig253 started to
fail. `CreateProcessW` with `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES`
returned **1450** (ERROR_NO_SYSTEM_RESOURCES) for any executable, `cmd.exe`
included, and for any user's container (`acprobe.ps1`). Plain launches still
worked. The machine had been up 12 days, through tens of thousands of
`hg-launch-tool` round trips, each of which launches children in fresh
AppContainer profiles.

## What was measured

| # | Run | Result |
|---|---|---|
| 1 | Kernel system-call trace (`wpr` with `sc.wprp`, reader `etl4.ps1`) during three failing launches | **`NtCreateLowBoxToken` returned STATUS_INSUFFICIENT_RESOURCES (0xC000009A) three times**, once per launch. Named with the public kernel symbols for that build. |
| 2 | Pool tags before the reboot (`pool.ps1`, readable unelevated) | Allocations tagged **`SeLs`: 65,623 live**, flat across three snapshots. Live processes: 247. |
| 3 | Fresh boot, 300 launches of `cmd.exe`, each in a new profile, no Izcal code (`acloop.ps1`) | `SeLs` **+300** |
| 4 | Same, 300 launches in one profile | `SeLs` **+1** |
| 5 | 20 `hg-launch-tool round-trip` runs | `SeLs` **+40**: one per child, as the launcher gives each child a fresh profile |
| 6 | Two minutes after the runs, nothing running | `SeLs` still there (389, then 401 with other system activity) |

Ruled out on the way: window-station and desktop DACL growth (4 and 2 ACEs),
leftover `AppContainerNamedObjects` entries (2), leftover profiles (0),
handle leaks (the largest holder of process handles held 1,030), memory
(17 of 27.6 GB in use).

## Finding

Each **distinct** AppContainer SID that starts a process leaves one kernel
allocation tagged `SeLs`. It outlives the process, the profile's deletion and
the container's named-object directory, and is freed only by a reboot. Near
65,536 of them (2^16; 65,623 seen), `NtCreateLowBoxToken` fails and **no
user's AppContainer process can start until a reboot**: Izcal's, other
applications' and Windows' own.

What `SeLs` is inside the kernel was not looked at (black-box rule). The
count, its growth per distinct SID and the failing call are measured.

## Meaning for the launcher

W4 (`windows_facts.md`) requires that **siblings never share a SID**: two
children in one container reach each other's pipes outside the broker. The
launcher meets that with a fresh profile for every launch, plus spares made
ahead. The cost is one `SeLs` entry per child launch, for the life of the
boot, counted across the whole machine.

So a machine allows about 65k Izcal child launches per boot, less whatever
else uses AppContainers. A browser that relaunches renderers and services
over weeks of uptime can reach that, and when it does it breaks every other
AppContainer app on the machine as well as Izcal.

## Options (for the owner)

1. **Keep a fresh SID per launch** (now). Simplest isolation story; the
   budget above stands.
2. **A recycled pool of SIDs.** Children alive at the same time still get
   distinct SIDs (W4 holds). A SID is reused only after its last process has
   ended and its named-object directory is gone, and the profile's folder is
   emptied first. Entries stay bounded by the pool's size. Before adopting
   it, a probe must show that nothing of the previous holder (named objects,
   files, registry) reaches the next one.
3. **One SID per site or per kind.** Bounded too, but it ties persistent
   identity to a site and shares a namespace across that site's processes
   over time; not recommended.

Recommendation: option 2, with the reuse probe first.
