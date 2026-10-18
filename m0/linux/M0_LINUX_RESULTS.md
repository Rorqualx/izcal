# M0 Linux x86-64: rusty_v8 fork, sandbox build, cppgc spike rerun

Host: huey (Ubuntu 24.04, kernel 6.8, 8 cores, 31 GB RAM, shared machine), 14 October 2026. Everything lives under `~/hgba-m0/` on huey. No sudo was used, and no containers or system configuration were touched.

## Summary

- **The fork builds from a git checkout with the sandbox on, in release and debug.** No crates.io package was involved, so the rusty_v8#2070 gaps did not apply. The one exception is a missing `fadec` source on x64 (see Problems).
- **The cppgc-ext patch compiles inside the V8 build.** The macOS work could not show that (see v8-cppgc-ext README, "binding.cc part has NOT been compiled inside rusty_v8's V8 build"). All patch symbols are defined in `librusty_v8.a`.
- **Tests:**
  - 25 of 25 pass against the release archive.
  - 25 of 25 pass against the debug archive. Two of those return early on debug V8 by design, exactly as on macOS: forced compaction, and the no-barrier control.
  - The 25 include the patch's own `upstream_api` tests.
- **Spike items 1–5 reproduce the macOS (sandbox-off, prebuilt) results under the sandbox on Linux.** They run against the native fork API: no shim and no mangled symbols.
- **Item 5:** debug V8 catches the omitted barrier (`MarkingVerifier: Encountered unmarked object`); release misses it silently.
- **TSAN:** item 2 runs under TSAN despite the 1 TiB sandbox reservation. The ~140 reports per run come from uninstrumented V8 internals; none touch the spike's worker read path. The racy-control run is detected. See item 2.

For HGBA-2.1 start condition (i) (spike rerun under `v8_enable_sandbox`), Linux x86-64 is **met**. Exposure route = fork (`hgba` branch).

## Fork

| | |
|---|---|
| Checkout | `~/hgba-m0/rusty_v8`, `denoland/rusty_v8` tag `v152.2.0` (commit 2768994), `--depth 1`, submodules `--depth 1 --recursive` (56 s) |
| Branch | `hgba`. Commit `38e7dda`: "cppgc: expose scopes, HeapState, statistics, prefinalizers, precise testing GC (v8-cppgc-ext upstream patch)". `rusty_v8-152.2.0-cppgc-ext.patch` applied cleanly with `git apply`: binding.cc +366, cppgc.rs +803, isolate.rs +29. |
| Extra (untracked) | `third_party/fadec/src` @ `c9f78f532b9004de278489019ab2c6c28ae9746e` (from `v8/DEPS`). Needed by the debug sandbox build; see Problems. |
| V8 | 15.2.124.1-rusty |

### Build configuration

Scripts are in `~/hgba-m0/scripts/` (`env.sh`, `build_v8.sh`, `monitor.sh`, `tests_env.sh`).

```
V8_FROM_SOURCE=1  CARGO_BUILD_JOBS=6 (→ NUM_JOBS → ninja -j6; debug resumed at 3)
LIBCLANG_PATH=~/hgba-m0/llvm22/lib        # libclang 22.1.8 (LLVM release tarball)
CLANG_PATH=~/hgba-m0/llvm22/bin/clang     # REQUIRED as well, see Problems
CLANG_BASE_PATH=~/hgba-m0/chromium-clang  # Chromium clang llvmorg-23-init-19482-g53d18800-1 (rusty_v8's pin)
RUSTUP_TOOLCHAIN=stable (1.98.1)
nice -n 15 ionice -c3 cargo build [--release] --lib --features v8_enable_sandbox
```

The resulting `args.gn` (both profiles, besides `is_debug`):
- `use_custom_libcxx=true`
- `v8_enable_sandbox=true`
- `v8_enable_external_code_space=true`
- `v8_enable_pointer_compression=true`
- `v8_enable_v8_checks=false`
- `treat_warnings_as_errors=false`. This is set because a system `CLANG_BASE_PATH` was used.

The bindgen defines include `V8_ENABLE_SANDBOX`, `V8_COMPRESS_POINTERS` and `V8_COMPRESS_POINTERS_IN_SHARED_CAGE`.

At runtime the process VmSize is about **1.45 TB** (`/proc/<pid>/status` during item 2). That is the sandbox reservation.

### Build cost

| | Release (`is_debug=false`) | Debug (`is_debug=true`) |
|---|---|---|
| Ninja steps | 2,456 | 2,472 |
| Wall time | **45 min 42 s** (2,742 s), -j6, under load from other users (load 8–12). The bindgen failure (see Problems) cost a 12 s rerun. | **63 min** (3,808 s) in total, over three segments: 1,411 s at -j6 under heavy swap throttling (interrupted), 1,192 s at -j6 (paused by the guard and stopped), then 1,205 s at -j3. On a quiet host the time is probably about 30–35 min at -j6. That is an estimate (debug compiles are cheaper than release, and step rates in the unthrottled stretches suggest it). |
| CPU time | 14,035 s user + 675 s sys | ≥ 3,450 s user for the last segment. Earlier segments were not measurable after the interruption. |
| Largest single process (`/usr/bin/time` maxRSS) | 1.5 GB | 2.2 GB |
| Peak aggregate build RSS (10 s samples) | 4.7 GB at -j6 | 6.4 GB at -j6 (maglev files at about 1–1.3 GB each); 4.0 GB at -j3 |
| `librusty_v8.a` | 202 MB (`.gz -9`: **41 MB**) | 536 MB (`.gz -9`: **87 MB**) |
| `gn_out` + cargo target | 1.6 GB | 2.9 GB |

**One-time disk:**

| | Size |
|---|---|
| Checkout | 2.6 GB, of which 1.4 GB is the Chromium Rust toolchain fetched by `tools/rust_toolchain.py` |
| Chromium clang | 347 MB |
| libclang 22 subset | 577 MB |
| Nightly toolchain (TSAN) | 685 MB, in `~/.rustup` |

**Final total under `~/hgba-m0`: 15 GB**, including three test target directories (6.1 GB) that can be deleted. No cleanup was done; the total is far below the 60 GB threshold.

### Archives (`~/hgba-m0/archives/`, with `SHA256SUMS`)

- `librusty_v8_ptrcomp_sandbox_release_x86_64-unknown-linux-gnu.a` (+ `.a.gz`)
- `librusty_v8_ptrcomp_sandbox_debug_x86_64-unknown-linux-gnu.a` (+ `.a.gz`)
- `src_binding_ptrcomp_sandbox_{release,debug}_x86_64-unknown-linux-gnu.rs`. The two are byte-identical.

The names follow rusty_v8's `prebuilt_features_suffix()` convention.

Both `.a.gz` files are under 500 MB and were copied to the Mac next to this file.

**Consumption was verified the way developers will get it from a mirror.** The tests build *without* `V8_FROM_SOURCE`, using:
- `RUSTY_V8_ARCHIVE=<archive>.a`
- `RUSTY_V8_SRC_BINDING_PATH=<src_binding>.rs`
- `features = ["v8_enable_sandbox"]`

## How the spike was ported (no mangled symbols)

`~/hgba-m0/m0-tests/` is a workspace with `v8 = { path = "../rusty_v8", features = ["v8_enable_sandbox"] }`.

- **`compat/`:** a small facade crate named `v8-cppgc-ext`. It **re-exports the fork's native `v8::cppgc` API** (`HeapHandle`, `NoGarbageCollectionScope`, `DisallowGarbageCollectionScope`, `HeapState`, `HeapStatistics` / `DetailLevel`, `PreFinalizer`, `LivenessBroker`, `WeakCallbackMethod`, `testing::*`, `heap_consistency::*`). It also provides the old extension-trait names (`CppHeapExt`, `IsolateExt`, `VisitorExt`), which forward to the fork's inherent methods.
  - The only `extern "C"` declarations are two C-ABI functions exported by the patched `binding.cc`: the raw weak callback and the raw liveness query.
  - No C++ shim is compiled, and nothing is linked by Itanium-mangled name.
  - This is the "v8-cppgc-ext becomes a compatibility layer over the fork" state from Stage 1 §7.1 (draft 0.9 wording). Since HGBA-2.1.6 S-4e the crate is retired: its remaining raw entry points and tests move into the fork, and no ABI-check layer is kept.
- **Tests and examples copied unchanged:**
  - From `v8-cppgc-ext`: tests `scopes`, `statistics`, `prefinalizer`, `testing_api`, `heap_consistency`, and examples `spike_item1/2/4/6`.
  - From the patch: `upstream/verify/tests/upstream_api.rs`, minus its `use v8_cppgc_ext as _` shim line.
  - `tests/abi.rs` was not run. It tests the out-of-tree shim's ABI matching, which does not exist here.
- **Items 3 and 5:** `cppgc-spike/src/bin/item{3,5}*.rs` moved from the spike's `mod ffi` helpers to `examples/spike/common.rs`, which is fork-API based. Only import lines and `TestingHeap::new(&heap)` changed.
- `nm` on the archive confirms that all patch symbols are defined in `librusty_v8.a`: `cppgc__NoGarbageCollectionScope__*`, `cppgc__HeapState__*`, `v8__CppHeap__CollectStatistics`, `cppgc__RustObj__RegisterPrefinalizer`, `cppgc__testing__*`, `cppgc__HeapConsistency__*` and `v8__Isolate__RequestGarbageCollectionForTestingWithStackState`.

## Test results

`cargo test [--release] -- --test-threads=1`. Logs: `~/hgba-m0/logs/{release,debug}_cargo_test.log`.

| Test file | Release sandbox | Debug sandbox |
|---|---|---|
| `heap_consistency.rs` (4) | 4 ok | 4 ok (`control_without_barrier_loses_object` returns early on debug V8: the verifier aborts by design) |
| `prefinalizer.rs` (5) | 5 ok | 5 ok |
| `scopes.rs` (5) | 5 ok | 5 ok |
| `statistics.rs` (3) | 3 ok | 3 ok |
| `testing_api.rs` (6) | 6 ok | 6 ok (`forced_compaction_does_not_move_objects` returns early on debug: `DCHECK(!enable_for_next_gc_for_testing_)`) |
| `upstream_api.rs` (2, the patch's own tests) | 2 ok | 2 ok |
| **Total** | **25/25** | **25/25** (2 early returns, same as macOS) |

## Spike items 1–5 vs macOS

**Logs** (in `~/hgba-m0/logs/items/`):
- `release_*`: the first release run, made while the debug build was compiling.
- `release_quiet_*`: release items 1–2 rerun on a quiet host (other users' load about 3, no build running).
- `debugv8_*`: runs against the debug archive.

**Columns:** the macOS column is sandbox off, prebuilt, through mangled symbols or v8-cppgc-ext (`cppgc-spike/RESULTS.md`, `v8-cppgc-ext/README.md`). The Linux columns are sandbox on, from source, through the native fork API.

| Item / scenario | macOS arm64 | Linux x86-64 release | Linux x86-64 debug V8 | Same? |
|---|---|---|---|---|
| 1 `detached-quiescent` | 0 finalized in scope, 0 reader errors, 5,000 after; PASS | 1,054 GC requests refused, 0 finalized, 64,569 traversals, 0 errors, 5,000 after; PASS | 977 refused, 0 finalized; PASS | yes |
| 1 `detached-marking` | marking progresses, finalization deferred; PASS | 904 steps, marker done, 0 finalized, 0 errors; PASS | 1,028 steps, 0 finalized; PASS | yes |
| 1 `unified-marking` / `unified-alloc-only` | **5,000 finalized inside the NoGC scope**; readers fine | **5,000 finalized inside the scope** (both); 0 live finalized, 0 errors | same: 5,000 / 5,000 | yes |
| 1 `unified-blocked-owner` (both scopes, owner blocked) | 10,244 (spike) / 13,302 (port) phases, **0 finalized**, 0 errors | 7,380 phases, all under marking, **0 finalized**, 0 errors, 1.48 M garbage collected between phases; PASS | 808 phases, **0 finalized**, 0 errors; PASS | yes |
| 1 `unified-disallow` | `Check failed: !IsGCForbidden()` | same (Signal 5) | same, `cpp-heap.cc:913` | yes |
| 1 `compaction-probe` | release: addresses stable; debug: DCHECK in `Compactor` | addresses stable, tree intact | `compactor.cc:470 Debug check failed: !enable_for_next_gc_for_testing_` | yes |
| 1 latency (quiet) | 2.0–2.5 ns per enter+leave pair; p99 42 ns (41.7 ns timer) | 4.0 ns per pair; detached p50/p99 16/17 ns; under marking 20/73 ns; unified under V8 marking 17/58 ns, max 12 µs | n/a | consistent (x86 has a finer timer) |
| 2 (30 s release / 20 s debug) | 10,364 / 3,268 cycles, 0 finalized in worker phases, 0 errors | quiet: 8,212 cycles (8,148 under marking), 131,392 traversals, 89,873 finalized, **0 in worker phases, 0 errors**; PASS (loaded run: 3,547 cycles, same zeros) | 2,329 cycles, **0 / 0**; PASS | yes |
| 3 DOM↔JS edges (3a, 3a′, 3b, 3c precise, 3d negative) | PASS (3c fails only with a conservative stack) | PASS `[true,true,true,true]`; `ITEM3_CONSERVATIVE=1` → 3c FAIL, as on macOS | PASS | yes |
| 4 Rust-owned data + prefinalizers | 5,000 → 10,000 drops, Arc 5,001 → 1, 100 → 200 prefinalizers, `Drop` on the main thread only; PASS | identical counts; `Drop` on ThreadId(1) only; unified V8-scheduled GC reached 5,000 drops in 147–350 ms (macOS 39 ms; load-dependent); PASS | PASS | yes |
| 5 write barrier, with barrier | B survives | B survives | B survives | yes |
| 5 write barrier, **omitted** | release: **missed**, dangling Member; debug: **caught** by `MarkingVerifier` | **missed** (B finalized while A still points at it), exit 0 | **caught**: `Fatal error in v8/src/heap/cppgc-internal/marking-verifier.cc:42 MarkingVerifier: Encountered unmarked object.` (Signal 5) | yes |

- **No behavioral difference** was observed between macOS sandbox-off and Linux sandbox-on.
- The sandbox does not change any cppgc semantic tested here:
  - the scope semantics;
  - no compaction (the heap has no custom spaces);
  - the prefinalizer ordering;
  - the verifier.
- Member is 4 bytes on both platforms (cppgc pointer compression).

### Item 2 under TSAN (optional)

**Setup:** nightly 1.101.0 (2026-10-13), `RUSTFLAGS=-Zsanitizer=thread -Zbuild-std --target x86_64-unknown-linux-gnu --release`, against the *uninstrumented* release sandbox archive. The build took 35 s.

- **The 1 TiB sandbox reservation did not break TSAN on Linux x86-64.** The process started, ran and passed.
- **Clean run** (`--secs 10`):
  - RESULT PASS: 666 cycles, 0 finalized in worker phases, 0 errors.
  - TSAN reported **140 warnings**, and **0 of them involve a spike worker-thread frame**.
  - 124 are purely inside V8: the platform worker vector, ConcurrentMarking, BytecodeAnalysis, FreeList and so on.
  - 14 are `rusty_v8_RustObj_trace`, run by cppgc's concurrent marker, against the main thread's initialization of the object in `make_garbage_collected`. The happens-before for that pair lives in uninstrumented cppgc atomics, so TSAN cannot see it.
  - These are artifacts of mixing instrumented and uninstrumented code, not findings.
  - This differs from macOS, which reported 0 warnings. On Linux, TSAN's interceptors also catch the statically linked V8's `malloc`/`memcpy`, so V8 memory accesses become visible without V8's synchronization.
- **`--racy-control`:** the deliberate unsynchronized worker write **was detected**. 1 report has a spike worker-closure frame.
- **Conclusion:** usable TSAN coverage of the Rust side on Linux needs either a suppression file for V8 frames or a from-source `is_tsan=true` V8 build. Without one, the noise floor is about 14 warnings per second.
- Item 1 was not run under TSAN.

Logs: `items/tsan_item2_smoke.log`, `items/tsan_item2_racy_control.log`, `tsan_build.log`.

## Problems found (precise)

1. **libclang ≥ 21.1 is required, but Ubuntu 24.04's system libclang is 18.1.3.**
   - The sudo fix would be `apt install libclang-21-dev` from apt.llvm.org. Noble's own archive stops at `libclang-20-dev` 20.1.2, so a third-party repository would be needed.
   - Chromium's `tools/clang/scripts/update.py --package libclang` ships **only the Python bindings on Linux**, with no `libclang.so`. It cannot be used.
   - **Workaround used, no sudo:** stream `LLVM-22.1.8-Linux-X64.tar.xz` from the GitHub release (1.85 GB) and extract only `lib/libclang.so*`, `lib/clang/*` and `bin/clang*` (577 MB, about 2 min).
2. **`LIBCLANG_PATH` alone is not enough: `CLANG_PATH` is required too.**
   - bindgen 0.72 also runs the first `clang` executable it finds, which was `/usr/bin/clang` (18), to detect include paths.
   - That appended `/usr/lib/llvm-18/lib/clang/18/include` after clang 22's builtin headers.
   - The result was that libc++'s `<cstdint>` failed with `reference to unresolved using declaration` and `cast from pointer to smaller type 'int'` (43 errors in `src/binding.hpp`), *after* the full 45-minute ninja build had already succeeded.
   - **Fix:** `CLANG_PATH=<clang ≥21>/bin/clang`. Add it to the build farm's environment, and to rusty_v8's README next to `LIBCLANG_PATH`.
3. **x64 sandbox debug builds need `third_party/fadec/src`.** This is the x86 analogue of #2070's `disarm`.
   - The cause: `v8_enable_generated_code_validator = v8_enable_verification_features && v8_enable_sandbox` (true in debug) depends on `//third_party/fadec` on x64.
   - The rusty_v8 git checkout has `v8/third_party/fadec/BUILD.gn` but not the source; the `.gitmodules` has no entry for it.
   - It was fetched manually at the commit pinned in DEPS, and it compiled (`obj/third_party/fadec/libfadec.a`).
   - **The fork should add it as a submodule.** It is not needed for release builds.
4. **`rustfmt` is missing on the stable toolchain.** bindgen prints `error: 'rustfmt' is not installed` but continues. This is cosmetic, but the build farm should install rustfmt so that `src_binding.rs` is formatted.
5. **`rust-toolchain.toml` pins 1.91.0** with seven cross targets. This run overrode it with `RUSTUP_TOOLCHAIN=stable` (1.98.1) to avoid that download. The fork's CI should decide whether to keep the pin.
6. **Shared-host lesson: the cgroup cap, not `free`.**
   - On huey, every uid-1000 process shares `user.slice/user-1000.slice` with `memory.high = 9 GiB` and `memory.max = 11 GB`.
   - During the first debug segment, another job under the same uid (a vitest node process, about 6 GB) plus the -j6 build pushed the slice into reclaim. The result was heavy swapping (20–55 MB/s each way, iowait 54–76%) and near-zero build progress, while `free -g` still showed **18 GB available**.
   - The guard was rewritten (`scripts/monitor.sh`):
     - it watches `memory.high − memory.current` of the slice;
     - it SIGSTOPs ninja and clang when headroom is below 2.5 GiB and resumes above 4 GiB.
   - At -j6, debug V8's maglev translation units (about 1–1.3 GB each) alone exceeded the headroom. The guard paused the build and could not resume, because stopped processes keep their memory. The build was therefore restarted at **-j3** and finished without a pause.
   - **For the build farm:** size by cgroup limit. Plan about 1.3 GB per job for debug and 0.8 GB per job for release, plus headroom.
7. **`ionice -c3` (idle class) stalls the build completely when the disk is busy** (swap, other jobs). That is acceptable on a shared host, but it makes the build times above upper bounds.

## Reproduce

```
ssh hueyj
source ~/hgba-m0/scripts/env.sh                              # source-build env (V8_FROM_SOURCE, LIBCLANG_PATH, CLANG_PATH, ...)
JOBS=6 ~/hgba-m0/scripts/build_v8.sh release                 # → target-sandbox/release/gn_out/obj/librusty_v8.a
JOBS=3 ~/hgba-m0/scripts/build_v8.sh debug
source ~/hgba-m0/scripts/tests_env.sh release; cd ~/hgba-m0/m0-tests
cargo test --release -- --test-threads=1
$CARGO_TARGET_DIR/release/examples/spike_item1_nogc_scope    # items 1–5 likewise; item2: --secs 30
source ~/hgba-m0/scripts/tests_env.sh debug; cargo test -- --test-threads=1   # debug V8 = dev profile
```
