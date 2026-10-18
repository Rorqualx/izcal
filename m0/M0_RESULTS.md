# M0 results: local portion (macOS arm64)

Run on 14 October 2026 on an Apple M4 (10 cores, 24 GB), macOS 15.6, rustc 1.91.0 (pinned by rusty_v8's `rust-toolchain.toml`) and Chromium clang `llvmorg-23-init-19482-g53d18800-1` (rusty_v8's pin). Raw logs are in `logs/`.

**Result: every local step passed.** The fork builds in the sandbox configuration in both release and debug-V8. With it, the spike and the companion crate behave exactly as they did with the sandbox off. The predictable platform makes a 1,000-run V8 probe bit-for-bit deterministic.

## Per-step status

| # | Step | Status |
|---|---|---|
| 1 | Fork: shallow clone of `denoland/rusty_v8` at `v152.2.0` with submodules, branch `hgba` | **Done.** 1.1 GB, 48 s |
| 2 | Apply the v8-cppgc-ext upstream patch as a commit | **Done.** Applied cleanly. It is now **compiled inside the V8 build for the first time**, and its API tests pass natively in release and debug (`fork-api-verify/`) |
| 3 | Predictable platform in the fork's C++ glue plus a Rust API and a flags helper | **Done.** About 320 lines including a test. Clippy-clean and rustfmt-clean |
| 4 | Build from source, sandbox configuration, release (plus debug if disk allows) | **Done, both.** Release took 17 min 27 s and debug-V8 took 28 min 38 s |
| 5 | Rerun spike items 1–5 and the v8-cppgc-ext suite against the sandbox build | **Done.** All results match the sandbox-off originals. The companion crate needs a "fork mode" to act as a thin layer (see below) |
| 6 | Determinism probe, 1,000 runs | **1,000 of 1,000 hashes identical**, plus 50 of 50 fresh processes. Without the predictable platform and without `--predictable`, the GC-observable channel diverges (848 distinct hashes in 1,000 runs) |
| 7 | CI build-farm draft | **Written:** `ci/v8-build.yml`. Linux and Windows are **untested**. *Replaced on 17 October 2026 by `rusty_v8/.github/workflows/izcal-v8-build.yml`; see `ci/README.md`* |
| 8 | This report | Done |

## Fork commits (`git log --oneline hgba`, in `rusty_v8/`, local only and not pushed)

```
895b6e7 style: rustfmt src/cppgc.rs (cppgc patch was not formatted)
18b3335 platform: add d8-style predictable platform with embedder clock
6d065ee build: add third_party/disarm/src submodule (V8 DEPS pin 2d13d3f)
8843380 cppgc: expose GC scopes, heap state, statistics, prefinalizers, weak callbacks, testing heap and write barriers
2768994 v152.2.0 (#2069)
```

The total diff over `v152.2.0` is 9 files, +1,599 lines. Of that, the cppgc patch is about 1,200 lines and the platform about 320 lines including its test. `6d065ee` is the fix for rusty_v8#2070 on git checkouts:
- V8 enables its generated-code validator on arm64 sandbox builds with verification features, and the validator needs `//third_party/disarm`.
- The release build did not need it. **The debug build did.**

## Measurements

| | Value |
|---|---|
| Free disk at start | 44.2 GiB (46,325,840 KB) |
| Checkout (`--depth 1 --shallow-submodules`) | **1.1 GB** (`.git` 269 MB, `third_party` 656 MB, `v8` 198 MB), 48 s |
| Extra downloads | Chromium Rust toolchain 1.3 GB (`third_party/rust-toolchain`, fetched by build.rs); libclang 21.1.8 arm64 150 MB (LLVM release tarball, stream-extracted); gn/ninja 2.5 MB. Chromium clang (262 MB) was reused from `~/.cache/v8-cppgc-ext` |
| **Release sandbox build** | **1,047 s (17.5 min)** wall time, 2,466 ninja steps, no sccache |
| Release build disk | Repo grew from 1.17 GB to 3.9 GB: `target/` 1.5 GB (of which `gn_out` 1.0 GB) plus the Rust toolchain. Free space fell by 4.8 GB. Guard minimum free: 35.9 GiB |
| **Debug-V8 sandbox build** | **1,718 s (28.6 min)**, 2,482 steps (adds disarm and the validator) |
| Debug build disk | `target/debug` 2.9 GB (`gn_out` 1.7 GB); repo peak 7.3 GB. Guard minimum free: 30.8 GiB |
| Archives | Release: `librusty_v8_ptrcomp_sandbox_release_aarch64-apple-darwin.a` 161 MB (41.7 MB gzip). Debug: 450 MB (111.8 MB gzip). The bindgen output (`src_binding*.rs`) is identical for release and debug |
| Peak for everything (both builds, all test targets) | About 23 GB under `m0/`, about 20 GB free at the low point. The 15 GB guard never tripped |

The facts doc estimated 5–10 GB and 20–60 min. The measurements fall at the low end of that range. A build machine needs about 10 GB per platform for release plus debug, without sccache.

**Environment trap on Apple Silicon.** `/usr/local/bin/python3` and Homebrew on this Mac are x86_64 (Rosetta). rusty_v8's `tools/*.py` choose gn, ninja, clang, rust and libclang host packages from `platform.machine()`, so the build was run with a native arm64 Python 3.12 (`scripts/env.sh`).

Homebrew's libclang is x86_64 and cannot be loaded by an arm64 bindgen. The Chromium `libclang` package for Mac contains only Python bindings. So libclang came from the LLVM 21.1.8 release instead.

## Step 3: predictable platform (`src/binding.cc`, `src/platform.rs`, `src/V8.rs`)

```rust
v8::V8::set_predictable_flags(seed);  // --predictable --single-threaded --random-seed=<seed>
let platform = v8::new_predictable_platform(
    Some(Box::new(|| virtual_seconds())),   // impl PlatformClock; None = d8's synthetic clock
    v8::WorkerTaskMode::Queued,             // or Inline
    1,                                      // NumberOfWorkerThreads() reported to V8
).make_shared();
v8::Platform::run_worker_tasks(&platform);    // embedder drains worker tasks on its thread
v8::Platform::pending_worker_tasks(&platform);
```

- It is a `DefaultPlatform` subclass with no worker threads. Subclassing is required because `v8::platform::PumpMessageLoop` and `NotifyIsolateShutdown` `static_cast` the platform to `DefaultPlatform`.
- It is modelled on d8's `PredictablePlatform`:
  - Worker and job tasks run either inline or queued for the embedder.
  - Delayed worker tasks are queued on the platform clock. d8 drops them instead.
- `MonotonicallyIncreasingTime` and `CurrentClockTimeMillis` (which `Date.now` reads through `CurrentClockTimeMilliseconds`) call a Rust `PlatformClock`.
  - Any `Fn() -> f64 + Send + Sync` that returns seconds is a clock.
  - The wall clock defaults to `monotonic × 1000`, as in d8.
  - The foreground task runners use the same clock (`SetTimeFunctionForTesting`), so delayed foreground tasks run only when virtual time advances.
- Test: `tests/test_predictable_platform.rs` checks:
  - `Date.now()` follows the virtual clock exactly (1000000000 → 1000000001 after +1.5 ms);
  - two isolates give identical results;
  - no worker task is left pending.
- It also passes on debug V8.

## Step 5: spike and companion crate under the sandbox

Setup:
- The consumers point at the fork with `[patch.crates-io] v8 = { path = "../rusty_v8" }`.
- The built archive is supplied through `RUSTY_V8_ARCHIVE` and `RUSTY_V8_SRC_BINDING_PATH` (`scripts/consume-release.sh`, `scripts/consume-debug.sh`), so V8 is not rebuilt.
- Copies of the code live in `spike-sandbox/` and `v8-cppgc-ext-sandbox/`. The originals are untouched.

### Spike rerun vs original (original = crates.io prebuilt, sandbox off)

| Item | Original release / debug | Sandbox release (fork) | Sandbox debug-V8 (fork) |
|---|---|---|---|
| 1 detached-quiescent / detached-marking | 0 finalized in scope, 0 reader errors; PASS | 0 / 0, 3,191 / 3,582 refused GCs; **PASS** | **PASS** |
| 1 unified-marking / unified-alloc-only | NoGC scope does **not** stop V8 finalization (5,000 in scope) | **same** (5,000 in scope, readers fine) | **same** |
| 1 unified-blocked-owner | 10,244 phases, 0 finalizations / debug 1,396, 0 | **15,006 phases, 0 finalizations, 0 errors** | **1,410 phases, 0** |
| 1 unified-disallow | `Check failed: !IsGCForbidden()` | **same** | **same** |
| 1 compaction-probe | release: addresses stable; debug: `DCHECK(!enable_for_next_gc_for_testing_)` | addresses stable | **same DCHECK** |
| 1 latency | 2.04 ns/pair; p99 42 ns; unified-marking max 65 µs | 2.35 ns/pair; p99 42 ns; max 10.5 µs | — |
| 2 cross-thread raw reads | 30 s: 9,540 cycles, 0 in-phase finalizations, 0 errors / debug 20 s: 3,120, 0, 0 | **8,931 cycles (8,861 marking), 0, 0; PASS** | **3,187 cycles, 0, 0; PASS** |
| 3 DOM↔JS edges | PASS (3a–3d) | **PASS** (3d leaks as expected) | **PASS** |
| 4 Rust-owned data | PASS, 100/200 prefinalizers, `Drop` on main thread | **PASS**, identical counts | **PASS** |
| 5 write barrier | release misses the omitted barrier; debug: `MarkingVerifier: Encountered unmarked object` | release **misses** (as before) | **Caught**: same `marking-verifier.cc:42` fatal, `TNode → TNode` hint |
| 6 data plane | cppgc 128.9 / 141.8 / 136.7 B per node; DP share 26.2 / 28.3 / 26.1% | **identical** 128.9 / 141.8 / 136.7 B; 26.2 / 28.3 / 26.1%; oracle OK, negative control detected | — |

- **The V8 sandbox (with pointer compression on) does not change any spike outcome on macOS arm64.**
- The `Member` size stays at 4 bytes.
- Spike items 1–5 used the spike's own mangled-symbol bindings, and they still link against the fork's archive.

### v8-cppgc-ext against the fork

| Check | Result |
|---|---|
| Fork's native API (`fork-api-verify/`: the upstream patch's own tests, no shim) | **2 of 2 pass**, release and debug, sandbox. This is the first time the patch's `binding.cc` hunk has been compiled inside V8's build |
| Crate unchanged + fork | **Does not compile.** As the README predicted, the fork's inherent methods shadow the extension-trait methods, so e.g. `heap.collect_statistics(v8_cppgc_ext::DetailLevel)` hits a type mismatch: 53 errors across the tests and examples. With that call written in UFCS, `cargo test --test abi` **links and passes 5 of 5 under sandbox**. However, the shim's `BEGIN UPSTREAM` C functions are then defined twice, in the shim archive and in `librusty_v8.a`. The linker silently picks one. That is harmless only while the two texts are identical |
| **Crate in "fork mode"** (`--features "v8_enable_sandbox rusty_v8_fork"`, patch in `v8-cppgc-ext-fork-mode.patch`) | **28 of 28 integration tests pass in release-sandbox and in debug-V8-sandbox**, including `cargo test --test abi`: V8 accepted the shim's sandbox, pointer-compression and 31-bit-Smi configuration bits, and the `Member` round trips and libc++ statistics matched. The ported spike examples item1 and item4 also pass (13,027 blocked-owner phases, 0 finalizations) |

**What fork mode changes:**
- The old `lib.rs` moves unchanged to `standalone.rs`.
- The new `fork.rs` (190 lines):
  - re-exports `v8::cppgc::{HeapHandle, NoGarbageCollectionScope, …, testing, heap_consistency}`;
  - keeps `CppHeapExt`, `VisitorExt` and `IsolateExt` as forwarders;
  - adds `LivenessBrokerExt::is_heap_object_alive_raw` and the raw `register_weak_callback`, which the fork's Rust API does not wrap (their C functions are in the fork);
  - keeps the `abi` checks.
- `build.rs` defines `V8CE_UPSTREAM_IN_RUSTY_V8`, which removes the UPSTREAM block. The shim drops from 45 exported functions to **7**, all of them ABI checks.

**Remaining differences in fork mode:**
- The 4 doc tests (`compile_fail` checks for `!Send`) live in `standalone.rs` and do not run in fork mode. The fork's types carry the same markers.
- The scope guards' `heap_handle()` getter is not in the fork.

**Recommendation:** make fork mode the default once the fork is the plan of record. Long term, retire the crate down to its `abi` checks, which still matter under S-4e as a cross-check of the defines.

### Upstream `test_api` under sandbox (an extra check)

**290 of 291 rusty_v8 `test_api` tests pass in release-sandbox.** `test_fast_calls_pointer` dies with SIGTRAP inside JIT code (`EXC_BREAKPOINT` in the code space) on both release and debug V8. That test is a Fast API call with a `Type::Pointer` argument and return.

Upstream has no sandbox CI, so this is untested upstream territory. **Implication for bindings:** generated WebIDL fast-call paths must not pass raw pointers. They should use wrapped cppgc objects (`Object::wrap` / CppHeapPointer). This needs root-causing before M2.

## Step 6: determinism probe (`determinism-probe/`)

**Program** (`src/probe.js`): 60 steps. Each step does the following:
- reads `Math.random()` and `Date.now()`;
- allocates 200 linked nodes, all registered in a `FinalizationRegistry`, with every 25th node behind a `WeakRef`;
- keeps some chains alive for a while;
- creates 3,000-object allocation churn, so V8 runs **allocation-triggered GCs** (finalizations are visible at steps 2 and 8, before any forced GC);
- runs a JIT-hot loop.

**Embedder loop between steps:**
1. advance the virtual clock by 16 ms;
2. pump foreground tasks (FinalizationRegistry cleanup), drain worker tasks, and run a microtask checkpoint (which clears WeakRef keep-alive sets);
3. every 10 steps, a forced full GC, then drain again.

**Channels hashed separately (FNV-1a):** R (`Math.random`), D (`Date.now`) and G (WeakRef liveness counts, FinalizationRegistry counts, last tokens and the full finalization order).

Each run uses a fresh isolate. All runs use `--random-seed=42` and `--expose-gc`, the sandbox release build, and in-process repetition plus fresh-process repetition.

| Variant | In-process | Fresh processes | R | D | G |
|---|---|---|---|---|---|
| **`pred`**: predictable platform (virtual clock, queued workers) + `--predictable --single-threaded` | **1,000 / 1,000 identical** | **50 / 50 identical, same hash as in-process** | 1 | 1 | 1 |
| `pred-inline`: same, worker tasks inline | 1,000 / 1,000 identical | 50 / 50 | 1 | 1 | 1 (**different** G than `pred`) |
| `pred-noflags`: predictable platform, **no** `--predictable`/`--single-threaded` | 1,000 / 1,000 identical | 50 / 50 | 1 | 1 | 1 (different G) |
| `default-flags`: default platform + `--predictable --single-threaded` | D differs every run | D differs | 1 | 1,000 | **1** (= `pred`'s G) |
| `default`: default platform, seed only | all differ | all differ | 1 | 1,000 | **848 distinct** (50 / 50 fresh) |
| Debug V8: `pred` (200 + 20 fresh), `pred-noflags` (200) | identical, **and identical to the release-V8 hashes** | | 1 | 1 | 1 |
| Debug V8: `default-flags` (200) | | | 1 | 200 | **2 distinct** (198 match `pred`; 2 match `pred-inline`) |

Worker tasks that V8 still posts, as counted by the embedder: **8 per run under `--predictable --single-threaded`**, and 152 per run without the flags.

**Conclusions:**
1. **Yes: with the forked predictable platform, a fixed virtual clock and a fixed seed, all 1,000 hashes match**, across processes and across release and debug V8.
2. **Without `--predictable` the result still holds when the predictable platform is used.** Removing worker threads and virtualizing the clock was enough here. The GC-observable channel is deterministic but different from the `--predictable` value, because the flags change GC heuristics.
3. **Without the platform, the GC channel diverges.** With the default platform and no flags, 1,000 runs gave 848 distinct hashes. With the default platform plus `--predictable --single-threaded`, it diverged in 2 of 200 debug runs and 0 of 1,000 release runs. In those runs V8's few remaining worker tasks race on real threads, and the outcome flips between the "queued" and the "inline" results.
4. **So the flags alone are not sufficient, and the platform is the essential part.** `--single-threaded` does not stop all worker tasks. `Date.now` is only controllable through the platform clock. `Math.random` is fixed by `--random-seed` alone.
5. **The worker-task policy is part of the determinism key.** Inline and queued are each deterministic but give different GC observables. A DST seed must record the policy (and the flags), like any other configuration input.

**Limits.** This is one synthetic script, not a corpus page, and there is no DOM, cppgc or wasm in it. Section 3.3's real probe (DOM serialization hashes on corpus pages under the simulation substrate) remains M1 work.

## Step 7: CI draft

*Historical: this draft was replaced by the fork's `izcal-v8-build.yml` (see `ci/README.md`).* The draft was `ci/v8-build.yml`. It builds `hgba` with `submodules: recursive`, in the sandbox configuration, for these targets:

| Target | Variants | Status |
|---|---|---|
| macos-arm64 | release, debug | Commands exercised by hand here |
| linux-x86_64 | release, debug | **Untested** |
| windows-x86_64 | release, plus a **"verify"** variant | **Untested** |

*Superseded by the Windows M0 run (`windows/M0_WINDOWS_RESULTS.md`): a real debug V8 does build and link on Windows with `GN_ARGS=is_debug=true` plus RUSTFLAGS for `libcmtd`, so the draft's "verify" variant should be replaced by that recipe. Original text follows.* The Windows "verify" variant exists because **rusty_v8 cannot link a debug V8 on Windows** (`build.rs` forces `is_debug=false`). It is a release build with `v8_enable_verification_features`, `v8_dcheck_always_on` and `v8_enable_debugging_features` set. That turns on `CPPGC_VERIFY_HEAP`.

The workflow runs the fork's tests, then publishes the following under the file names that build.rs expects for `RUSTY_V8_MIRROR`:
- `librusty_v8_ptrcomp_sandbox_{release,debug}_<target>.a.gz`
- `src_binding_ptrcomp_sandbox_*.rs`
- `args.gn`
- `SHA256SUMS`

## What remains of M0 (cannot be done on this machine)

*Written when only macOS was done. Status as of 14 October 2026 is in `M0_SUMMARY.md`: Linux and Windows builds and runs are done, Windows debug V8 works from source, TSAN works on Linux, and the items below marked "still open" remain.*

- **Linux x86-64 and Windows x86-64 builds and runs.** Start condition (i) requires the spike and companion suite on three platforms; only macOS arm64 is done.
  - On Windows, v8-cppgc-ext's `build.rs` still panics (clang-cl/MSVC flags not worked out).
  - Windows also needs the "verify" configuration in place of debug V8.
  - **Done since:** both platforms passed in release and debug, Windows through a compat crate over the fork rather than the shim, and with real debug V8.
- **The CI farm itself:** runners with about 10–15 GB free per job, sccache, a mirror host, and a first run of `ci/v8-build.yml`. The fork is local only, so it has to be pushed to a project remote. The debug-V8 CI configuration required by start condition (iii) exists only as this draft. **Still open.**
- **The `release-sandbox + TSAN` configuration** (design doc Section 7.1) was not attempted here. **Done since on Linux** (works under the sandbox; needs suppressions for V8's uninstrumented code).
- **The `test_fast_calls_pointer` sandbox crash** needs root-causing, and possibly an upstream issue. **Still open.**
- **Non-technical M0 items:** naming the `rusty_v8` owner, and staffing M1–M2. **Still open** (staffing blocked on D-S8).
- The debug build was **not** skipped.

## Disk at the end

- **Kept:**
  - the fork source (`rusty_v8/`, 1.1 GB);
  - the archives (`archives/`, 735 MB: both `.a` and `.a.gz`, plus bindings, args and SHA256SUMS);
  - libclang 21 (`tools/llvm21`, 150 MB);
  - logs;
  - the source of the consumer crates.
- **Deleted:** all cargo `target*` directories (about 21 GB, rebuildable from the archives in under a minute each) and the downloaded Chromium Rust toolchain (1.3 GB; build.rs downloads it again on the next build).
- **Totals:** `m0/` is 2.0 GB. Free space right after cleanup was 21.3 GB. A Time Machine local snapshot (`2026-10-14-154712`) still holds the deleted blocks, and macOS releases that space automatically.

## Layout

```
rusty_v8/                 fork (branch hgba), shallow, with submodules
archives/                 sandbox release + debug archives for aarch64-apple-darwin
scripts/env.sh            from-source build environment (arm64 Python, clang, libclang)
scripts/consume-*.sh      use the archives without building V8
scripts/guard.sh          disk guard used for the builds
spike-sandbox/            copy of cppgc-spike pointed at the fork (logs/ = sandbox runs)
v8-cppgc-ext-sandbox/     copy of v8-cppgc-ext with fork mode
v8-cppgc-ext-fork-mode.patch
fork-api-verify/          upstream patch API tests against the fork, no shim
determinism-probe/        step 6
ci/README.md              step 7 (draft replaced by rusty_v8/.github/workflows/izcal-v8-build.yml)
logs/                     build logs, disk-guard samples, test and probe output
```
