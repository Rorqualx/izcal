# v8-cppgc-ext

> **M0 sandbox copy.** This directory is the M0 copy of `../../v8-cppgc-ext/`, changed to build against the project's `rusty_v8` fork (`../rusty_v8`, via `[patch.crates-io]`) with the V8 sandbox on. The README below is the original's, unchanged except for this note and the sandbox-feature status. What differs from the original:
>
> - **Fork mode** (`--features "v8_enable_sandbox rusty_v8_fork"`): `src/fork.rs` re-exports the fork's native `v8::cppgc` API, the old `lib.rs` moved to `src/standalone.rs`, and the shim compiles only its 7 ABI-check functions. HGBA-2.1.4 made this a permanent ABI-check layer; **HGBA-2.1.6 S-4e retires the crate**, because every build is now from source and the APIs are compiled only inside the fork. This directory is kept as the M0 record.
> - The change from the original is captured in `../v8-cppgc-ext-fork-mode.patch`. It contains a git copy header, so apply it with `git apply`, not `patch`.
> - Results: `../M0_RESULTS.md` ("v8-cppgc-ext against the fork"). The canonical copy for anything not fork-specific remains `../../v8-cppgc-ext/`.

Safe Rust bindings for the cppgc (Oilpan) APIs that **rusty_v8 152.2.0 (V8 15.2.124.1)** does not expose.

They go through a small C++ shim, [`src/binding.cc`](src/binding.cc), which `build.rs` compiles against rusty_v8's own V8 headers and Chromium libc++ (`std::__Cr`). The defines are the ABI-relevant ones the prebuilt `librusty_v8.a` was built with.

- Only `extern "C"` functions and opaque pointers cross the boundary.
- No mangled C++ symbol is linked from Rust.
- No C++ standard-library layout is mirrored in Rust.

This replaces the test-grade `mod ffi` in `../cppgc-spike/src/lib.rs`.

> **Pinned:** `v8 = "=152.2.0"`, V8 15.2.124.1-rusty, Chromium clang `llvmorg-23-init-19482-g53d18800-1`. Any other rusty_v8 version needs the shim and the ABI checks re-validated.

## API surface

| C++ API | Rust (`v8_cppgc_ext::…`) | Status |
|---|---|---|
| `v8::CppHeap::GetHeapHandle` | `HeapHandle<'h>`: `HeapHandle::new(&heap)`, `CppHeapExt::heap_handle`, `IsolateExt::cpp_heap_handle`, `unsafe HeapHandle::from_isolate_unbounded` | exposed, tested |
| `cppgc::subtle::NoGarbageCollectionScope`: RAII and static `Enter`/`Leave` | `NoGarbageCollectionScope::new` (guard), `unsafe enter`/`leave` | exposed, tested |
| `cppgc::subtle::DisallowGarbageCollectionScope`: RAII, `Enter`/`Leave`, `IsGarbageCollectionAllowed` | `DisallowGarbageCollectionScope::{new, enter, leave, is_garbage_collection_allowed}` | exposed, tested |
| `cppgc::subtle::HeapState::{IsMarking, IsSweeping, IsSweepingOnOwningThread, IsInAtomicPause, PreviousGCWasConservative}` | `HeapState::*` | exposed, tested (all 5) |
| `v8::CppHeap::CollectStatistics(kBrief / kDetailed)` | `CppHeapExt::collect_statistics` → Rust-owned `HeapStatistics` / `SpaceStatistics` / `PageStatistics` / `FreeListStatistics` / `ObjectStatsEntry` | exposed, tested (both levels, detached and isolate heap) |
| `CPPGC_USING_PRE_FINALIZER` (`PrefinalizerRegistration`) | trait `PreFinalizer`, `make_garbage_collected_with_prefinalizer`, `unsafe register_prefinalizer` | exposed, tested |
| `cppgc::LivenessBroker::IsHeapObjectAlive` | `LivenessBroker::is_heap_object_alive(&impl GetRustObj<T>)`, `is_heap_object_alive_raw` | exposed, tested (in prefinalizers and in weak callbacks) |
| `cppgc::Visitor::RegisterWeakCallback(Method)` | trait `WeakCallbackMethod`, `VisitorExt::register_weak_callback_method`, `unsafe register_weak_callback` | exposed, tested |
| `v8::Isolate::RequestGarbageCollectionForTesting(type, EmbedderStackState)` | `IsolateExt::request_garbage_collection_for_testing_with_stack_state` | exposed, tested (full precise and conservative; minor) |
| `cppgc::testing::OverrideEmbedderStackStateScope` | `testing::OverrideEmbedderStackStateScope` | exposed, tested (attached heap; see notes) |
| `cppgc::testing::StandaloneTestingHeap::{StartGarbageCollection, PerformMarkingStep, FinalizeGarbageCollection, ToggleMainThreadMarking, ForceCompactionForNextGarbageCollection}` | `testing::StandaloneTestingHeap` | exposed, tested (all 5) |
| `cppgc::testing::IsHeapObjectOld` | `testing::is_heap_object_old` | exposed, tested |
| `cppgc::subtle::HeapConsistency::{GetWriteBarrierType, DijkstraWriteBarrier, SteeleWriteBarrier, GenerationalBarrier}` | `heap_consistency::{get_write_barrier_type, write_barrier, dijkstra_write_barrier, steele_write_barrier, generational_barrier}` | exposed, tested. The generational barrier is only exercised in its "does not fire" case, because young-generation GC is not enabled at runtime. |
| `HeapConsistency::DijkstraWriteBarrierRange`, `GenerationalBarrierForUncompressedSlot/ForSourceObject` | — | not exposed (no Rust use case: rusty_v8 has no inline GC collections) |
| Custom / compactable spaces | — | **deliberately not exposed** (see below) |
| "no compactable spaces" runtime check | `CppHeapExt::assert_no_compactable_spaces()` | added, tested |

All handle and guard types are `!Send + !Sync`, and scope guards borrow the heap. `compile_fail` doc tests check both properties.

### Usage

```rust
use v8_cppgc_ext::*;

// Detached heap (Heap::create)
let h = heap.heap_handle();
{
  let _no_gc = NoGarbageCollectionScope::new(h);
  let _fail_fast = DisallowGarbageCollectionScope::new(h);
  // hand raw pointers to workers, block until they finish
}
let stats = heap.collect_statistics(DetailLevel::Detailed);

// Isolate's CppHeap, held across JS execution
let h = unsafe { HeapHandle::from_isolate_unbounded(isolate) }.unwrap();
isolate.request_garbage_collection_for_testing_with_stack_state(
    v8::GarbageCollectionType::Full, v8::cppgc::EmbedderStackState::NoHeapPointers);

// Prefinalizer
impl PreFinalizer for MyNode {
  fn pre_finalize(&self, broker: &LivenessBroker) {
    if !broker.is_heap_object_alive(&self.parent) { /* … */ }
  }
}
let p = unsafe { make_garbage_collected_with_prefinalizer(&heap, MyNode { … }) };
```

### Semantics worth knowing

These come from tests and the V8 source.

- **`NoGarbageCollectionScope` does not stop V8-initiated GCs** from marking and finalizing an attached CppHeap. Ported spike item 1 shows it again: 5,000 objects were finalized inside the scope.
  - It only defers GCs that cppgc itself initiates.
  - To get "no finalization", hold `DisallowGarbageCollectionScope` as well, so that any GC becomes a `CHECK` crash, and keep the owning thread from triggering V8 GCs.
- **`OverrideEmbedderStackStateScope` applies to GCs that V8 runs for an attached heap.**
  - For a detached heap, V8 only stores the override until the heap is attached (`cpp-heap.cc`).
  - `Heap::collect_garbage_for_testing` on a detached heap keeps using its explicit argument.
- **Prefinalizers:**
  - They run in the atomic pause, before any `Drop` of the same cycle.
  - They may read dead objects that have not been swept yet.
  - They must not allocate. cppgc `CHECK`s this unless V8 was built with `CPPGC_ALLOW_ALLOCATIONS_IN_PREFINALIZERS`.
  - The registration is per object, made after allocation (see "Design").
- **`StandaloneTestingHeap::new` is `unsafe`.** It is only valid on a detached heap with detached GCs enabled.
  - `perform_marking_step`, `finalize_garbage_collection` and `toggle_main_thread_marking` panic if no GC is running, where the C++ code would dereference a null marker.
  - Debug V8 `DCHECK`s `start` and `finalize` inside a no-GC scope.
  - `force_compaction_for_next_garbage_collection` makes debug V8 `DCHECK` at the next GC, because the heap has no compactable spaces.
- **Statistics:**
  - `kBrief.used` is the allocation counter, which includes the current linear allocation buffer.
  - `kDetailed` first finishes sweeping and resets the allocation buffers, then walks the pages. Without a GC in between, the two can differ: item 6 at 1k nodes reported 257,880 B brief against 136,000 B detailed.
  - Type names are the `get_name()` strings, so per-type object counts and bytes are available.
- **No compactable spaces.** cppgc only moves objects that live in custom spaces declared compactable. rusty_v8 creates every CppHeap with `CppHeapCreateParams{{}}`, which means no custom spaces, and so does the isolate's own heap.
  - So cppgc never relocates rusty_v8 objects, and raw pointers stay valid until the object is swept.
  - This crate exposes no custom spaces. `assert_no_compactable_spaces()` checks the property at runtime: custom spaces show up as `CustomSpaceN` in detailed statistics.

## ABI matching

### How the shim is built (`build.rs`)

- **Compiler.** The shim is compiled with **Chromium's clang, the revision pinned by rusty_v8** (`tools/clang/scripts/update.py`).
  - Apple clang 17 cannot parse Chromium's libc++ headers; they need clang ≥ 21.
  - `build.rs` downloads the package (~47 MB, 262 MB unpacked) once into `~/.cache/v8-cppgc-ext/`. Override with `V8_CPPGC_EXT_CLANG_DIR` or `CLANG_BASE_PATH`.
- **Headers.** `-nostdinc++ -isystem buildtools/third_party/libc++ -isystem third_party/libc++/src/include -isystem third_party/libc++abi/src/include`, all from the v8-152.2.0 crate.
  - `__config_site` fixes `_LIBCPP_ABI_NAMESPACE=__Cr` and `_LIBCPP_ABI_VERSION=2`. Both are static-asserted.
  - `cpp_link_stdlib(None)`: libc++ comes from `librusty_v8.a`.
- **Flags:** `-std=c++20 -fno-exceptions -fno-rtti`, as in Chromium's `build/config/compiler`.
- **Defines.** They are derived from `v8/BUILD.gn` (`enabled_external_v8_defines`, `enabled_external_cppgc_defines`), `gni/v8.gni` and Chromium `build/config/c++`, using the gn args rusty_v8 passes (`build.rs: build_v8()`, `.gn`). For macOS arm64:
  - Release: `V8_ARRAY_BUFFER_INTERNAL_FIELD_COUNT=2 V8_ARRAY_BUFFER_VIEW_INTERNAL_FIELD_COUNT=2 V8_PROMISE_INTERNAL_FIELD_COUNT=1 V8_USE_DEFAULT_HASHER_SECRET=false V8_CPPGC_MICROTASK_QUEUE V8_TARGET_OS_MACOS V8_HAVE_TARGET_OS V8_TARGET_ARCH_ARM64 CPPGC_CAGED_HEAP CPPGC_YOUNG_GENERATION CPPGC_POINTER_COMPRESSION CPPGC_ENABLE_LARGER_CAGE CPPGC_SUPPORTS_OBJECT_NAMES CPPGC_ENABLE_OBJECT_SECTION_GCINFO CPPGC_SLIM_WRITE_BARRIER _LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_EXTENSIVE _LIBCPP_DISABLE_VISIBILITY_ANNOTATIONS _LIBCXXABI_DISABLE_VISIBILITY_ANNOTATIONS _LIBCPP_INSTRUMENTED_WITH_ASAN=0`
  - Debug prebuilt (`V8_FORCE_DEBUG=true`): the release set, plus `DEBUG CPPGC_ENABLE_API_CHECKS V8_ENABLE_MEMORY_ACCOUNTING_CHECKS`.
  - Not defined: `V8_COMPRESS_POINTERS`, `V8_ENABLE_SANDBOX` and `V8_ENABLE_CHECKS`. The default prebuilt has V8 pointer compression and the sandbox off, and rusty_v8 passes `v8_enable_v8_checks=false`.
  - The crate features `v8_enable_pointer_compression` and `v8_enable_sandbox` forward to rusty_v8 and switch the corresponding defines. They were untested when this README was written; M0 has since run the sandbox configuration, in fork mode, on macOS, Linux and Windows (`../M0_SUMMARY.md`).

### How it was verified (all checks pass on macOS arm64, release and debug prebuilts)

1. **Compile time: bindgen layouts.** `build.rs` reads rusty_v8's own bindgen output for the exact target, profile and features (`gen/src_binding_<features>_<profile>_<target>.rs`). That file was generated with the real GN defines. The shim `static_assert`s against it:
   - `sizeof` of `Member`, `WeakMember`, `TracedReference`, `Eternal`, `String::ValueView` and `ScriptOrigin`;
   - size and alignment of `RustObj`, `v8::HeapStatistics`, `v8::HeapSpaceStatistics`, `Object::Wrappable` and `cppgc::NameProvider`.
2. **Link time: define-gated symbols.** The shim's header-inline code references private slow paths that `librusty_v8.a` defines only under the matching define (checked with `nm -u` on the shim object and `nm -gU` on the prebuilt):
   - `WriteBarrier::CombinedWriteBarrierSlow<kCompressed>` needs `CPPGC_SLIM_WRITE_BARRIER` and `CPPGC_POINTER_COMPRESSION`;
   - `WriteBarrier::GenerationalBarrierSlow` needs `CPPGC_YOUNG_GENERATION`;
   - `CagedHeapBase::g_heap_base_` needs `CPPGC_CAGED_HEAP`;
   - `CageBaseGlobal::g_base_` needs `CPPGC_POINTER_COMPRESSION`.

   Claiming a feature the prebuilt lacks therefore fails to link.
3. **Runtime: V8's own configuration check** (`tests/abi.rs`, `v8_accepts_shim_build_configuration`). A child process calls the *shim-compiled* inline `v8::V8::Initialize()`. V8 compares the shim's pointer-compression, 31-bit-Smi, sandbox and `V8_ENABLE_CHECKS` bits with the library's, and FATALs on a mismatch. It accepts them.
4. **Runtime: Member compression round trip.** For 20,000 objects, interleaved with large-page objects:
   - the shim compresses and rusty_v8's prebuilt `cppgc__Member__Get` decompresses;
   - the prebuilt compresses and the shim decompresses;
   - the shim's barriered `operator=` writes and the prebuilt reads.
5. **Runtime: libc++ across the boundary.**
   - `CollectStatistics` builds `std::vector<SpaceStatistics{std::string,…}>` inside the prebuilt; the shim reads it and destroys it.
   - The tests check the exact space names (`NormalPageSpace0..3`, `LargePageSpace`), per-type object counts equal to the number allocated (777 and 50), the sum of per-space used bytes equal to the total, and sizes that are consistent page by page.
   - Item 6 measures exactly 136 B per `DomNode` object: 104 B payload, plus the 24 B `RustObjConcrete` head, plus an 8 B cppgc header.
6. **Runtime: `RustObjConcrete` mirror.** Prefinalizers have to find the Rust value inside rusty_v8's private `RustObjConcrete<T>`. The offset is re-checked on every registration, including for 16-byte-aligned `T`; a mismatch panics.

### Remaining ABI risks

- **Defines that no check observes.**
  - `CPPGC_SUPPORTS_OBJECT_NAMES` and `CPPGC_ENABLE_OBJECT_SECTION_GCINFO` only matter when the shim instantiates `GarbageCollected` types or GCInfo. It does not: it only uses `RustObj`, whose GCInfo lives in the prebuilt.
  - `_LIBCPP_HARDENING_MODE` does not change layouts.
  - `CPPGC_ENABLE_API_CHECKS` and `DEBUG` only add checks.
  - They are set from the GN logic, but a wrong value would not be detected.
- **Root `build_overrides` are not shipped in the crate.** `enable_safe_libcxx` (→ hardening mode) and similar settings come from V8's `build_overrides` and Chromium defaults. rusty_v8's actual root `build_overrides` is not in the published crate, so those values are inferred.
- **Feature unification.** If another crate enables `v8/v8_enable_pointer_compression` or `v8/v8_enable_sandbox` without enabling this crate's matching feature, the shim is built with the wrong V8 defines.
  - `cargo test --test abi` (check 3) catches this. A plain build does not.
  - The cppgc-side checks are unaffected, because cppgc compression is independent of V8 pointer compression.
- **Compiler drift.** The shim uses the same Chromium clang revision. A different clang ≥ 21 is very likely ABI-identical for this code, but has not been tried.
- **Upgrades.** Every rusty_v8 or V8 bump can change header-inline code, `RustObjConcrete` or the defines. That is why the dependency is pinned with `=`.

## Testing

Run single-threaded or not; each test binary serializes V8 use behind a lock.

```sh
cargo test --release                                            # release prebuilt
V8_FORCE_DEBUG=true cargo test --target-dir target-v8debug      # debug prebuilt (DCHECKs, CPPGC_VERIFY_HEAP)
cargo run --release --example spike_item1_nogc_scope            # ported spike programs
```

To reuse an already downloaded prebuilt, set `RUSTY_V8_ARCHIVE=/path/to/librusty_v8.a`.

**Results** (macOS 15.6, Apple M4, rustc 1.93.0). There are 28 integration tests plus 4 doc tests (1 compile check and 3 `compile_fail` checks for `!Send` and lifetimes). All pass against both the release and the debug prebuilt. In debug, 2 tests skip themselves because debug V8 aborts by design in those scenarios: forced compaction, and the no-barrier control.

| file | covers |
|---|---|
| `tests/abi.rs` | V8 build-config check (child process); shim config bits; Member round trips; statistics across libc++ |
| `tests/scopes.rs` | NoGC defers cppgc GCs (RAII, nested, static); Disallow `is_garbage_collection_allowed`; HeapState through an incremental GC, during marking under a NoGC scope, inside `Drop` (`is_sweeping_on_owning_thread`), conservative vs precise |
| `tests/statistics.rs` | kBrief and kDetailed consistency (detached heap and isolate heap); no compactable spaces |
| `tests/prefinalizer.rs` | spike item 4's prefinalizer part (detached incremental and unified precise); registration after allocation; 16-byte-aligned types; `is_in_atomic_pause` inside the prefinalizer; broker sees the partner alive in the first GC and dead in the last; weak callback clears a dead untraced pointer |
| `tests/testing_api.rs` | StandaloneTestingHeap start/step/toggle/finalize; panic instead of crash without a GC; forced compaction moves nothing; Override scope; `is_heap_object_old`; the 2-argument isolate GC (precise collects a stack-referenced object, conservative keeps it) |
| `tests/heap_consistency.rs` | spike item 5 scenario: a late raw store into a black object survives with `dijkstra_write_barrier` and is lost without it (release control); Steele re-trace; no-ops outside a GC |

### Ported spike programs vs the originals

The ports are `examples/spike_item{1,2,4,6}*` and their logs are in `logs/`. They use only this crate, with no `mod ffi`.

| | spike (mangled symbols) | port (v8-cppgc-ext) |
|---|---|---|
| item 1, `detached-quiescent` / `detached-marking` | 0 finalized in scope, 0 reader errors, 5,000 after the scope; PASS | same (3,646 / 4,078 refused GC requests); PASS |
| item 1, `unified-marking` / `unified-alloc-only` | **NoGC scope did NOT stop V8-initiated finalization: 5,000 finalized in scope**, readers fine | **same: 5,000 finalized in scope**, 0 live finalized, 0 errors |
| item 1, `unified-blocked-owner` (both scopes, owner blocked, marking active) | 10,244 phases, **0 finalizations**, 0 errors; debug V8 1,396 phases, 0 | 13,302 phases, **0 finalizations**, 0 errors; debug V8 1,518 phases, 0; PASS |
| item 1, `unified-disallow` | `Check failed: !IsGCForbidden()` | same (Signal 5) |
| item 1, `compaction-probe` | release: addresses stable; debug: DCHECK in `Compactor` | same |
| item 1, latency | 2.0 ns per enter+leave pair; p99 42 ns | 2.5 ns per pair; p99 42 ns |
| item 2, 30 s release / 20 s debug | 9,540 / 3,120 cycles, 0 finalized in worker phases, 0 errors | 10,364 / 3,268 cycles, **0** finalized in worker phases, 0 errors; PASS |
| item 4, detached + unified | 100 then 200 prefinalizers, none after a destructor, `Drop` on the main thread only; PASS | identical numbers; PASS (release and debug) |
| item 6, cppgc bytes/node (kBrief) | 128.9 / 141.8 / 136.7 B at 1k / 10k / 100k | **identical** 128.9 / 141.8 / 136.7 B; DP share 26.2% / 28.3% / 26.1% identical. New: kDetailed gives exactly 136.0 B per `DomNode` |

The TSAN runs and items 3 and 5 were not ported as programs: they needed no unexposed API, except the precise GC, which `tests/testing_api.rs` and `tests/heap_consistency.rs` cover.

### Not tested

- **Linux:** the define logic is written for it but has never been built.
- **Windows:** `build.rs` stops with an error. rusty_v8's Windows prebuilt uses clang-cl with the MSVC ABI, and the flag set for that has not been worked out.
- x86_64 macOS, iOS and Android.
- The `v8_enable_pointer_compression` and `v8_enable_sandbox` features.
- TSAN.
- A rusty_v8 built from source with custom `GN_ARGS` (a warning is printed).

## Upstream patch

`upstream/` contains:

- `rusty_v8-152.2.0-cppgc-ext.patch`: against `src/binding.cc`, `src/cppgc.rs` and `src/isolate.rs` of the v8 152.2.0 crate. It applies with `patch -p1`.
- `PR_DESCRIPTION.md`.
- `verify.sh` plus `verify/`: they recreate a patched copy of the crate in `upstream/work/v8`, symlinking its large directories, and run `verify/tests/upstream_api.rs` against the patched `v8::cppgc` API.

> **The `binding.cc` part of the patch has NOT been compiled inside rusty_v8's V8 build.** That needs a full V8 source build, which was out of scope on ~45 GB of free disk.

What was verified:

- The patch applies cleanly.
- The patched Rust compiles without warnings.
- The patched API passes its tests against the prebuilt.

The patch's C++ functions are supplied by this crate's shim. Its `BEGIN UPSTREAM` … `END UPSTREAM` block is pasted verbatim into the patch, so the same C++ text was compiled with the same headers and flags, but out-of-tree.

The patched API uses the same names as this crate. When upstream lands, rusty_v8's inherent methods (`Heap::heap_handle`, `Heap::collect_statistics`, `Isolate::request_garbage_collection_for_testing_with_stack_state`) shadow this crate's extension-trait methods. Migrating means switching imports from `v8_cppgc_ext::` to `v8::cppgc::`.

## Layout

```
build.rs                 defines, Chromium clang, bindgen-size expectations
src/binding.cc           C++ shim (ABI checks + BEGIN/END UPSTREAM block)
src/lib.rs               Rust API
tests/                   integration tests (see above)
examples/spike*/         ported cppgc-spike programs (items 1, 2, 4, 6)
logs/                    release and debug-V8 logs of the ported programs
upstream/                patch, PR description, verification workspace
```
