# cppgc: expose GC scopes, heap state, statistics, prefinalizers, weak callbacks, testing heap and write barriers

> **Status: NOT compiled inside rusty_v8's V8 build.** The `src/binding.cc` hunk
> was never built as part of `librusty_v8.a`, because that needs a full V8
> source build. What *was* checked, on macOS arm64 against the 152.2.0
> prebuilts:
> - The patch applies cleanly to the v8 152.2.0 crate sources (`patch -p1`).
> - The patched `src/cppgc.rs` and `src/isolate.rs` compile.
> - `upstream/verify` runs the patched Rust API against the prebuilt library.
>   The new C++ functions come from the out-of-tree `v8-cppgc-ext` shim, whose
>   `BEGIN UPSTREAM` block is the same text as the `binding.cc` hunk below.
>   It is compiled with the same headers, Chromium libc++ and defines.

## Summary

rusty_v8 binds `Heap::create`, `make_garbage_collected`, `Member`/`Persistent` and the 1-argument testing GC. The APIs below are missing from rusty_v8, although `librusty_v8.a` already exports every symbol they need. An embedder that wants them today has to link mangled Itanium names. This PR adds C ABI wrappers in `binding.cc` and safe Rust types in `cppgc.rs` and `isolate.rs`.

| C++ | Rust |
|---|---|
| `CppHeap::GetHeapHandle` | `cppgc::HeapHandle<'h>` (`Heap::heap_handle()`, `HeapHandle::from_isolate_unbounded`) |
| `subtle::NoGarbageCollectionScope` (RAII and static Enter/Leave) | `cppgc::NoGarbageCollectionScope` |
| `subtle::DisallowGarbageCollectionScope` (RAII, Enter/Leave, `IsGarbageCollectionAllowed`) | `cppgc::DisallowGarbageCollectionScope` |
| `subtle::HeapState::{IsMarking, IsSweeping, IsSweepingOnOwningThread, IsInAtomicPause, PreviousGCWasConservative}` | `cppgc::HeapState::*` |
| `CppHeap::CollectStatistics(kBrief / kDetailed)` | `Heap::collect_statistics` → Rust-owned `cppgc::HeapStatistics` |
| `CPPGC_USING_PRE_FINALIZER` + `LivenessBroker::IsHeapObjectAlive` | `cppgc::PreFinalizer`, `make_garbage_collected_with_prefinalizer`, `register_prefinalizer`, `LivenessBroker` |
| `Visitor::RegisterWeakCallback(Method)` | `Visitor::register_weak_callback_method` + `cppgc::WeakCallbackMethod` |
| `Isolate::RequestGarbageCollectionForTesting(type, EmbedderStackState)` | `Isolate::request_garbage_collection_for_testing_with_stack_state` |
| `testing::{OverrideEmbedderStackStateScope, StandaloneTestingHeap, IsHeapObjectOld}` | `cppgc::testing::*` |
| `subtle::HeapConsistency::{GetWriteBarrierType, DijkstraWriteBarrier, SteeleWriteBarrier, GenerationalBarrier}` | `cppgc::heap_consistency::*` |
| — | `Heap::assert_no_compactable_spaces()` |

## Design notes

- **Opaque pointers only.** Every new C function takes or returns
  `HeapHandle*`, `CppHeap*`, `RustObj*` or POD structs that are defined in
  `binding.cc`.
  - `HeapStatistics` is returned as an owned `cppgc::HeapStatistics*`. Rust
    copies it out through accessor functions and then deletes it, so no
    `std::vector` or `std::string` layout is mirrored in Rust.
  - `ObjectStatsEntry` is two `size_t`s; a `static_assert` guards this.
- **Threading and lifetimes.** `HeapHandle` borrows the `Heap`, and the handle
  and all guards are `!Send + !Sync`.
  - The isolate's CppHeap is reachable only through `&mut Isolate`. Unified-heap
    code has to hold a scope while it keeps running JS, so there is an `unsafe`
    constructor with an unbounded lifetime.
- **Scope semantics.** In `heap-consistency.cc`, the RAII constructor and
  destructor of each scope are exactly the static `Enter` and `Leave`, so the
  Rust guards call those.
  - The documentation states a behavior seen in testing: `NoGarbageCollectionScope`
    does **not** stop V8-initiated major GCs from finalizing an attached CppHeap.
  - `DisallowGarbageCollectionScope` turns such a GC into a `CHECK` failure.
- **Prefinalizers.** `CPPGC_USING_PRE_FINALIZER` is a data member whose
  constructor calls `PrefinalizerRegistration(this, Class::InvokePreFinalizer)`.
  `RustObj` is one C++ class shared by all Rust types, so registration happens
  per object after allocation, with a callback that is monomorphized for each
  Rust type.
  - The callback reaches `T` through `RustObjConcrete<T>`.
  - The callback does the liveness check itself, as `InvokePreFinalizer` does.
  - An alternative is to add `const HAS_PRE_FINALIZER: bool = false` and
    `fn pre_finalize` to `GarbageCollected`, and have `make_garbage_collected`
    register automatically. That changes an existing trait, so this PR keeps a
    separate trait.
- **Weak callbacks** pass the address of the traced `&T` as `data`. The callback
  receives `&LivenessBroker` (`const LivenessBroker&` has the same ABI as a pointer).
- **Write barriers.** `WriteBarrierParams` depends on the build
  configuration, so it never leaves C++. Each entry point computes the barrier
  type and dispatches in a single call, the same way `Member::operator=` does.
  `DijkstraWriteBarrierRange` is not exposed.
- **StandaloneTestingHeap** is only valid on detached heaps, so its constructor
  is `unsafe`. Marking-step, finalize and toggle calls panic if no GC is in
  progress, because the C++ code would dereference a null marker.
- **No custom spaces.** rusty_v8 creates every CppHeap with
  `CppHeapCreateParams{{}}`, so cppgc never compacts (moves) rusty_v8 objects.
  `assert_no_compactable_spaces()` lets code that relies on stable addresses
  check this at runtime. It uses detailed statistics: custom spaces are named
  `CustomSpaceN`.

## Testing

The out-of-tree crate `v8-cppgc-ext` has 28 integration tests, 4 doc tests
(including `compile_fail` checks for `!Send` and for lifetimes) and ports of
four programs from a cppgc evaluation spike. Its README has the full results.
They were run on macOS arm64 against both the release and the debug
(`V8_FORCE_DEBUG=true`) 152.2.0 prebuilts. `upstream/verify` exercises the
patched `v8::cppgc` API the same way.

Not tested: Linux and Windows.
