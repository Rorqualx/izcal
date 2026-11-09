# M1 D5 Windows IPC: the AppContainer SID budget

Measured 7–8 November 2026 on rig253 (Windows 11 build 26100.9711, Ryzen AI
Max+ 395), as the standard user `izcal-run` except where noted. Black-box
only: the probes call documented Win32 APIs and the native calls Windows
exports, and read the kernel's pool-tag counters. Probes and the run log are
in `probes/sid-budget/`; the SID-reuse probes of 8 November are in
`probes/sid-reuse/`.

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

## Does a recycled SID leak? (8 November)

Two probes ran as a standard user (`probes/sid-reuse/`, logs `run3.log`,
`run6.log` and `izr.log`). The first (`izp.rs` with `reuse.ps1`) asks what
the next holder of a recycled SID can reach. The second (`izq.rs` with
`izq.ps1`) plants one of each remaining channel and checks it from four
positions: a sibling container while the planter runs, the sibling after it
exits, the next holder of the recycled SID, and the plain user. Both prove
their detector first: the planter's own container sees everything planted
while it lives. The third (`izr.ps1` with `clip.ps1`) adds the two channels
that need user32 or WinRT, run from a scheduled task in an interactive
`izcal-run` session because user32 fails its DllMain inside a container
reached over ssh (0xC0000142).

**The SID is a function of the name.** Deleting a profile and creating one of
the same name gives the same SID, measured in both probes; a pool is a set of
names.

`izp` — named objects, files and registry. After the planter ends, its named
objects die with the container's directory. Its file and container registry
value survive only if the profile is kept: delete and recreate the profile,
and the next holder finds nothing (control: all six channels found while the
planter lives). Two states break the clean handover, and both are
detectable unelevated. An orphan process left in the container keeps
everything reachable, and a leftover process is found by reading each
process token's container SID — conhost.exe included, since a console child
brings one into the container. And keeping the profile leaks the file and
registry value. A handle the broker keeps on the planter's named object does
not keep the directory alive.

`izq` — credentials, cookies, atoms, clipboard, `Global\` objects, ACLs.
Nothing planted reaches any other position. `CreateEventW` and
`CreateFileMappingW` in `Global\` succeed from inside a container, but the
objects land in that container's own namespace: a sibling container cannot
see them even while the planter lives. `Global\` pipes and global atoms are
denied outright. Credential Manager is unavailable inside the container:
`CredWriteW` and `CredReadW` fail with ERROR_NO_SUCH_LOGON_SESSION (1312)
over ssh and access-denied (5) in an interactive session, and
`CredEnumerateW` is access-denied, so that channel cannot leak either. No
ACE naming the container SID survives `DeleteAppContainerProfile` in the
bounds scanned (C:\ and its top level, C:\ProgramData one level, the user's
Packages directory, HKCU and Local Settings two levels, HKLM top level and
SOFTWARE one level; eight filesystem paths unreadable unelevated, recorded
as such). Three keys grant write to ALL APPLICATION PACKAGES in those
bounds — HKCU's root, `HKCU\…\Local Settings` and HKLM's root — but that is
shared by every container whether SIDs are recycled or not, and the
container's write-opens to all three are denied (5) in both sessions.

`izr` — the clipboard and the Credential Locker. In the interactive session
user32 loads (powershell runs inside the container where it died over ssh),
but the container's raw clipboard open is denied (5) in both sessions, so
nothing can be planted there. The WinRT PasswordVault works: a container
plants a credential and reads it back, a sibling container never sees it
while the planter lives or after it exits, and the plain user never sees
it — per-SID isolation holds. But the entry survives
`DeleteAppContainerProfile`: the next holder of the recycled SID reads it.
So delete-and-recreate is **not** a complete handover for the Credential
Locker; a SID-reuse pool must clear the SID's vault entries before reuse
(by a process of that SID, whose `PasswordVault.Remove` succeeds in the
planter's container, or wherever else they are stored — the container's AC
folder is wiped by the recycle, so the store lives elsewhere; locating it
was not probed).

Reuse is cheap for the kernel: 200 launch cycles on one recycled name grew
`SeLs` by 1, against 200 for 200 fresh names.

## Options (for the owner)

1. **Keep a fresh SID per launch** (now). Simplest isolation story; the
   budget above stands.
2. **A recycled pool of SIDs.** Children alive at the same time still get
   distinct SIDs (W4 holds), and the SID is a function of the profile name,
   so a pool is a set of names. Reuse only after the last container process
   has ended (the directory is gone; leftover processes are counted by
   reading each process token's container SID) and the profile is deleted
   and recreated, its folder emptied first. The handover also **clears the
   SID's Credential Locker entries**, which delete-and-recreate does not
   touch: the probes show the recycled holder reads the previous holder's
   vault credential (`PasswordVault.Remove` from a process of that SID
   works). Entries stay bounded by the pool's size, and reuse costs about
   one `SeLs` entry per pool name, not per launch. With the vault cleared,
   the probes show the next holder reaches nothing.
3. **One SID per site or per kind.** Bounded too, but it ties persistent
   identity to a site and shares a namespace across that site's processes
   over time; not recommended.

Recommendation: option 2.
