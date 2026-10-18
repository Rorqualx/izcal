# M0 Windows x86-64 results: rusty_v8 fork, sandbox configuration

Run on 14 October 2026 on the build laptop "HueyTheSojurn":
- Windows 11 Pro (build 26200), 32 logical CPUs, 28 GB RAM.
- Visual Studio **Build Tools** 2022 (MSVC 14.44), with Windows SDK 10.0.26100 and 10.0.22621 installed.
- rustc 1.91.0, the version pinned by rusty_v8's `rust-toolchain.toml`.
- Chromium clang `llvmorg-23-init`, downloaded by `build.rs`.

Everything lives under `C:\hgba-m0\`.

## Summary

- **Start condition (i) is met for Windows x86-64.**
  - Spike items 1–5 and the companion crate's tests pass with rusty_v8's `v8_enable_sandbox` configuration, in both release and debug V8 builds.
  - They use the **fork's native `v8::cppgc` APIs**: no C++ shim and no mangled symbols.
  - Every behaviour matches macOS (`cppgc-spike/RESULTS.md`).
- **The sandbox is really on.** After `V8::initialize`, the process holds about 1.4 TiB of virtual address space (`logs_*/sandbox_probe.log`).
- **A debug V8 builds from source on Windows**, contrary to the "cannot link a debug build" comment in `build.rs`. It needs two overrides, given in the recipe below.
- **Admin action still needed:** install "Debugging Tools for Windows" for SDK 10.0.26100 (problem 2 below). Everything else was worked around without admin rights.

## Fork

- **Location:** `C:\hgba-m0\rusty_v8`, a clone of tag `v152.2.0` (`2768994`) with shallow submodules.
  - The submodules were checked out with `core.autocrlf=false`. The laptop's global `core.autocrlf=true` would have converted the whole Chromium tree to CRLF.
  - The global setting is unchanged; `core.longpaths=true` was set globally.
- **Branch `hgba`, commit `aff8a33`:** applies `v8-cppgc-ext/upstream/rusty_v8-152.2.0-cppgc-ext.patch` to `src/binding.cc`, `src/cppgc.rs` and `src/isolate.rs`.
  - The patch applied cleanly.
  - **It needed no Windows-specific change.** Only `extern "C"` functions, opaque pointers and POD structs cross the boundary. `DetailLevel` is `uint8_t` on both sides, and the enum and `usize` arguments follow existing rusty_v8 patterns.
  - **The `binding.cc` hunk now compiles inside rusty_v8's own V8 build**, with clang-cl and the MSVC ABI. That is the first time; on macOS it had only been compiled out of tree.
- **Untracked inputs, not committed:**
  - `tools/win`: the pinned archive the README asks for (`faefd1b6…`).
  - `third_party/fadec/src` at `c9f78f53…` and `third_party/disarm/src` at `2d13d3f4…`, pinned by `v8/DEPS`.

## Build recipe (release, sandbox)

Run from `C:\hgba-m0\rusty_v8` in cmd. The full scripts are in `C:\hgba-m0\build_rel.bat` and `build_dbg.bat`.

```bat
set V8_FROM_SOURCE=1
set PYTHON=C:\Users\Joe\AppData\Local\Programs\Python\Python312\python.exe
set CARGO_TARGET_DIR=C:\hgba-m0\t-rel
set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set WINDOWSSDKDIR=C:\hgba-m0\winsdk
cargo build -j 12 --release --features v8_enable_sandbox -vv
```

The resulting gn args:

```
is_debug=false use_custom_libcxx=true v8_enable_sandbox=true v8_enable_external_code_space=true v8_enable_pointer_compression=true v8_enable_v8_checks=false
```

**`-j 12` matters.** At the default `-j 32`, 32 clang-cl processes used about 17 GB of RAM. The machine thrashed: 1 GB free, 20–28k pages/s, 5% CPU. `cargo -j N` sets `NUM_JOBS`, which `build.rs` passes to `ninja -j`.

## Debug-from-source recipe (heap verification)

`build.rs` forces `is_debug=false` on Windows. Two changes make a debug build work:

1. **Override in gn.** gn honours the last assignment, so a later `is_debug=true` wins:

   ```bat
   set GN_ARGS=is_debug=true
   set CARGO_TARGET_DIR=C:\hgba-m0\t-dbg
   cargo build -j 12 --release --features v8_enable_sandbox -vv
   ```

   The resulting `CPPGC_VERIFY_HEAP`, `VERIFY_HEAP`, `_DEBUG`, DCHECKs and `V8_ENABLE_GENERATED_CODE_VALIDATOR` were verified with `gn desc`.

   The generated-code validator needs `fadec` (problem 3 below).
2. **Link Rust against the static debug CRT.** V8 is compiled `/MTd`, because Chromium's `static_crt` config uses it when `is_debug=true`.

   With Rust's default CRT, linking fails:

   ```
   LNK4098: defaultlib 'libcmtd.lib' conflicts with use of other libs
   LNK2019: unresolved external symbol _CrtSetReportMode / _CrtSetReportFile
            referenced in v8::base::EnsureConsoleOutputWin32  (platform-win32.obj)
   ```

   The fix follows Chromium's `build/config/win/BUILD.gn` comment for `static_crt`. It applies to every binary that links the debug V8:

   ```bat
   set RUSTFLAGS=-Ctarget-feature=+crt-static -Clink-arg=/NODEFAULTLIB:libcmt.lib -Clink-arg=libcmtd.lib
   ```

   With this flag, tests and spike programs link and run. The failing link log is `logs_debug_sandbox/sandbox_probe.link_fail_default_crt.log`, on the laptop.

## Workarounds (problems found)

1. **Visual Studio was not found.**
   - Error: `vs_toolchain.py` raised "No supported Visual Studio can be found".
   - Cause: Chromium looks for VS 2022 under `%ProgramFiles%`, but Build Tools 2022 is installed under `Program Files (x86)`.
   - Fix: `set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`.
2. **"Debugging Tools for Windows" is missing (needs admin).**
   - Error: `gn gen` fails with `dbghelp.dll not found in "C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\dbghelp.dll"`.
   - The SDK feature is not installed, and the build only uses it to copy `dbghelp.dll` and `dbgcore.dll` into `gn_out`.
   - Workaround, no admin needed:
     - `C:\hgba-m0\winsdk` mirrors the SDK through directory junctions (`mklink /J`) for every SDK subfolder.
     - `Debuggers\x64\` holds copies of `C:\Windows\System32\dbghelp.dll` and `dbgcore.dll`.
     - `set WINDOWSSDKDIR=C:\hgba-m0\winsdk`.
   - **Proper fix (admin):** in Apps > Installed apps, modify "Windows Software Development Kit 10.0.26100" and add the "Debugging Tools for Windows" feature. Then drop the `WINDOWSSDKDIR` override.
3. **`fadec` is missing for the debug and verification sandbox configuration.**
   - Error: `ninja: error: '../../../rusty_v8/third_party/fadec/src/parseinstrs.py' ... missing`.
   - Cause: on x64, `v8_enable_generated_code_validator` defaults to on when verification features and the sandbox are both on. It depends on `//third_party/fadec`, which rusty_v8 does not carry as a submodule. This is the x64 counterpart of rusty_v8#2070's `disarm` problem on arm64.
   - Because of `secondary_source = "//v8/"`, the sources must sit in the **root** `third_party/`, not under `v8/third_party/`. Fix:

     ```
     git clone https://chromium.googlesource.com/external/github.com/aengelke/fadec.git third_party/fadec/src  (checkout c9f78f532b9004de278489019ab2c6c28ae9746e)
     git clone https://chromium.googlesource.com/external/github.com/aengelke/disarm.git third_party/disarm/src (checkout 2d13d3f410a52daff1c5d8ef07d623332f372560)
     ```

   - The release sandbox build does not need either one.
4. **The `tools/win` submodule is skipped by design** (`update = none`). Populate it from the pinned archive, as the README describes, with Windows' built-in `curl` and `tar`:

   ```
   curl -fsSL -o tools_win.tar.gz https://chromium.googlesource.com/chromium/src/tools/win/+archive/faefd1b6fa9eeb033ad6fe60368ccb9bf908cbd0.tar.gz
   tar -xzf tools_win.tar.gz -C tools\win
   ```

**Not needed:**
- Symlinks: Developer Mode is already on.
- The long-paths registry key: it is off, but the short `C:\hgba-m0` root was enough.
- Extra Visual Studio components: VC tools x64, the Win11 SDK and the Clang toolset are present.

## Build table

All builds ran at `-j 12` with low other load. The source checkout is 3.35 GB, of which 2.40 GB is Chromium's Rust toolchain in `third_party/rust-toolchain`.

| Config | Wall time | `rusty_v8.lib` | `gn_out` | Target dir (V8 only) | Target dir incl. test binaries |
|---|---|---|---|---|---|
| release sandbox | **16m33s** for the final run, plus a 3m14s partial `-j32` run that was stopped for thrashing: **about 20 min of compile in total** | **239 MB** (SHA-256 `fbf3f312…830e`) | 1.14 GB | 2.26 GB | 4.28 GB |
| debug sandbox (`is_debug=true`) | **23m26s**, clean, including the clang download | **533 MB** | 1.95 GB | 3.58 GB | 8.31 GB |

**Download overhead** for a first build: Chromium clang plus libclang about 0.3 GB, gn/ninja, and the Rust toolchain about 2.4 GB. That is a few minutes on this connection.

**Release build steps:** 2,427 ninja steps. The heaviest are `torque_generated_initializers/*-tq-csa` and `v8_base_without_compiler`.

**Clone:** about 65 s for 19 submodules, shallow.

**Archives:**
- Release: copied to the Mac as `m0/windows/rusty_v8_ptrcomp_sandbox_release_x86_64-pc-windows-msvc.lib`, together with `src_binding_ptrcomp_sandbox_release_x86_64-pc-windows-msvc.rs`. These names follow rusty_v8's `RUSTY_V8_MIRROR` naming, but the `.lib` is not gzipped.
- Debug: 533 MB, so it was **not copied**. It is at `C:\hgba-m0\t-dbg\release\gn_out\obj\rusty_v8.lib`, with its binding at `C:\hgba-m0\t-dbg\release\gn_out\src_binding.rs`.

## Test harness

The harness is `C:\hgba-m0\m0win`, a workspace that patches `v8` to `../rusty_v8` and enables the sandbox feature by default. It has two crates.

**`compat/`, package `v8-cppgc-ext` 0.2.0-fork**
- The v8-cppgc-ext API as a thin compatibility layer over `v8::cppgc`:
  - It re-exports the fork's types.
  - It provides the extension traits `CppHeapExt`, `IsolateExt`, `VisitorExt` and `LivenessBrokerExt`.
  - Raw weak-callback and liveness calls go through the fork's own C-ABI functions.
  - It contains **no C++ and no mangled symbols**.
- It runs v8-cppgc-ext's tests (`scopes`, `statistics`, `prefinalizer`, `testing_api`, `heap_consistency`), the patch's own `upstream_api` tests, and its spike ports for items 1, 2 and 4.
- **`abi.rs` was adapted to the fork**, because there is no separately compiled shim to check. It now checks three things:
  - V8 accepts the build configuration (child process).
  - `Member` round trips over 20k objects.
  - Statistics cross the libc++ boundary intact.
- `run_child` reports Windows crashes as exit codes; `0x80000003` is the breakpoint V8 hits on FATAL or CHECK.

**`spike/`, package `cppgc-spike` 0.2.0-fork**
- The spike's support library, re-implemented with the fork APIs; `mod ffi` is removed.
- It runs spike items 3 and 5 from `cppgc-spike/src/bin` with one line changed: `TestingHeap::for_heap`.

Item 6 and the TSAN runs were not part of this task.

## Results

### Integration tests (`cargo test --release -p v8-cppgc-ext -- --test-threads=1`)

| Suite | Release sandbox | Debug sandbox |
|---|---|---|
| abi (fork variant), 5 tests | 5/5 | 5/5 |
| heap_consistency, 4 tests | 4/4 | 4/4: `control_without_barrier_loses_object` **printed "skipped on debug V8 (MarkingVerifier aborts by design)"** |
| prefinalizer, 5 tests | 5/5 | 5/5 |
| scopes, 5 tests | 5/5 | 5/5 |
| statistics, 3 tests | 3/3 | 3/3 |
| testing_api, 6 tests | 6/6 | 6/6: `forced_compaction_does_not_move_objects` **printed "skipped on debug V8 (DCHECK(!enable_for_next_gc_for_testing_))"** |
| upstream_api (the patch's own tests), 2 tests | 2/2 | 2/2 |
| **Total** | **30/30** | **30/30**, of which 2 self-skip by design, as on macOS |

The two "skipped" messages were confirmed with `--nocapture` (`logs_debug_sandbox/11_skipcheck.log`). Doc tests were not run, because the compat crate has none.

### Spike items vs macOS (`cppgc-spike/RESULTS.md`; sandbox off, mangled symbols)

**Item 1: no-GC scope**

| Scenario | Windows release sandbox | Windows debug sandbox | macOS (release / debug) |
|---|---|---|---|
| detached-quiescent / detached-marking | 0 finalized in scope, 0 reader errors, 5,000 finalized after; PASS | same; PASS | same |
| unified-marking / unified-alloc-only | **5,000 finalized inside the scope** (scope does not stop V8-initiated GC); 0 live finalized, 0 errors | same | same |
| unified-blocked-owner | 8,891 phases, 0 finalizations, 0 errors; PASS | 1,769 phases, 0 finalizations, 0 errors; PASS | 10,244 / 1,396 phases, 0 finalizations |
| unified-disallow | `Check failed: !IsGCForbidden()`, exit `0x80000003` | same | same (SIGTRAP) |
| compaction-probe | addresses stable | `Debug check failed: !enable_for_next_gc_for_testing_` (compactor.cc:470) | same |
| enter+leave pair | 3.2 ns; p99 100 ns (timer tick 100 ns) | 4.2 ns | 2.0 ns; p99 42 ns |

**Items 2–5**

| Item | Windows release sandbox | Windows debug sandbox | macOS (release / debug) |
|---|---|---|---|
| 2: raw pointers across threads | 30 s: 8,109 cycles (8,039 with marking active), **0 finalized during worker phases, 0 errors**; PASS | 20 s: 3,271 cycles, 0 finalized, 0 errors; PASS | 9,540 / 3,120 cycles, 0 finalized, 0 errors |
| 3: DOM–JS edges, precise GC | 3a, 3b, 3c pass; 3d leaks as expected; PASS | PASS | PASS |
| 3, conservative variant | 3c fails, as on macOS | same | same |
| 4: Rust-owned data and prefinalizers | `Drop` on the main thread only; 100 then 200 prefinalizers; Arc count 5001→1; PASS | PASS | PASS |
| **5: omitted write barrier** | **Missed silently:** B finalized while A still points to it, leaving a dangling `Member` | **Caught:** `MarkingVerifier: Encountered unmarked object. Hint: TNode → TNode` (marking-verifier.cc:42), exit `0x80000003` | Missed in release / caught in debug |

**Conclusion:** the semantics found on macOS hold on Windows x64 under the sandbox:
- the narrow `NoGarbageCollectionScope` behaviour;
- the safety of the blocked-owner pattern combined with `DisallowGarbageCollectionScope`;
- no compaction, because there are no compactable spaces;
- write-barrier errors are caught only by debug V8.

Item 5 shows that debug V8 is a workable CI configuration on Windows (start condition (iii)), using the recipe above.

## Logs

On the Mac:
- `m0/windows/logs_release_sandbox/`
- `m0/windows/logs_debug_sandbox/`

The verbose cargo build logs were not copied. All logs remain on the laptop in `C:\hgba-m0\logs\`, which includes the `build_*` attempt logs showing each failure.

## Laptop state after this run

- Kept:
  - fork source (`C:\hgba-m0\rusty_v8`)
  - release and debug archives (`t-rel`, `t-dbg`)
  - harness (`m0win`)
  - shadow SDK (`winsdk`)
  - scripts and logs
- Removed: the scheduled task `hgba_m0` (used to run jobs detached with `conhost --headless`) and the scratch folder `gntest`.
- **Final disk use of `C:\hgba-m0`: 15.95 GB**, split as rusty_v8 3.35, t-rel 4.28 and t-dbg 8.31. 324 GB is still free on C:.
