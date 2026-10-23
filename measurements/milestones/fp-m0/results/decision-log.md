# FP-M0 decision log

Started 22 October 2026. Every entry that sets or moves a † threshold is recorded here item by item, per the footprint spec's rule — never as a lump.

| Date | Budget or question | Finding | Old value | New value / effect |
|---|---|---|---|---|
| 2026-10-22 | Smoke run, harness check (not a threshold) | Chrome 155, macOS, main process only, one blank tab, 12 samples over 1 min: phys_footprint 120–126 MB, peak 141 MB. Caveat: main process only; the tree-wide matrix (all browser processes) is the next step, and the harness will gain a tree-sum mode before the real matrix runs | — | Harness works; tree-sum mode queued |
| 2026-10-22 | Harness correctness (review findings, not a threshold) | The advisory AI review found three harness defects: an unreachable pgrep fallback, a README usage mismatch, and TSV columns not matching the header. All three fixed (sample-footprint.sh v2): the fallback runs, the README states app-name/PID usage and the single-process scope, and the TSV carries five declared columns (t, sample, pid, phys_footprint_MB, peak_MB). Validated: 4 samples at 126/168 MB, clean columns | — | v2 harness is the canonical sampler; tree-sum mode still queued |

## The first matrix (22 October 2026, macOS, rig local) — v4, shipped

Methodology: fresh disposable profiles per browser; Chrome tabs created through its DevTools endpoint (`/json/new`, renderer-backed targets — direct launches create no windows in this environment, which invalidated v1) and **verified by page count** (1, 11 and 41 pages for the 1/10/30-tab loads); Firefox tabs by direct invocation with the process-tree growth from v1 as the tab-opening evidence, its page-count check queued (Firefox's debugger endpoint does not serve Chrome-style `/json/list` under the 2026 flags tried); tree sampling walks the process tree by parentage with a final `sort -u` dedup and **normalizes the footprint unit** (the CLI prints KB/B for small processes; v3 dropped the unit and inflated rows — the v3 TSVs are discarded and the v3 "startup spike" was a misparse, not a spike); 2-minute samples, steady state = the last 12 samples. All loads are example.com tabs.

| Load | Chrome 155 | Firefox |
|---|---|---|
| Blank | **162 MB** (5 procs) | **639 MB** (11 procs) |
| 1 tab | 276 MB (7) | 703 MB (12) |
| 10 tabs | 644 MB (17) | 782 MB (15) |
| 30 tabs | **1,729 MB** (47) | **840 MB** (15) |

**Reading (first, provisional):** Chrome starts ~4x cheaper but pays ~52 MB per same-site tab (one renderer per tab); Firefox's floor is ~640 MB with same-site tabs nearly free (~7 MB each, shared content processes). For FP-1's budgets the load-bearing rows are the **blank floor** (the incumbent floor Izcal's own floor must be argued against) and the **30-tab slope** (FP-S30's target). Neither is a threshold — they are FP-M0's incumbent observations, the baseline the substrate floor will be compared against.

**Caveats recorded:** same-site-only tabs (cross-site mixes come later); one machine (Apple silicon, macOS 26); Firefox's page-count verification queued (its rows rest on v1's process-growth evidence plus the unit-corrected sampler); Safari queued (single-instance model). **Discarded versions:** v1 (Chrome opened no windows; invalid loads), v2 (`local -A` is not bash-3.2), v3 (unit-suffix drop corrupted KB-scale rows; the data was discarded, the methodology kept).

## MADV_FREE_REUSABLE probe (22 October 2026, macOS 26, Apple silicon)

FP-M0 open question 17, the macOS part. A standalone 30-line probe (task_info's phys_footprint, no third-party source) allocates 256 MB, touches it, applies the advice values and reports after each step:

| Step | phys_footprint |
|---|---|
| After touch (256 MB resident) | 263,472 KB |
| After `MADV_FREE_REUSABLE` | **1,328 KB** |
| After re-touching the same region | 1,328 KB (unchanged) |
| After `MADV_FREE` (comparison) | 1,328 KB |

**Finding:** `MADV_FREE_REUSABLE` releases the `phys_footprint` charge on this OS — the spec's FP-1 macOS release path works as intended. The re-touch observation (no re-charge within the run) is recorded as observed, not explained; a longer-observation variant is queued if any budget depends on the re-charge rate. Probe: `harness/madvise-probe.c` (build: `clang -O2`).

## Cross-site mix (22 October 2026, macOS, rig local)

30 tabs across 10 distinct domains (example.com, example.org, iana.org, wikipedia.org, archive.org, openstreetmap.org, gutenberg.org, ietf.org, w3.org, python.org — three tabs each), Chrome's page count verified (30), Firefox by direct invocation with the queued page-count caveat, same sampler and steady-state rule as the v4 matrix.

| Load | Chrome | Firefox |
|---|---|---|
| 30 same-site tabs (v4) | 1,729 MB (47 procs) | 840 MB (15 procs) |
| 30 cross-site tabs | **1,830 MB (36 procs)** | **2,066 MB (41 procs)** |

**Reading:** Firefox's same-site economy disappears across sites (+1,226 MB, 15→41 processes — each distinct domain gets its own content process); Chrome's cross-site cost is mild (+101 MB) because its per-tab process model was already paying. The FP-S30 budget's worst case is the cross-site row, not the same-site one — the corpus's FP-S30 caveat ("≤ 10–12 cross-site frames per iframe-heavy page") now has its first incumbent baseline. Neither row is a threshold; both are FP-M0 observations.

## Linux leg (22 October 2026, rig hueyj — Ubuntu, snap chromium + Firefox 158)

Counter: **VmRSS + VmSwap summed across the process tree** (`/proc/<pid>/status`, per the footprint spec's Linux definition), same steady-state rule (last 12 samples), same loads. Harness: `harness/sample-tree-linux.sh` + `harness/run-matrix-linux.sh`.

| Load | Chromium (snap) | Firefox 158 (snap) |
|---|---|---|
| Blank | **897 MB** (9 procs) | **1,432 MB** (13 procs) |
| 1 tab | 1,024 MB (10) | 1,241 MB (12) |
| 10 tabs | 2,187 MB (20) | 1,562 MB (15) |
| 30 tabs | **5,656 MB** (50) | **1,583 MB** (15) |
| 30 cross-site tabs | **9,698 MB** (80) | **4,958 MB** (41) |

**Discarded versions:** Firefox v1 (no `-headless` — hueyj has no display, so Firefox never launched; all-zero rows) and v2 (repeated invocations to open tabs: the headless instance runs the profile lock but no remote listener, so every invocation after the first died and the tab count stayed at zero — identical 13-proc/1,431 MB rows at every load, the giveaway). **v3 opens each load as one invocation with N URLs** (one window, N tabs); its cross-site row's 41 processes (ten origins, ten content processes) is the evidence that the multi-URL mechanism genuinely loads every site. Same-site loads stay near-flat (1,241→1,583 MB, 15 procs) because Fission shares one content process per site — the 29 extra same-site documents cost ~350 MB inside that shared process. Same-site page-count verification stays queued (headless Firefox serves no page list); the cross-site process evidence is the load evidence. Chromium page counts were verified via `/json` (42 pages at the 30-tab load, 72 at cross-site).

**Rig notes recorded:** the firefox snap cannot see host `/tmp` ("Could not find profile folder" on a valid dir), so its profile lives under `$HOME`; the chromium snap accepted a `/tmp` profile path. Chromium needed `--headless=new`; Firefox `-headless`.

**Reading (Linux):** both browsers' Linux floors sit far above their macOS floors in this counter — ~5.5x for Chromium, ~2.2x for Firefox — because VmRSS counts shared pages fully where macOS phys_footprint does not; this is exactly the counter-mapping evidence FP open question 17 asks for, and it means FP-1's Linux budget must be argued against the Linux counter, never the macOS number. The cross-site slope is the dominant cost on Linux for both engines (Chromium +4,042 MB same→cross; Firefox +3,375 MB), consistent with macOS. Neither row is a threshold; both are FP-M0 observations.

## Windows leg (22 October 2026, rig rig253 — Edge 155 + Firefox 158.0.1)

Counters: **commit charge (PrivateMemorySize64) primary, working set secondary**, both summed across the process tree, same steady-state rule and loads. Edge stands in for the Chromium family here — the rig ships no Chrome, and Edge 155 is the Windows Chromium incumbent (recorded as a stand-in, not a substitution). Harness: `harness/sample-tree-windows.ps1` + `harness/run-matrix-windows-edge.ps1` + `harness/run-matrix-windows-ff.ps1`.

| Load | Edge 155 (private / ws) | Firefox 158 (private / ws) |
|---|---|---|
| Blank | **161 / 369 MB** (8 procs) | **608 / 803 MB** (12 procs) |
| 1 tab | 193 / 440 (9) | 619 / 767 (12) |
| 10 tabs | 430 / 1,037 (19) | 720 / 946 (15) |
| 30 tabs | **1,152 / 2,825 MB** (50) | **742 / 919 MB** (15) |
| 30 cross-site tabs | **2,184 / 5,200 MB** (80) | **1,711 / 2,941 MB** (41) |

**Discarded versions:** Firefox v1 (the GUI instance dies at window creation from the ssh service session; every sampled process was a ~4 MB launcher stub — max_single 4 MB was the giveaway) and v2 (`-headless` fixed the instance, but repeated invocations polluted the process set: 296 procs at the cross-site load, roughly a third of the private total in stubs). **v3 is the Linux v3 pattern**: one invocation per load with N URLs; the cross-site row's 41 processes (ten content processes) is the load evidence. Edge page counts verified via its DevTools `/json` (2/12/42/72 pages).

**Rig notes recorded:** the default `%TEMP%` under this user profile contains a space and an apostrophe (`joe's pc`), which breaks `Start-Process -ArgumentList` joining — profiles live under `C:\fp-m0-run\profiles` instead; `timeout` fails from the ssh session ("input redirection is not supported") — the install watchdog uses `ping -n` delays; `rig run` executes its command locally, so rig commands need an explicit `ssh`. **Firefox delivery:** Mozilla no longer publishes portable zips (the `firefox-latest` URL serves the UPX-packed setup exe; `msiexec /a` on the MSI ran a real install instead of an extract — it rolled back with no artifacts; winget per-user failed with 0x80070534 SID mapping from the ssh session), so Firefox 158.0.1 was installed via the documented `-ms` silent switch, which landed **machine-scope** in `C:\Program Files\Mozilla Firefox`. Flagged to the owner; a normal uninstaller exists.

**Reading (Windows):** the family pattern holds on a third OS — the Chromium-family blank floor (~161 MB private) sits far below Firefox's (~608 MB), Firefox's same-site tabs are nearly free (shared content processes, +134 MB private across 30 same-site tabs) while Edge pays ~33 MB private per same-site tab, and the cross-site mix is the dominant cost for both (+1,032 MB for Edge, +969 MB for Firefox — near-identical absolute jumps). The counter-mapping observation now has all three OSes: Task Manager's "Memory" column is private working set, so on Windows the reported figure follows neither raw commit nor total working set alone. Neither row is a threshold; both are FP-M0 observations.

## HTTP-cache sizing (22 October 2026, macOS 26, rig local)

FP-M0 open question 17, the HTTP-cache part: each incumbent's HTTP-cache size against free disk, black-box. Fresh profiles, the cross-site load (30 tabs, 10 domains), settle, graceful quit, then the profile's cache directories sized. Free disk: ~979 GB before and after.

| Browser | HTTP cache | Other material disk writes | Notes |
|---|---|---|---|
| Chrome 155 | **~7.8 MB, 310 files at `Default/Cache/Cache_Data`** (30 pages verified, 60-s settle, graceful quit) | Code Cache 4.8 MB, GPUCache 0.5 MB, Dawn shader caches | No `disk_cache` keys in Preferences. The pre-commit review caught the first build measuring `Default/Cache_Data` — the wrong path — and silently concluding "no cache"; the corrected path shows the cache does materialize |
| Firefox 158 | **cache2 = 44 MB, 383 entry files** (45-s settle) | — | No cache prefs written — defaults apply: smart-size computes the capacity from free disk at runtime (nothing persisted to prefs.js) |

**Reading:** against ~979 GB free disk, both incumbents' measured caches are tiny — Chrome ~0.0008%, Firefox ~0.005% — nowhere near the naive "1% of disk" guess. The 1% rule's derivation (the spec's Section 7.1 note) is confirmed as an upper-bound prior, not an observed default. Caveat recorded: the load's per-response cacheability was not controlled (example.com serves `max-age=0`-style headers) and the Chrome figure is a 60-s settle (the 3-min run measured the wrong path and was discarded); a cache-planted local server run is the follow-up if any budget needs the per-response breakdown. Probe: `harness/http-cache-probe.sh`.

## Linux PSI probe (22 October 2026, rig hueyj) — the reaction, measured

Two runs. **First run (single 16 GB shot):** chromium reached 11 verified pages, then mid-run its process tree was **SIGKILLed** — and the shot took the rig down (sshd wedged, reboot ~18:07; the TSV died with `/tmp`). **Second run (staircase 8 → 12 → 16 GiB, ~60 s each, bail-out below 1 GiB avail, TSV in `$HOME`):** 56 rows, 10-tab chromium (19 procs), every 5 s:

| Phase | Browser total | nprocs | PSI some avg10 (peak) | MemAvail |
|---|---|---|---|---|
| Baseline | **2,049 MB** | 19 | 0.00 | 19.2 GB |
| 8 GiB hold | 2,064 MB (no reaction) | 19 | 0.41 | 10.8 GB |
| 12 GiB hold | **2,064 → ~550 MB (−73%)** | 19 (no kills) | 13.8 | 10.2 GB |
| 16 GiB hold | ~450–490 MB | 19 | 16.5 | 10.0 GB |
| Recovery (60 s) | **~545 MB — stays shed** | 19 | 0.02 | 19.2 GB |

**Reading (Q-17's PSI part):** chromium reacts to memory pressure before the OOM killer — at the 12 GiB step it shed ~73% of its resident set in one step while keeping every process, and it does not re-inflate after the pressure passes (the shed pages stay shed). The 8 GiB step produced no reaction (no pressure on the browser's share). The first run's outright SIGKILL was the single-shot's fault — an instantaneous 16 GB touch storm leaves no time to react — which is why the staircase is the harness's canonical shape. The TSV: `results/linux/2026-10-22-psi-probe.tsv`; the probe: `harness/psi-probe.sh`.

## Per-element memory (22 October 2026, macOS 26, rig local)

FP-M0 open question 17, the per-element part: memory charged per element when a page creates a million of them, black-box. A localhost page appends 1,000,000 `<div>`s (one-character text node each); each browser's tree is sampled on the blank page and on the million-element page (1-minute samples, steady state = last 6; Chrome's load gated on its "million done" title, blank tab closed before sampling; Firefox loads by single invocation with the profile-lock kill).

| Browser | Blank | Million elements | Per element |
|---|---|---|---|
| Chrome 155 | 169 MB (5 procs) | 2,761 MB (7 procs) | **~2,592 bytes** |
| Firefox 158 | 641 MB (10 procs) | 1,770 MB (10 procs) | **~1,129 bytes** |

**Reading:** Chrome pays ~2.3x Firefox per `<div>`+text node on this OS. Both figures are the element variant only — the attribute-set variant (the spec's "per attribute set" wording) is a follow-up with the same harness shape. Neither row is a threshold; both are FP-M0 observations. Probe: `harness/per-element-probe.sh`; TSVs in `results/`.

## Chrome's soft process limit (22 October 2026, macOS 26, rig local)

FP-M0 open question 17, the soft-limit part: Chrome 155 opened same-site tabs one at a time through its DevTools endpoint and the process-tree size was recorded every 5 tabs. **No plateau by 80 tabs**: the tree grew linearly, one renderer per tab (nprocs = tabs + 6 helpers, 86 processes at 80 tabs, page counts verified). The soft limit on this lab device (12-core Apple silicon) lies beyond 80 tabs — no renderer reuse was observed. The same probe shape on the other lab devices is the follow-up if the limit needs locating there. TSV: `results/2026-10-22-soft-limit.tsv`; probe: `harness/soft-limit-probe.sh`.

## Safari leg (22 October 2026, macOS 26, rig local)

Safari's single-instance model: the measurement ran inside the live session with the rule that only a dedicated window, bound by reference at creation, is touched — existing windows and tabs are never touched and Safari is never killed. Loads match the matrix; the cross-site load runs in its own fresh window (the first build accumulated tabs across loads, which made its cross-site sample a 61-tab window — discarded). Tab counts verified via AppleScript (61 in the accumulated window; 31 in the fresh cross-site window).

| Load | Safari 26 |
|---|---|
| Blank | **503 MB** (21 procs) |
| 1 tab | 527 MB (21) |
| 10 tabs | 682 MB (30) |
| 30 tabs | **1,007 MB** (50) |
| 30 cross-site tabs | **1,458 MB** (44) |

**Reading:** Safari's macOS floor (503 MB) sits between Chrome's 162 MB and Firefox's 639 MB, and its same-site slope (~17 MB/tab) is close to Firefox's economy; the cross-site jump (+451 MB over 30 tabs, +17 processes) is the smallest of the three browsers on macOS. **Discarded versions:** run 1 (Safari not running — `-600`), run 2 (all-zero rows: the tree sampler's comma-separated patterns were a latent bug — the comma string was passed to `pgrep -f` as one literal regex that matches nothing; fixed by joining commas into an ERE alternation, which the matrix had never exercised because it used single patterns). The harness's macOS tree sampler is the canonical one, so the fix is recorded there. Probe: `harness/run-matrix-safari.sh` (+ `run-matrix-safari-crosssite.sh`).

## The Q-17 close (22 October 2026)

The footprint spec's revision 0.3.2 (PR #11's branch) closes open question 17 on its measured parts, item by item — the three-OS counter mapping, the per-element variant, Chromium's PSI reaction, `MADV_FREE_REUSABLE` accounting, macOS cache sizing and the local soft limit — each citing the rows above. The parts still open stay in the spec's Section 11.1 with their FP-M0 gates: the per-attribute-set variant, Firefox's PSI reaction, `MADV_FREE_REUSE`, cache sizing on Linux and Windows, and the soft limit on the other lab devices. No decision, target or threshold changed by the close.

## Spawn-latency probe (23 October 2026, macOS 26 + rig253)

FP-M0's spawn-latency part (the spare-process gate: keep a spare only while spawn-to-ready exceeds 10 ms†). The full sandbox path is gated on the sandbox itself, so this run measures the un-sandboxed floor — spawn call to the child's "ready" byte on an inherited pipe, 500 runs each. The sandbox can only add cost.

| OS | min | p50 | p90 | p99 | max |
|---|---|---|---|---|---|
| macOS 26 (Apple silicon) | 0.548 ms | 0.568 ms | 0.680 ms | **1.696 ms** | 73.1 ms |
| Windows (rig253) | 2.216 ms | 2.931 ms | 3.463 ms | **5.441 ms** | 755.3 ms |

**Reading:** both un-sandboxed p99s sit below the 10 ms† gate, so the gate is reachable pre-sandbox — the sandbox's own cost (measured at FP-M0 half B) is the deciding term. The single-run maxima are cold-start outliers, not distributions. Probes: `harness/spawn-latency-probe.c` + `spawn-latency-child.c` (macOS, clang) and `spawn-latency-probe-win.c` + `spawn-latency-child-win.c` (Windows, MSVC 14.44 via vcvars64).

## Half B: the substrate floor (23 October 2026, macOS 26, rig local)

The first Izcal code milestone (crates `hg-substrate-os`, tier O): an empty process on Izcal's substrate with the **full sandbox** of security scope 6.7, **two threads** and an **IPC endpoint** (fd 3, pre-opened by the supervisor — nothing opens after lockdown). The sandbox is a **custom deny-default SBPL** applied at start through `sandbox_init_with_parameters` (never a named profile, per the spec's SDK-27 measurement), written from first principles: `(version 1) (deny default) (allow sysctl-read)`. Tier verified by probe: file-read, mach-lookup and network-outbound all denied; the sysctl allow verified **behaviorally** (a real `sysctlbyname("hw.memsize")` read succeeds — `sandbox_check`'s "sysctl-read" name returns a false deny and is recorded as a probe-naming finding, not a tier reduction).

| Item | Value |
|---|---|
| **The floor** (release build, 2 threads + IPC endpoint, sandbox applied) | **4,096 KB phys_footprint** (stable across samples) |
| Spawn-to-ready on the full sandbox path (supervisor → ready byte) | **Distribution, 100 runs: min 4.28 / p50 4.42 / p90 4.84 / p99 11.20 / max 28.07 ms** (the ~80 ms first-run singles were cold-start noise; the bare floor's p99 was 1.70 ms, so the sandbox apply adds ~9.5 ms at p99) |

**FP-2's ship blocker, item by item (never a lump):** the measured floor **4,096 KB** on macOS 26, plus the line items as they land — the broker's process-kind machinery, IPC tables, per-partition state and the engine's heaps each add their measured rows here at their milestones; FP-2's blocker is the running sum. Linux and Windows legs of the same probe are queued (macOS first, per the README). Probe: `crates/hg-substrate-os/src/bin/fp-m0-probe.rs` (supervisor) and `fp-m0-empty.rs` (the sandboxed process); sandbox: `src/sandbox/macos.rs`.

**The spare-process gate, measured (23 October 2026):** the sandboxed spawn-to-ready distribution's p99 (11.20 ms) exceeds the 10 ms† gate on macOS — the sandbox apply pushes the bare p99 (1.70 ms) over it — so the gate's own rule says **keep a spare** on this OS. The Linux sandboxed single (0.805 ms) and the Windows leg will add their rows; the rule stays "keep a spare only while spawn-to-ready exceeds 10 ms†" and now has its first data.

## Half B, Linux leg (23 October 2026, rig hueyj)

The same empty-process probe, Linux backend: a seccomp-bpf allowlist with an architecture check (x86_64/aarch64), written from the kernel's seccomp documentation — the strictest tier of security 6.7 (compute plus IPC; no filesystem, no network). The deny action is **EPERM, not KILL**, so a denial is observable: the tier probe performs the two must-deny operations and reports their errnos (both EPERM on the run — the filter working as written). The black-box method shaped the allowlist: the first run's strace exposed `sendto` (std's unix-socket writes carry MSG_NOSIGNAL) and `set_robust_list` (glibc thread teardown) as the two syscalls the empty process needs beyond the first-principles set; each addition is labelled with its reason in the table.

| Item | Value |
|---|---|
| **The floor** (release build, 2 threads + IPC endpoint, filter applied) | **2,148 KB VmRSS, 0 KB VmSwap** (stable across samples) |
| Spawn-to-ready on the full sandbox path | 0.805 ms |

**FP-2's running blocker sum (item by item, never a lump):** macOS 26 — 4,096–4,144 KB phys_footprint; Linux (hueyj) — 2,148 KB VmRSS + 0 swap. The Windows row lands with its leg. Probe: the same `fp-m0-probe`/`fp-m0-empty` pair; sandbox: `src/sandbox/linux.rs`.

## Half B, Windows leg (23 October 2026, rig253) — floor measured, loader stall recorded

The Windows supervisor creates the restricted token (DISABLE_MAX_PRIVILEGE | SANDBOX_INERT, Untrusted label SID), the kill-on-close job and the IPC pipe; the child starts suspended under the token (CreateProcessWithTokenW — CreateProcessAsUserW demands SE_INCREASE_QUOTA_NAME, error 1314, from a non-admin caller), is assigned to the job (fatal on failure), then resumes. **The floor: 372–376 KB private commit charge, 1,992 KB working set, stable across samples** — the restricted child's steady state.

**Recorded findings (all follow-ups, none silent):**
1. **The child stalls in the loader before main under the token** — one thread, zero CPU. The bisect (23 October, rig253): the stall reproduces at **Low and Untrusted** integrity, with and without CREATE_SUSPENDED; **cmd.exe runs fine under the same token** (exits in 11.4 ms), and the plain-C spawn child exits without signaling (44.6 ms EOF) — the stall class is runtime-specific (the Rust CRT's loader path at reduced integrity), not token-general. The floor above is the loader-stalled state's footprint; a debugger session on rig253 diagnoses the stall, then the ready line and the tier probe (the SAM must-deny read) land.
2. **The environment scrub fails ERROR_INVALID_PARAMETER (87)** in both forms tried — a hand-built block and one filtered from GetEnvironmentStringsW — with CreateProcessW and WithToken alike; NULL (inherit) works. The scrub joins the Windows-tier follow-up list (AppContainer, win32k lockdown, ACG).
3. The must-deny probe's first target (`drivers\etc\hosts`) is world-readable even at Untrusted (file-read=ALLOWED on the diagnostic run) — the probe now targets the SYSTEM-only SAM hive.

**FP-2's running blocker sum:** macOS 26 — 4,096–4,144 KB phys_footprint; Linux (hueyj) — 2,148–2,152 KB VmRSS + 0 swap; Windows (rig253) — 372–376 KB private / 1,992 KB WS (the stalled-loader state, to be re-measured once the stall is fixed). The sum updates when the Windows tier line lands.

## Half B, Windows leg — the loader stall, root-caused and fixed (23 October 2026, rig253)

**Root cause, measured:** the supervisor now reports how a child ended when no ready line arrives (SEC-8). Every run of the old pair ended with **exit code `0xC0000142` (STATUS_DLL_INIT_FAILED)**, about 6 ms after spawn — a DLL's initialization failed in the loader, before `main`. The child imports no GUI library (kernel32, advapi32, ntdll, VCRUNTIME140 and the UCRT API sets only), so the creation flags were bisected:

| Creation flags (with CREATE_SUSPENDED) | Result |
|---|---|
| none (console child, default console) | `0xC0000142` before main |
| CREATE_NEW_CONSOLE | `0xC0000142` before main |
| DETACHED_PROCESS | rejected by CreateProcessWithTokenW (error 87) |
| **CREATE_NO_WINDOW** | **loader completes; the child reaches main** |

So **creating a console window for the child under the restricted token is what fails**. With CREATE_NO_WINDOW the console still exists, as a windowless `conhost.exe` child (956 KB private, 6,200 KB working set — more than the sandboxed process itself). The fix removes the console entirely: `fp-m0-empty` is built for the **GUI subsystem** (no console, no conhost), and the supervisor keeps CREATE_NO_WINDOW for any console-subsystem child. The 23 October "one thread, zero CPU" observation and the 372–376 KB floor were taken from this failing launch, not from a running child; both are superseded below.

**Second defect, measured:** the IPC handle never reached the child. CreateProcessWithTokenW starts the child through the Secondary Logon service, which passes the three STARTUPINFO std handles and no inherited handle: the pipe handle named on the command line was invalid in the child (the supervisor read end-of-pipe at once). The pipe now goes as the child's **stdout** (STARTF_USESTDHANDLES), stdin and stderr are null, and **neither pipe end is inheritable** — the service duplicates the stdout handle itself, so no inheritable handle of ours exists for any process to pick up (I-2). The child's Windows path no longer needs `unsafe` (no raw handle from argv).

**The Windows floor, re-measured** with the final build (GUI-subsystem child, 2-minute sample, one process in every row; exclusive booking, `rig check` OK at 14:14:38 and 14:16:51 -06:00): **880 KB private / 4,612 KB working set for the first ~30 s, then a steady 796 KB private / 4,552–4,568 KB working set**. Six threads at start (main, the two probe threads, and three the process did not create itself), 51 handles. TSV: `results/windows/2026-10-23-halfb-win-fixed.tsv`. The sampler now also writes KB totals (the MB columns round a sub-MB process to 1). Its timestamps are rig253's clock, which runs on Pacific time: 13:14:44 there is 14:14:44 -06:00.

**Sandboxed spawn-to-ready, 100 runs** with the final build (exclusive booking, `rig check` OK at 14:14:13 -06:00): **0 failures; min 8.30, p50 10.19, p90 10.93, p99 12.33 ms** (nearest-rank: p99 is the 99th of 100 sorted values). The first run, the first launch after a rebuild, took 879.61 ms (cold start). Without it, the 99 warm runs give the same p50, p90 and p99, with a maximum of 12.33 ms. Raw values: `results/windows/2026-10-23-halfb-win-spawn.tsv`. The p99 is above the 10 ms† gate, as on macOS (11.20 ms): **keep a spare process on Windows too.** Before the GUI-subsystem change (with conhost) spawn-to-ready was 21–23 ms. (An earlier 100-run pass on this branch reported its maximum, 11.71 ms, as the p99; this pass replaces it.)

**Failure reporting, checked on rig253.** The supervisor prints the timing line only for a ready child. Every other outcome prints one `FAILED` line with the child's state and exits 1; the child reports its own errors, including panics, down the pipe as one line. Each path was exercised with a temporary test switch, removed before commit:

| Child behavior | Supervisor output |
|---|---|
| ready | timing line + ready line; holds the child |
| panics | `FAILED … reported: fp-m0-empty: error: panic: …; child exited with code 0x00000001` |
| exits before writing | `FAILED … closed the pipe without a ready line; child exited with code 0x00000007` |
| hangs without writing | `FAILED … no ready line within 10 s; child still running (wait=0x102); terminating it` |

**The child does not get the supervisor's environment.** With a NULL environment block, CreateProcessWithTokenW does not hand the child the caller's environment; per its documentation, it builds a new one from the token user's profile. A variable set in the supervisor did not reach the child. The 23 October note that "NULL (inherit) works" is corrected: NULL works, but it does not inherit. The environment scrub stays on the Windows-tier follow-up list.

**The tier line now arrives, and it shows two things to fix (follow-ups, not silent):**
1. **The must-deny probe cannot answer.** Opening the SAM hive fails with **error 32 (ERROR_SHARING_VIOLATION)** for every caller, sandboxed or not — the system holds it open exclusively. The probe now prints `file-read=failed-<code>` for any failure other than access denied, so this reads `failed-32`, never `denied`.
2. **The current Windows tier does not block reads of user files.** *(Fixed later the same day by the AppContainer tier, below.)* A one-off diagnostic build pointed the probe at a file the user created in `Documents`: **`file-read=ALLOWED`** under the Untrusted, DISABLE_MAX_PRIVILEGE | SANDBOX_INERT token (also `ALLOWED` for `drivers\etc\hosts`). The Untrusted label stops writes up, not reads; read denial needs restricting SIDs on the token or an AppContainer. That joins the Windows-tier follow-up list (AppContainer, win32k lockdown, ACG, the environment scrub) together with a must-deny target that answers: a supervisor-created canary the user can read.

**FP-2's running blocker sum:** macOS 26 — 4,096–4,144 KB phys_footprint; Linux (hueyj) — 2,148–2,152 KB VmRSS + 0 swap; **Windows (rig253) — 796 KB private / 4,552–4,568 KB working set steady, 880 KB / 4,612 KB in the first ~30 s** (replaces the 372–376 KB loader-failure figure).

**Windows TSVs and the BOM.** Windows PowerShell's `-Encoding utf8` writes a byte-order mark, so every Windows TSV from `sample-tree-windows.ps1` began with one and its first column read as `\ufefft`. The sampler now writes UTF-8 without a BOM. The BOM was removed from the 11 existing Windows TSVs (22 and 23 October); no data changed.

## Half B, Windows leg — AppContainer: user files are now denied (23 October 2026, rig253)

**The gap:** under the previous tier (Untrusted integrity, privilege-stripped token, job) a canary file in the user's Documents read `ALLOWED`. The Untrusted label blocks writes up, not reads. Security scope 6.7's utility row already names the missing piece: **AppContainer without capabilities**.

**Measured first, with a throwaway supervisor/child pair** (documented APIs only; the pair and its AppContainer profile were deleted afterwards):

| Question | Measured |
|---|---|
| AppContainer, no capabilities: user file? SAM? | **denied (5)** and **denied (5)**; `hosts` and `kernel32.dll` stay readable (system files grant ALL APPLICATION PACKAGES read) |
| Launch with only a derived AppContainer SID | fails, ERROR_FILE_NOT_FOUND (2): **a registered profile is required** (`CreateAppContainerProfile`) |
| `CreateProcessAsUserW` with our restricted token + the AppContainer attribute | **works**, and the token is honored (1 privilege in the child vs 2 without it). Made from the process token directly; the old duplicate-first token failed with 1314 |
| `CreateProcessWithTokenW` + the attribute list | rejected, error 87 |
| Integrity | Windows forces **Low** for an AppContainer; the child can **lower itself to Untrusted** (verified), and reads stay denied |
| Does the exe need an ACE for the container? | **No**: a fresh copy with no container ACE launched |

**The tier now** (`src/sandbox/windows.rs`): the supervisor registers the AppContainer profile `izcal.fp-m0.empty` (per user, persistent, as an installed app's would be), and builds the lockdown token, a restricted version of its own token with every privilege but SeChangeNotify removed. It then starts the child with `CreateProcessAsUserW` in the AppContainer (no capabilities), suspended, with a handle list that allows only the stdout pipe through, and assigns it to the kill-on-close job before resuming it. The child's first step in `main` lowers it to Untrusted (`sandbox::apply`). **SANDBOX_INERT is dropped:** the old module comment said it fixed the token as a sandbox token; per its documentation it turns AppLocker and Software Restriction Policy checks *off*.

**The tier probe reports what it observes, not a fixed claim.** Before this change the mitigation list was a constant string, so a child started outside the supervisor still claimed every mitigation (SEC-8). It now reads its own token (integrity, AppContainer flag, privilege count) and its job (kill-on-close required). The supervisor creates a canary file in the user's temp folder, passes its path, and deletes it once the child has reported.

| Child | Probe line |
|---|---|
| sandboxed | `integrity=untrusted appcontainer=yes privileges=1 user-file-read=denied sam-read=denied mitigations=appcontainer+lockdown+job+untrusted (not-applied:win32k,acg,cig,cet)` |
| same binary, no sandbox (negative control) | `integrity=untrusted appcontainer=no privileges=24 user-file-read=ALLOWED sam-read=failed-32 mitigations=job+untrusted (not-applied:appcontainer,lockdown,win32k,acg,cig,cet)` |

The control shows the probe does report `ALLOWED` when a read is allowed. Its `job` is the SSH session's job, which also has kill-on-close, so that claim is true.

**Re-measured** (exclusive booking, `rig check` OK at 15:19:21 and 15:21:34 -06:00; rig253's clock is Pacific):
- **Floor:** 880 KB private / 4,668 KB working set for the first ~30 s, then a steady **792 KB private / 4,620 KB working set**, one process. TSV: `results/windows/2026-10-23-halfb-win-appcontainer.tsv`.
- **Spawn-to-ready, 100 runs:** 0 failures; min 6.36, **p50 7.774, p90 8.705, p99 10.415**, max 14.840 ms (nearest-rank); 2 runs over 10 ms. Raw: `results/windows/2026-10-23-halfb-win-appcontainer-spawn.tsv`. The p99 is still just above the 10 ms† gate: **keep a spare process on Windows.**

**FP-2's running blocker sum:** macOS 26 — 4,096–4,144 KB phys_footprint; Linux (hueyj) — 2,148–2,152 KB VmRSS + 0 swap; **Windows (rig253) — 792 KB private / 4,620 KB working set steady** (AppContainer tier; replaces 796 / 4,552–4,568).

**Still open on Windows (named, not silent):** win32k lockdown, ACG, CIG and CET (the rest of 6.7's utility row); the environment scrub; and the alternate desktop that 6.7's renderer row adds.

## Idle wakeups: the empty process polled; now it does not (23 October 2026, all three OSes)

**The target:** footprint spec 6.8 says the engine has **no periodic timers of its own and no polling**, and FP-18 budgets idle wakeups for the *whole browser* at **≤ 1/s†** with one static page visible. FP-M0 had measured memory and spawn time but never wakeups.

**The counters** (new harness files; each reads one documented per-process counter at the start and end of a 60 s window):
- macOS: `wakeups-mac.c` — `proc_pid_rusage` (RUSAGE_INFO_V2): interrupt wakeups, and of those the wakeups out of package idle.
- Linux: `wakeups-linux.sh` — voluntary and involuntary context switches summed over `/proc/<pid>/task/*/status`.
- Windows: `wakeups-windows.ps1` — the raw cumulative "Context Switches" count of each thread (`Win32_PerfRawData_PerfProc_Thread`). A first version averaged the formatted `Context Switches/sec` counter; it failed with invalid samples when the process's start-up threads exited, so the raw totals replace it, and the window starts 40 s after launch so the thread set is stable (3 threads).

**Measured: the empty process woke twice a second.** Its two probe threads each looped on a 1 s sleep to check a stop flag:

| OS (rig) | Before (main) | After (blocking wait) |
|---|---|---|
| macOS 26 (local) | **2.000/s** interrupt wakeups (0.350/s out of package idle), 0.034 ms CPU per minute | **0.000/s**, 0 CPU, two 60 s windows |
| Linux (hueyj) | **2.000/s** voluntary switches, 0 involuntary | **0.000/s**, two 60 s windows |
| Windows (rig253) | **2.033–2.067/s** context switches | **0.000/s**, 0 CPU, two 60 s windows |

That is twice FP-18's whole-browser budget from an empty process. **The fix:** the two threads and the main thread now block with `std::thread::park()` (no timer; the loop only absorbs spurious returns), so the process sits at zero wakeups until something wakes it.

Bookings: `rig check --kind exclusive` OK on local (15:37:27, 15:38:53), hueyj (15:41:07) and rig253 (15:57:55, -06:00). The first Windows pass ran after its booking had already ended (the check passed, the rig was free); the table's Windows row is from the booked rerun, which matched it. The supervisor (`fp-m0-probe`) still sleeps 1 s in its hold loop; it is test harness only, and only the sandboxed child is counted.

**FP-M0 idle-wakeup row, the empty process:** **0 wakeups/s on macOS, Linux and Windows.**

## Idle wakeups with the real substrate running (23 October 2026, M1 phase B)

**The question:** the empty process reached 0 wakeups by parking its own threads. Does the OS substrate (`hg-substrate-os::os_substrate`: worker pool, `MayBlock` pool, timer thread) keep that once all its threads exist?

**The probe:** `substrate-idle` starts the substrate with `Threads::for_this_machine()`, runs one task and one 50 ms timer so every pool has been used, then idles. The idle workers wait on a condition variable with no timeout, and with no timers pending the timer thread does too. Same counters and 60 s windows as the section above; Windows waits 40 s after launch, as before.

| OS (rig) | Substrate threads (+ main) | Idle wakeups |
|---|---|---|
| macOS 26 (local) | 15 (+1) | **0.000/s** interrupt and package-idle wakeups, 0 CPU, two 60 s windows |
| Linux (hueyj) | 11 (+1) | **0.000/s** voluntary and involuntary switches, 0 CPU, two 60 s windows |
| Windows (rig253) | 19 (+1) | **0.000/s** context switches, 0 CPU, no threads started or exited, two 60 s windows |

Bookings: light bookings on each rig (a count of wakeups, not a timing run). The Linux run used the tree from just before one last executor change (`Drop` no longer joins the thread it runs on), which does not touch the idle path.

**FP-M0 idle-wakeup row, the substrate at rest:** **0 wakeups/s on macOS, Linux and Windows.**
