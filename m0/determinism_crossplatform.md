# V8 determinism probe: macOS, Linux and Windows

Run on 14 October 2026. This repeats M0 step 6 (`M0_RESULTS.md`, "Step 6: determinism probe") on Linux x86-64 (huey) and Windows x86-64 (the build laptop). It uses the same fork commits, the same probe program and the same variants.

**Result: the forked predictable platform is deterministic on all three OSes. Its hashes are also bit-identical across the three OSes** for every variant, seed and worker mode, in release and debug V8. The stock platform's GC channel diverges on every OS.

## Setup

### Fork commits

The macOS commits `18b3335` (predictable platform plus `tests/test_predictable_platform.rs`) and `895b6e7` (rustfmt of `src/cppgc.rs`) were moved with `git format-patch` (`m0/xplat/patches/`) and applied with `git am --3way` on top of each host's `hgba` branch. Both applied cleanly, with no conflicts.

| Host | Base (cppgc commit) | Platform commit | rustfmt commit |
|---|---|---|---|
| macOS arm64 | `8843380` (+ `6d065ee` disarm) | `18b3335` | `895b6e7` |
| Linux x86-64, `~/hgba-m0/rusty_v8` | `38e7dda` | `28bf4ec` | `006afd3` |
| Windows x86-64, `C:\hgba-m0\rusty_v8` | `aff8a33` | `b555b4f` | `9c0be58` |

### Rebuild (reusing the existing build directories)

Only `binding.cc` and the Rust sources changed, so ninja ran **2 steps** in every configuration: `CXX obj/rusty_v8/binding.o(bj)` and `AR`/`LIB` of the archive. V8 itself was not recompiled.

| | Release sandbox | Debug-V8 sandbox |
|---|---|---|
| Linux (`target-sandbox`, -j6/-j4, nice 15, ionice idle) | **15 s** (max RSS 0.47 GB) | **19 s** (0.82 GB) |
| Windows (`t-rel`, `t-dbg`, -j12; `GN_ARGS=is_debug=true`) | **38 s** (includes `gn gen` and build.rs) | **24 s** |
| Probe crate build afterwards | Linux 19 s; Windows 33 s | Linux 22 s; Windows 39 s (libcmtd `RUSTFLAGS`) |

Notes:
- On Linux the probe links the rebuilt archive directly: `RUSTY_V8_ARCHIVE=target-sandbox/<profile>/gn_out/obj/librusty_v8.a` (`scripts/probe_env.sh`). The debug probe uses the `dev` profile.
- On Windows the probe builds with `V8_FROM_SOURCE` into the same target dirs, which is a ninja no-op. The debug probe uses `--release` with `GN_ARGS=is_debug=true` and the libcmtd `RUSTFLAGS`, as in `windows/M0_WINDOWS_RESULTS.md`.
- **The Linux memory guard was adjusted.** huey's uid slice showed `memory.current` 8.1 GB against `memory.high` 9.0 GiB, but 7.5 GB of that was page cache (`anon` was 21 MB). The original guard would have paused at once. `scripts/monitor_anon.sh` subtracts `memory.stat:file` from `memory.current`. Headroom stayed above 8 GiB throughout and the guard never paused.

### Platform test

`cargo test --features v8_enable_sandbox --test test_predictable_platform`:

| | Release | Debug V8 |
|---|---|---|
| Linux | **pass** | **pass** |
| Windows | **asserts pass, then the test binary aborts at platform teardown** (see "Windows teardown crash") | **pass** |

### Probe

`m0/determinism-probe/` was copied unchanged (`src/main.rs`, `src/probe.js`, `Cargo.lock`) to `~/hgba-m0/determinism-probe` and `C:\hgba-m0\determinism-probe`.

The run sequence matches macOS:
- **Release:** each variant 1,000 in-process runs plus 50 fresh processes.
- **Debug V8:** `pred` 200 plus 20 fresh; `pred-inline`, `pred-noflags`, `default-flags` and `default` 200 each.
- **Added to the macOS sequence:** debug `pred-inline` and `default` (both hosts), and **1,000 debug `default-flags` runs on all three OSes**. The macOS 2/200 divergence is rare, so a larger sample was needed. The macOS debug `pred-inline` 200 + 20 fresh and 1,000 `default-flags` runs were made today against the existing macOS debug archive.

The one exception is Windows release fresh processes; see "Windows teardown crash" for why.

## Per-OS results

Hash columns are FNV-1a. **R** is `Math.random`, **D** is `Date.now`, **G** is the GC observables. Every run of every variant on every OS had `finalized=11200` and **R = `aa508f33760136b6`**.

### macOS arm64 (Apple M4), from `M0_RESULTS.md` plus today's debug additions

| Variant | Build | In-process | Fresh | total | D | G |
|---|---|---|---|---|---|---|
| `pred` (queued) | release | 1,000/1,000 identical | 50/50 | `92c3fc8abd88ea44` | `e221096cd5924672` | `76372f44f262847b` |
| `pred-inline` | release | 1,000/1,000 | 50/50 | `cdc6061a80562140` | `e221…` | `3b91518141f2e11f` |
| `pred-noflags` | release | 1,000/1,000 | 50/50 | `7bedd8f98870e917` | `e221…` | `2df3e532ae1bf52e` |
| `default-flags` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **1** (`7637…`) |
| `default` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **848 distinct** (50/50 fresh) |
| `pred` | debug | 200/200 | 20/20 | `92c3…` | `e221…` | `7637…` |
| `pred-inline` (new) | debug | 200/200 | 20/20 | **`92c3…`** | `e221…` | **`7637…`** |
| `pred-noflags` | debug | 200/200 | | `7bed…` | `e221…` | `2df3…` |
| `default-flags` | debug | 200 | | | | **2 distinct** (198 `7637`, 2 `3b91`) |
| `default-flags` ×1,000 (new) | debug | 1,000 | | | | **4 distinct** (997 `7637`, 1 `3b91`, 1 `493f63baa5e870de`, 1 `53f96384fe731a19`) |

### Linux x86-64 (huey, Ubuntu 24.04, 8 cores, shared host, nice 15)

| Variant | Build | In-process | Fresh | total | D | G |
|---|---|---|---|---|---|---|
| `pred` (queued) | release | **1,000/1,000 identical** (63 s) | **50/50** | `92c3fc8abd88ea44` | `e221096cd5924672` | `76372f44f262847b` |
| `pred-inline` | release | **1,000/1,000** | **50/50** | `cdc6061a80562140` | `e221…` | `3b91518141f2e11f` |
| `pred-noflags` | release | **1,000/1,000** | **50/50** | `7bedd8f98870e917` | `e221…` | `2df3e532ae1bf52e` |
| `default-flags` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **1** (`7637…`), fresh 1 |
| `default` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **912 distinct**; fresh 49/50 distinct |
| `pred` | debug | 200/200 | 20/20 | `92c3…` | `e221…` | `7637…` |
| `pred-inline` | debug | 200/200 | | **`92c3…`** | `e221…` | **`7637…`** |
| `pred-noflags` | debug | 200/200 | | `7bed…` | `e221…` | `2df3…` |
| `default-flags` | debug | 200 | | | | **1** (`7637…`) |
| `default-flags` ×1,000 | debug | 1,000 | | | | **1** (0 divergences) |
| `default` | debug | 200 | | | | **144 distinct** |

### Windows x86-64 (laptop, 32 logical CPUs)

| Variant | Build | In-process | Fresh (with `PROBE_EXIT=leak`) | total | D | G |
|---|---|---|---|---|---|---|
| `pred` (queued) | release | **1,000/1,000 identical** (35 s) | **50/50** | `92c3fc8abd88ea44` | `e221096cd5924672` | `76372f44f262847b` |
| `pred-inline` | release | **1,000/1,000** | **50/50** | `cdc6061a80562140` | `e221…` | `3b91518141f2e11f` |
| `pred-noflags` | release | **1,000/1,000** | **50/50** | `7bedd8f98870e917` | `e221…` | `2df3e532ae1bf52e` |
| `default-flags` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **2 distinct** (999 `7637`, **1 `3b91`**); fresh 1 |
| `default` | release | D 1,000 distinct | D 50 distinct | n/a | n/a | **989 distinct**; fresh 50/50 distinct |
| `pred` | debug | 200/200 | 20/20 (unmodified program) | `92c3…` | `e221…` | `7637…` |
| `pred-inline` | debug | 200/200 | | **`92c3…`** | `e221…` | **`7637…`** |
| `pred-noflags` | debug | 200/200 | | `7bed…` | `e221…` | `2df3…` |
| `default-flags` | debug | 200 | | | | **1** (`7637…`) |
| `default-flags` ×1,000 | debug | 1,000 | | | | **1** (0 divergences) |
| `default` | debug | 200 | | | | **166 distinct** |

**Worker tasks drained by the embedder:**
- `pred` (queued): 8 per run on every OS and build.
- `pred-noflags` release, over 1,000 runs: Linux 147,008, Windows 147,004 (about 147 per run, plus a few extra in the first run). macOS reported about 152 per run.
- `pred-noflags` debug, over 200 runs: Linux 29,609, Windows 30,605.

The queued task counts differ slightly between OSes, but the G hash does not.

## Cross-OS comparison

| Variant (seed 42, virtual clock) | macOS arm64 | Linux x86-64 | Windows x86-64 | Identical across OSes? |
|---|---|---|---|---|
| `pred` queued, release | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | **yes** |
| `pred-inline`, release | `cdc6061a80562140` | `cdc6061a80562140` | `cdc6061a80562140` | **yes** |
| `pred-noflags`, release | `7bedd8f98870e917` | `7bedd8f98870e917` | `7bedd8f98870e917` | **yes** |
| `pred` queued, debug V8 | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | **yes** |
| `pred-inline`, debug V8 | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | `92c3fc8abd88ea44` | **yes** |
| `pred-noflags`, debug V8 | `7bedd8f98870e917` | `7bedd8f98870e917` | `7bedd8f98870e917` | **yes** |
| R channel, all variants | `aa508f33760136b6` | same | same | **yes** |
| D channel, all `pred*` variants | `e221096cd5924672` | same | same | **yes** |
| `default-flags` modal G | `76372f44f262847b` | same | same | yes (= `pred` queued G) |

**What the cross-OS identity means:**
- **The total hash covers the full JSON log**, including the complete finalization order and WeakRef liveness counts per step. So the GC-observable trace is the same on arm64 macOS (libc++, Mach VM), x86-64 Linux (glibc) and x86-64 Windows (MSVC ABI, clang-cl, PartitionAlloc). Release and debug V8 match too.
- **V8's heap growing and allocation-triggered GC heuristics produced the same decisions on all three OSes** for this script. Compressed-pointer object sizes are the same on arm64 and x64, and the sandbox and cage layout is the same, so this is plausible. It is still not guaranteed in general: page sizes, JIT tiering and code size could feed GC triggers on other workloads.
- **Inline vs queued worker modes:**
  - In **release** they differ on every OS (G `7637…` vs `3b91…`), and each mode is stable.
  - In **debug V8** the two modes give the **same** hashes on all three OSes (newly measured on macOS too). So the worker-task policy changes GC observables only in release builds, which matters for any plan that compares hashes between release and debug. With queued mode, release and debug agree on every OS.

## Stock platform

- **No flags (`default`):** the GC channel diverges everywhere.
  - Release: macOS 848, Linux 912 and Windows 989 distinct G values in 1,000 runs.
  - Debug: Linux 144/200 and Windows 166/200.
  - Fresh processes are essentially all distinct.
  - The D channel follows the real clock and is never stable.
- **`--predictable --single-threaded` (`default-flags`):** the G channel is *almost* stable, and the outliers are real.

  | | Release, 1,000 runs | Debug, 200 runs | Debug, 1,000 runs |
  |---|---|---|---|
  | macOS | 0 | 2 | **3** (3 distinct outlier values) |
  | Linux | 0 | 0 | 0 |
  | Windows | **1** (`3b91…`) | 0 | 0 |

  The outliers appear on macOS (debug) and Windows (release). Linux, an 8-core host under nice 15, showed none in 2,200 runs, but that is not evidence of safety.
  - Two of the macOS outlier values (`493f…`, `53f9…`) match no `pred*` mode. So racing worker threads can produce schedules that neither the inline nor the queued policy produces.
- **Conclusion:** the M0 finding holds on all three OSes. The flags alone are not enough, and the predictable platform is what makes the GC channel deterministic.

## Windows teardown crash (found during this run, not caused by the fork patch)

**Symptom:** on **Windows release** builds, a process that drops its last `SharedRef<v8::Platform>` (normally at the end of `main`, after `V8::dispose()` and `V8::dispose_platform()`) exits with `0xC0000005` or `0x80000003`. The `0x80000003` runs print `[FATAL:partition_root_internal.h(526)] Check failed: IsManagedByPartitionAlloc(object_addr)`.

**Impact:**
- The probe's `spawn` mode asserted on the child's exit status, so the five Windows-release fresh-process sets failed. In-process results were printed before the crash and are unaffected.
- `test_predictable_platform` aborts at teardown after its assertions.

**Diagnosis** (`windows/logs_determinism/shutdown_diag.txt`, using a copy of the probe with a shutdown switch, `m0/xplat/probe-src/main.rs`):
- The crash happens for the **stock `new_default_platform` as well as the predictable platform**.
- It happens whether the SharedRef is dropped before or after `dispose_platform`: it is the destruction of the platform object itself.
- It never happens when the SharedRef is forgotten (`mem::forget`) or the process exits before main's locals drop.
- The **debug-V8 build (`/MTd`, libcmtd) is not affected.** Debug fresh processes and the debug platform test pass.
- macOS and Linux are not affected.
- This looks like an allocator mismatch between the `/MT` release CRT and PartitionAlloc when the C++ platform object is deleted. It is in rusty_v8 on Windows and is independent of `18b3335`.

**Workaround for this run:**
- The Windows-release fresh-process sets were rerun with `PROBE_EXIT=leak`, which forgets the platform at exit. Hashing logic is unchanged (`determinism_probe_windows_spawn_leak.log`). All 50/50 runs matched the in-process hashes.

**Follow-up:**
- Root-cause the crash.
- Until then, embedders on Windows release should keep the platform alive for the life of the process, which is what rusty_v8's own tests and Deno do.
- Make the platform test skip the final teardown on Windows release, or hold the platform in a static.

## Files

| Location | Contents |
|---|---|
| `m0/xplat/patches/` | The two transferred commits (format-patch) |
| `m0/xplat/probe-src/main.rs` | Probe with the `PROBE_EXIT` shutdown switch (Windows diagnosis only) |
| `m0/xplat/win/` | Windows `.bat` drivers (rebuild, probe, diag). The scheduled tasks were deleted afterwards |
| `m0/linux/logs_determinism/` | `determinism_probe_linux.log` (release), `determinism_probe_linux_debugv8.log`, `..._defaultflags1000.log`, `rebuild_{release,debug}.log`, `mem_rebuild_*.log` (guard samples), `test_predictable_platform_*.log`, and the scripts (`rebuild_v8.sh`, `monitor_anon.sh`, `probe_env.sh`, `run_probe.sh`) |
| `m0/windows/logs_determinism/` | `determinism_probe_windows.log` (release, original spawn failures included), `determinism_probe_windows_spawn_leak.log`, `determinism_probe_windows_debugv8.log`, `..._defaultflags1000.log`, `rebuild_*_sandbox.log`, `build_{rel,dbg}.log` (probe builds), `test_predictable_platform_{rel,dbg}.log`, `shutdown_diag.txt` |
| `m0/logs/determinism_probe_debugv8_inline.log`, `m0/logs/determinism_probe_debugv8_defaultflags1000.log` | Today's macOS debug additions |
| huey `~/hgba-m0/` | `determinism-probe/`, `target-probe-{release,debug}/`, `target-platformtest-{release,debug}/` (about 1 GB; deletable) |
| laptop `C:\hgba-m0\` | `determinism-probe\` (`probe_rel.exe`, `probe_dbg.exe`), `probe-diag\`, `xplat\` |

The fork commits are local on each host and not pushed.
