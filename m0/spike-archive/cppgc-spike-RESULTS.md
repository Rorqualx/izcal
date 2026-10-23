# cppgc spike: results

This spike answers HGBA-2.0 Q-1b(a) and HGBA-3.0 D8 item 6 with running code. The source is in `src/`, and raw run logs are in `logs/`. Everything below was observed on the run date, 14 October 2026. Anything that was not observed is marked as such.

## Environment

| | |
|---|---|
| `v8` crate | **152.2.0** (crates.io, latest at the time), using the prebuilt static library. V8 was not built from source. |
| V8 | **15.2.124.1-rusty** (`v8::V8::get_version()`) |
| Prebuilt libs | `librusty_v8_release_aarch64-apple-darwin.a` (default). `librusty_v8_debug_aarch64-apple-darwin.a` was fetched with `V8_FORCE_DEBUG=true` and is `is_debug=true`, so DCHECKs and `CPPGC_VERIFY_HEAP` are on. |
| cppgc config | Pointer compression is on (`Member` is 4 bytes) and `TracedReference` is 8 bytes. rusty_v8 creates the `CppHeap` with no custom spaces (`binding.cc`: `v8::CppHeapCreateParams params{{}}`). |
| Platform | macOS 15.6 (24G84), Apple M4 (10 cores, 24 GB), `arm64` |
| Toolchains | rustc 1.93.0 stable for all normal runs. rustc 1.101.0-nightly (822182db4 2026-10-13) for TSAN. Apple clang 17.0.0. |

## How rusty_v8 exposure was assessed

I read `src/cppgc.rs`, `src/binding.cc` and `src/isolate.rs` in `~/.cargo/registry/src/*/v8-152.2.0/` and compared them with the V8 headers shipped in the same crate (`v8/include/cppgc/*.h`). I then ran `nm -gU` on both prebuilt libraries.

**What rusty_v8 152.2.0 exposes for cppgc:**
- `Heap::create`, `make_garbage_collected`, `Member`, `WeakMember`, `Persistent` and `WeakPersistent`.
- `TracedReference`, `Visitor::trace`, `GcCell`.
- `Object::wrap` / `Object::unwrap` (CppHeapPointer wrapping) and `Isolate::get_cpp_heap`.
- `collect_garbage_for_testing` and `enable_detached_garbage_collections_for_testing`.

**What it does not expose:**

| C++ API | Header | In rusty_v8? | Exported by the prebuilt `.a`? |
|---|---|---|---|
| `cppgc::subtle::NoGarbageCollectionScope` (Enter/Leave/ctor/dtor) | `cppgc/heap-consistency.h` | no | **yes** |
| `cppgc::subtle::DisallowGarbageCollectionScope` | `cppgc/heap-consistency.h` | no | **yes** |
| `cppgc::subtle::HeapConsistency` (barrier helpers; inline, calling `WriteBarrier::*Slow`) | `cppgc/heap-consistency.h` | no | the slow paths, yes |
| `cppgc::subtle::HeapState::IsMarking/IsSweeping/...` | `cppgc/heap-state.h` | no | **yes** |
| `v8::CppHeap::GetHeapHandle`, `CppHeap::CollectStatistics` | `v8-cppgc.h` | no | **yes** |
| `cppgc::testing::StandaloneTestingHeap` (incremental GC control, forced compaction) | `cppgc/testing.h` | no | **yes** |
| Prefinalizers: `CPPGC_USING_PRE_FINALIZER` → `cppgc::internal::PrefinalizerRegistration`, `LivenessBroker::IsHeapObjectAliveImpl` | `cppgc/prefinalizer.h`, `cppgc/liveness-broker.h` | no | **yes** (internal API) |
| Custom / compactable spaces (`CppHeapCreateParams::custom_spaces`) | `v8-cppgc.h` | no (hard-coded empty) | n/a (compile-time C++ types) |
| `Isolate::RequestGarbageCollectionForTesting(type, EmbedderStackState)` | `v8-isolate.h` | only the 1-argument overload | **yes** |
| Heap verification modes (`CPPGC_VERIFY_HEAP`) | build flag (`BUILD.gn`: `cppgc_enable_verify_heap = v8_enable_verification_features \|\| v8_dcheck_always_on`) | no runtime switch exists in V8 | on in the debug prebuilt only |

Every symbol marked "yes" is already exported, so the spike links to the mangled Itanium names directly (`src/lib.rs`, `mod ffi`). No C++ shim was compiled and V8 was not patched. This route is **test-grade only**. Mangled names are not a stable API, and `CollectStatistics` returns a struct containing `std::__Cr::vector`, which the spike reads with a hand-written layout and only at `kBrief` detail. **For production, rusty_v8 has to expose these APIs** (an upstream PR or a fork). This is the concrete form of HGBA-2.0's "`rusty_v8` owner" start condition.

Because of this, items that depend on these APIs have two statuses: the **exposure status in rusty_v8** and the **behavior status** observed through the exported symbols.

---

## Item 1: No-GC and no-compaction held together across a worker phase

**Exposure: BLOCKED (not exposed) in rusty_v8. Behavior through exported symbols: PASS, but the scope's semantics are narrower than the spec assumed (see below).**

`src/bin/item1_nogc_scope.rs`. The parent process runs each scenario in its own child process under a 120 s watchdog. Scope entry and every in-scope GC request also run under a 5 s in-process watchdog (exit code 86 = deadlock). Neither watchdog ever fired.

Each scenario uses a 20k-node tree plus 5k garbage nodes and 8 reader threads traversing through raw pointers.

| Scenario | What happened |
|---|---|
| `detached-quiescent` (standalone CppHeap, idle) | 3,022 `CollectGarbageForTesting` requests inside the scope were all refused. 0 objects finalized during the scope. 133,897 reader traversals, 0 errors. After leaving the scope, one GC finalized all 5,000 garbage nodes and 0 live nodes. |
| `detached-marking` (scope entered **while incremental + concurrent marking was running**) | Entry returned immediately. `IsMarking` stayed true. Marking **kept progressing** inside the scope: 3,082 steps, and the marker reported "done". **Finalization was deferred**: 0 finalized, 3,082 GC requests refused. 142,260 traversals, 0 errors. After the scope: 5,000 garbage finalized, 0 live. |
| `unified-marking` (isolate's CppHeap, `--stress-incremental-marking`, marking active at entry; the owner thread keeps allocating JS and requests 5 full GCs) | **The scope did NOT block V8-initiated finalization: all 5,000 garbage cppgc objects were finalized inside the scope.** Live objects were untouched (0 live finalized, 126,083 traversals, 0 errors). |
| `unified-alloc-only` (as above, but only JS allocation and no explicit GC request) | Same result: V8's allocation-triggered GC finalized 5,000 cppgc objects inside the scope. Readers were fine. |
| `unified-blocked-owner` (**the pattern the spec relies on**: the owner holds `NoGarbageCollectionScope` + `DisallowGarbageCollectionScope` and blocks on the workers while V8 marking is active; JS and cppgc garbage are allocated between phases) | 10,244 worker phases in 10 s, all started while marking was active. 245,856 traversals. **0 finalizations during phases, 0 errors.** Debug V8: same result. 2,048,800 garbage objects were collected between phases. |
| `unified-disallow` (full GC requested inside `DisallowGarbageCollectionScope`) | `Check failed: !IsGCForbidden()` crash, in both release and debug. This is by design and is the fail-fast guard. |
| `compaction-probe` (force cppgc compaction for 3 GCs over a fragmented heap) | Release: object addresses unchanged and the tree intact. Debug V8: `DCHECK(!enable_for_next_gc_for_testing_)` in `Compactor::ShouldCompact`. That DCHECK fires because **the heap has no compactable spaces at all**. |

**Why the unified result happens (V8 source, `v8/src/heap/`):**
- `NoGarbageCollectionScope` only increments `HeapBase::no_gc_scope_`.
- `CppHeap::IsGCAllowed()` checks that counter, and only cppgc-initiated paths consult it: `CppHeap::CollectGarbage`, allocation reporting and `RetryAllocate` (`cppgc-js/cpp-heap.cc`).
- V8's own major GC (`Heap::CollectGarbage` → `MarkCompactCollector`) does not consult it, and finalizes the attached CppHeap regardless.
- Only `IsGCForbidden()`, which is `DisallowGarbageCollectionScope`, is checked on that path, and the check is a CHECK, so it crashes.

**Answers to the spec's questions:**
- **Entering the scope under active marking neither blocks, nor defers marking, nor asserts, nor deadlocks.** Marking continues. Only finalization initiated by cppgc is deferred.
- **No-compaction is structural, not scoped.** cppgc compacts only custom spaces that declare themselves compactable. rusty_v8 cannot create custom spaces, so cppgc never moves rusty_v8 objects. Addresses stayed stable through 3 forced-compaction GCs, and debug V8 asserts that forcing compaction on such a heap is meaningless. V8's own compaction moves JS objects only, and workers never touch those.
- **There is no single scope that blocks all finalization in the unified heap.** The working discipline has two parts:
  - The owning thread does nothing that can trigger a V8 GC while workers read. V8 GCs run only on the owning thread, so blocking that thread is enough.
  - The owner holds `DisallowGarbageCollectionScope` together with `NoGarbageCollectionScope`, so any violation crashes instead of silently finalizing.
- With that discipline, no worker read ever observed a finalized or moved object, under continuous marking.
- In the debug V8 build, calling the testing-only `StandaloneTestingHeap::StartGarbageCollection` or `FinalizeGarbageCollection` inside a no-GC scope DCHECKs (`cpp-heap.cc:1160`). The scenarios therefore only use legal in-scope calls.

**Entry latency** (timer tick is about 41.7 ns, so p50 values are below resolution):

| | n | p50 | p99 | p99.9 | max |
|---|---|---|---|---|---|
| detached, idle | 100,000 | 0 ns | 42 ns | 42 ns | 84 ns |
| detached, incremental + concurrent marking active | 20,000 | 0 ns | 42 ns | 84 ns | 333 ns |
| unified, not marking | 20,000 | 0 ns | 42 ns | 125 ns | 625 ns |
| unified, V8 incremental marking active | 582,450 | 0 ns | 42 ns | 42 ns | 65 µs |

- The amortized cost of an enter+leave pair is **2.0 ns** (10M pairs).
- The maximums are preemption outliers; entry is a counter increment and cannot wait on the GC.
- **P-1 needs no GC-stall budget for scope entry.**

**Reproduce:**
```
cargo run --release --bin item1_nogc_scope                    # all scenarios
cargo run --release --bin item1_nogc_scope -- child unified-blocked-owner   # one scenario
V8_FORCE_DEBUG=true cargo run --target-dir target-v8debug --bin item1_nogc_scope   # debug V8 (DCHECKs)
```
Logs: `logs/release_item1_nogc_scope.log`, `logs/debugv8_item1_nogc_scope.log`, `logs/tsan_item1_nogc_scope.log`.

---

## Item 2: Cross-thread reads without `CrossThreadPersistent`

**Exposure: BLOCKED (not exposed) in rusty_v8, because it needs item 1's scopes. Behavior: PASS. TSAN: zero findings in the Rust code it can see; it cannot see V8 (limits below).**

`src/bin/item2_crossthread_raw.rs`. The setup is a unified heap with `--stress-incremental-marking`. Each cycle runs a mutator phase and then a worker phase:
- **Mutator phase.** 20 barriered `Member` writes: insert 2-node chains, or unlink subtrees, which turns them into garbage. JS garbage is allocated so that V8 runs GCs.
- **Worker phase.** The owner computes the expected checksum, enters `NoGarbageCollectionScope` + `DisallowGarbageCollectionScope`, and gives 8 workers raw `*const TNode` pointers. No `Persistent` of any kind crosses threads. Workers check every node's magic word and payload (with `debug_assert!`s), compare the checksum, and the owner blocks until they finish.

| Run | Cycles (with marking active) | Inserts / removes | Worker traversals | Garbage finalized | Finalized during worker phases | Errors |
|---|---|---|---|---|---|---|
| release V8, 30 s | 9,540 (9,397) | 94,990 / 33,048 | 152,640 | 104,247 | **0** | **0** |
| debug V8 + Rust debug assertions, 20 s | 3,120 (3,073) | 30,969 / 10,660 | 49,920 | 36,650 | **0** | **0** |
| TSAN (release Rust, release V8), 60 s | 3,963 (3,913) | 39,422 / 13,530 | 63,408 | 45,237 | **0** | **0**, **0 TSAN warnings** |

**TSAN** (`-Zsanitizer=thread -Zbuild-std`, nightly):
- **It ran.** The binary links `librustc-nightly_rt.tsan.dylib`. V8's large virtual-memory reservations did not conflict with TSAN's shadow memory on macOS arm64.
- **It catches races on cppgc memory.** The control run (`--racy-control`) adds one unsynchronized Rust write by a worker to a field other workers read. TSAN reported it as `WARNING: ThreadSanitizer: data race` (`logs/tsan_item2_racy_control.log`). The racing address was inside the cppgc cage.
- **Clean runs: 0 warnings.** That covers item 2 (60 s) and all item 1 scenarios.

**What TSAN could not observe.** The prebuilt V8 library is not instrumented, so TSAN sees only Rust-side memory accesses, plus synchronization through intercepted libc and pthread calls. Invisible to it:
- cppgc's concurrent marking threads reading `Member` slots;
- write barriers and sweeper threads;
- V8's own threads;
- **workers' `Member` reads**, because `Member::get` is an FFI call into `cppgc__Member__Get`.

So the "zero findings" result covers the Rust payload reads, the Rust `Drop` writes and the hand-off pattern. It does not prove there are no races between workers and cppgc internals.

What does cover those internals is the design argument:
- workers only read;
- the owner is blocked;
- concurrent markers only read;
- cppgc never moves these objects.

Debug V8's DCHECKs did not fire, and stress produced 0 integrity errors.

A fully instrumented V8 would need a from-source TSAN build of V8 (`is_tsan=true`). That is out of scope here because prebuilt-only was required.

**Reproduce:**
```
cargo run --release --bin item2_crossthread_raw -- --secs 30
V8_FORCE_DEBUG=true cargo run --target-dir target-v8debug --bin item2_crossthread_raw -- --secs 20
rustup toolchain install nightly --component rust-src
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly build -Zbuild-std --target aarch64-apple-darwin \
  --target-dir target-tsan --release --bin item2_crossthread_raw --bin item1_nogc_scope
TSAN_OPTIONS=halt_on_error=0 ./target-tsan/aarch64-apple-darwin/release/item2_crossthread_raw --secs 60
TSAN_OPTIONS=halt_on_error=0 ./target-tsan/aarch64-apple-darwin/release/item2_crossthread_raw --secs 5 --racy-control
```

---

## Item 3: DOM-to-JS edges

**PASS, using only rusty_v8 APIs (`TracedReference`, `Object::wrap`/`unwrap`, the isolate's CppHeap).**

`src/bin/item3_dom_js_edges.rs`. Full GCs are run 3 times, precisely (stack state `kNoHeapPointers`).

| Test | Result |
|---|---|
| 3a: node rooted only from the DOM side (cppgc `Persistent`) holds a `TracedReference` to its wrapper; forced unified GC | Wrapper survives with its expando intact, `unwrap(wrapper) == node`. 0 nodes finalized. **PASS** |
| 3a′: then the DOM root is dropped too | Wrapper and node are both collected. |
| 3b: pair rooted only from JS (`globalThis.keep = wrapper`) | Node survives through the wrapper's CppHeap pointer, same address, 0 finalized. **PASS** |
| 3c: cycle (wrapper → node through `Object::wrap`, node → wrapper through `TracedReference`, plus a JS closure on the wrapper capturing it), all roots dropped | Wrapper collected (`v8::Weak` empty) and cppgc node finalized (counter). **Cycle collected: PASS** |
| 3d: negative control: plain Rust `Box` struct holding `v8::Global<Object>`, wrapper points back through an internal-field `External`, all JS roots dropped | Wrapper still alive and the Rust struct never dropped: **leaks as expected**. The wrapper was collected only after the Rust struct was freed by hand. |

**Finding about the test hooks.** rusty_v8's `request_garbage_collection_for_testing` is the 1-argument V8 overload, which **scans the stack conservatively for cppgc pointers**. With it, 3c fails (`ITEM3_CONSERVATIVE=1`) because a raw `*const DomNode` still in `main`'s frame keeps the node, and therefore the wrapper, alive. That is conservative-scanning behavior, not a cycle leak. The precise run uses the exported 2-argument overload with `kNoHeapPointers`.

**Reproduce:**
```
cargo run --release --bin item3_dom_js_edges
ITEM3_CONSERVATIVE=1 cargo run --release --bin item3_dom_js_edges
```
Debug V8: PASS (`logs/debugv8_item3_dom_js_edges.log`).

---

## Item 4: GC objects owning Rust data

**PASS, using only rusty_v8 APIs. Prefinalizers: not exposed in rusty_v8, but work through exported internal symbols.**

`src/bin/item4_rust_owned_data.rs`. Each `Elem` cppgc object holds `Box<ElementData>` and `Arc<SharedRule>` as plain fields, with no `Member` and an empty `trace`. 10,000 objects are allocated and half are rooted. A counting global allocator tracks the Rust heap.

| Heap / GC trigger | Drops | `Arc` strong count | Rust heap after all collected | Drop thread |
|---|---|---|---|---|
| detached, incremental marking + concurrent sweeping (`StandaloneTestingHeap`) | 5,000, then 10,000 ✔ | 5,001 → 1 ✔ | baseline + 40 KB (the `Vec<Persistent>` capacity) | main thread only |
| unified, **V8-scheduled GCs from JS allocation only** (5,000 drops after 39 ms), then precise full GC | 5,000, then 10,000 ✔ | 5,001 → 1 ✔ | baseline + 41 KB | main thread only |

- **Destructors run on the owning thread in both modes.** cppgc's concurrent sweeper did not run Rust `Drop` on a background thread, so `Drop` impls need no `Send` or `Sync` reasoning.
- **Prefinalizers** were registered on 200 objects through `PrefinalizerRegistration` and `LivenessBroker::IsHeapObjectAliveImpl`. Exactly 100 fired when the dead half was collected and 200 once all were collected. None fired after a destructor in the same cycle, which matches the "prefinalizers before sweeping" contract.
- This works today, but `cppgc::internal::PrefinalizerRegistration` is an internal API. rusty_v8 needs to wrap it.

**Reproduce:** `cargo run --release --bin item4_rust_owned_data`. Debug V8: PASS.

---

## Item 5: Heap-consistency checks in a debug build

**PASS with the debug prebuilt (the omitted barrier is caught). The release prebuilt misses it silently. rusty_v8 exposes no `HeapConsistency` API and no verification mode, and V8 has no runtime switch for one.**

`src/bin/item5_write_barrier.rs`. Detached CppHeap, GC driven step by step, precise stack.
- A is rooted. B is white, referenced only by a raw pointer the GC cannot see.
- Marking runs to completion, so A is black.
- B is stored into `A.first_child`, either through `Member::set` (C++ `operator=` with the Dijkstra barrier) or by copying the 4-byte compressed pointer into A's slot. The second path is the omitted barrier.
- The GC is finalized.

| Variant | Release V8 | Debug V8 (`V8_FORCE_DEBUG=true`) |
|---|---|---|
| with barrier | B survives | B survives |
| **barrier omitted** | **Missed**: B finalized while A still points at it, leaving a dangling `Member` (use-after-free); exit 0 | **Caught**: `Fatal error in v8/src/heap/cppgc-internal/marking-verifier.cc:42 MarkingVerifier: Encountered unmarked object. Hint: TNode → TNode` (SIGTRAP) |

The check comes from `CPPGC_VERIFY_HEAP`, a compile-time define that is on in the debug prebuilt and off in release. Practical consequence: S-4c's barrier checking happens in debug-V8 test runs, as CI configuration. It cannot be a runtime mode of release or Nightly builds. The test used the detached CppHeap only; I did not run a unified-heap variant, though it is the same `v8::internal::CppHeap` class and verifier.

**Reproduce:**
```
cargo run --release --bin item5_write_barrier
V8_FORCE_DEBUG=true cargo run --target-dir target-v8debug --bin item5_write_barrier
```

---

## Item 6: Data-plane memory and flush cost

**PASS (measurements complete). INC-3 headroom: INCONCLUSIVE, because renderer memory per node was not measured.** Against toy node memory alone, the data plane is far above 10%.

`src/bin/item6_data_plane/`:
- `dom.rs`: the cppgc DOM, which appends to the log on every mirrored mutation.
- `log.rs`: plain-data mutation log.
- `data_plane.rs`: generational-index struct-of-arrays plus the **log application**.
- `projection.rs`: a **fresh, independent projection** of the cppgc DOM.
- `oracle.rs`: the equality check.

`data_plane.rs` and `projection.rs` do not import each other and share no helpers.

**Toy DOM:** 40 tags, 2–6 attributes per element (about 4.1 on average, fresh 6–30 character strings), 0–4 classes, 10% with state flags, 5% with an inline declaration block. Links are parent / first / last / prev / next `Member`s, plus an empty `TracedReference` wrapper slot. `size_of::<DomNode>()` = 104 B.

**Data plane:**
- Generation, 5 link columns (`u32`), tag, state.
- `Box<[(Atom, Arc<str>)]>` attributes. The strings are shared with the DOM through `Arc`, not copied.
- `Box<[Atom]>` classes, an `Option<Arc<DeclBlock>>` reference, and a dirty byte as incremental metadata.

The cppgc heap is measured with `CppHeap::CollectStatistics(kBrief).used_size_bytes` (exported symbol). Rust heap bytes come from a counting allocator, both requested and actual `malloc_size`.

### Memory (bytes per node)

| DOM size | cppgc DOM heap | DOM Rust-side malloc (actual) | DOM total | Data plane (actual malloc, after `shrink_to_fit`) | Vec growth slack | **DP share of DOM + DP** | DP / DOM |
|---|---|---|---|---|---|---|---|
| 1k | 128.9 | 301.0 | 429.9 | 152.5 | ~0 | **26.2%** | 35% |
| 10k | 141.8 | 299.2 | 441.1 | 174.0 | 23.5 | **28.3%** | 39% |
| 100k | 136.7 | 299.3 | 436.0 | 153.6 | 20.0 | **26.1%** | 35% |

- **Breakdown per node:** generation 4, links 20, tag 4, state 4, attrs pointer 16 + **attrs heap ≈83**, classes pointer 16 + classes heap ≈8, style reference 8, dirty 1.
- **Attributes are about 60% of the data plane.** That is the first target for INC-3's coarsening path: interned attribute arenas, or mirroring only the attributes selectors actually read.
- **Not counted on either side:** the script-side handle table (8 B/node). It exists only because rusty_v8 cannot turn `&T` back into a cppgc pointer.
- **Shared on the DOM side:** attribute strings and declaration blocks are counted once, on the DOM side. A data plane that copied strings would cost more.

### Time

| | 1k | 10k | 100k |
|---|---|---|---|
| Log append (steady state, includes `Arc` clone), `size_of::<Mutation>()` = 32 B | 4.7 ns | 4.5 ns | 4.4 ns |
| Initial flush (whole build log) | 8,160 mutations, 0.28 ms (35 ns each) | 81,546, 2.97 ms (36 ns) | 815,394, 29.2 ms (36 ns) |
| **Flush N=1** (mean / p50 / p99, n=5000) | 42 / 42 / 125 ns | 57 / 42 / 291 ns | 122 / 125 / 375 ns |
| **Flush N=100** (n=1000) | 4.9 / 4.4 / 16.4 µs | 5.8 / 5.5 / 10.0 µs | 14.8 / 14.0 / 35.4 µs |
| **Flush N=10k** (n=20) | 0.53 / 0.52 / 0.63 ms | 0.76 / 0.75 / 0.93 ms | 1.38 / 1.39 / 1.57 ms |
| **Fresh projection + equality** on the full built tree | 0.06 + 0.04 = **0.10 ms** | 0.87 + 0.59 = **1.46 ms** | 13.7 + 8.1 = **21.8 ms** |
| Projection + equality after the flush runs (tree grew to) | 1.31 ms (8,657 nodes) | 3.78 ms (18,969) | 39.8 ms (109,262) |

**Notes:**
- **Mutation mix:** 30% set attribute, 5% remove attribute, 25% toggle class, 15% state, 5% inline style, 6% insert, 6% remove leaf, the rest subtree moves.
- **The DOM grew during the flush runs**, because inserts outnumbered successful removals. Removed nodes keep their slots: the spike logs no GC-driven destroy, so slot reuse was not exercised.
- **The oracle held** after every measured batch at all sizes.
- **Negative control:** the oracle **detected** a single mutation applied to the DOM without being logged ("state flags differs").
- **Debug cost:** at 100k nodes, a fresh projection plus equality check costs about 22 ms, more than one 60 Hz frame. That supports the spec's choice of "every flush in debug, sampled in Nightly" rather than always on.

**Reproduce:** `cargo run --release --bin item6_data_plane` (log: `logs/release_item6_data_plane.log`).

---

## Mapping to the specs' decisions

| Decision | Rule | Outcome |
|---|---|---|
| **Q-1b(a)** (HGBA-2.0 Stage 1 start condition) | pass only if items 3–5 pass | **PASS on behavior.** Item 3 (TracedReference, unified-GC wrapper survival and cycle collection) and item 4 (Rust-owned data, `Drop` on collection) pass with **rusty_v8's public API**. Item 5 passes **only with the debug prebuilt** (`V8_FORCE_DEBUG=true`) and needs no API. But taken literally, Q-1b(a) asks whether *rusty_v8 exposes enough of cppgc*, and **it does not.** No-GC and disallow-GC scopes, `HeapState`, heap statistics, prefinalizers, testing-heap control and the precise testing GC are reachable only through mangled exported symbols. Stage 1 needs them added to rusty_v8 (upstream or a maintained fork) before S-4c code is written against them. That ties directly to the "`rusty_v8` owner" start condition. |
| **D8 fallback availability** (HGBA-3.0) | available only if items 1–2 pass | **Available, with a narrower guard than HGBA-2.0 assumed.** Workers can read `cppgc` objects through raw pointers without `CrossThreadPersistent`: 0 errors and 0 in-phase finalizations across about 27k stressed worker phases (item 1 blocked-owner: 10,244; item 2 release, debug and TSAN runs: 16,623), almost all under active marking, and TSAN clean on the Rust side. The guard, however, is not "a scope that blocks GC". It has three parts: (1) the owner thread is blocked for the whole worker phase; (2) it holds `NoGarbageCollectionScope` + `DisallowGarbageCollectionScope`, which makes a violating V8 GC crash instead of finalize; (3) there are no compactable custom spaces. `NoGarbageCollectionScope` alone does **not** stop V8-initiated unified GCs from finalizing cppgc objects. The fallback also depends on the same rusty_v8 exposure work as Q-1b(a), and on a from-source TSAN V8 build if S-4c's "TSAN with zero findings" must cover cppgc internals. |
| **INC-3 headroom** (≤10% of renderer memory for data plane plus incremental metadata) | from item 6 | **INCONCLUSIVE.** The data plane costs about **150–175 B/node** (plus about 20 B/node of growth slack), which is **26–28% of DOM + data-plane node memory** in this toy. INC-3 is measured against total renderer memory, which this spike does not measure. Arithmetically, the bound holds only if the renderer uses at least about **1.5–1.75 KB per DOM node** (DP bytes ÷ 0.10). That has to be checked on the corpus. If it fails, attribute storage (about 60% of the data plane) is the first thing to coarsen. Per-mutation cost is small (4.5 ns append, about 40–140 ns per mutation at flush), so flush time is not the risk. Memory is. |
| **Fallback shape** | | **F1 remains the default** (HGBA-3.0 D8; nothing here argues against it, and flush cost is low). **D8's fallback is the HGBA-2.0 primary path** (workers reading cppgc under the guard above). That fallback is available, so **F2 (single-threaded style and layout) is not forced.** F2 would apply only if the rusty_v8 exposure work cannot be done. |

## What was not tested or remains open

- **Platform coverage.** Only macOS arm64 was tested. Windows (no debug prebuilt link, per rusty_v8's build.rs) and Linux were not tried.
- **TSAN coverage.** TSAN covers Rust code only; V8 and cppgc internals are uninstrumented (see item 2).
- **Item 5 in the unified heap.** The write-barrier test ran on the detached CppHeap only.
- **Item 6 realism and scope.**
  - Content distributions are synthetic.
  - There is no GC-driven node destroy and no slot reuse in the benchmarks.
  - The data plane shares attribute strings with the DOM through `Arc`.
  - Renderer memory per node, and therefore INC-3 itself, is not measured.
- **Q-1b(b)** (GC pause times with a real DOM) is outside this spike, as the spec says.
- **ABI fragility.** The mangled-symbol bindings in `src/lib.rs` are valid for this exact prebuilt (V8 15.2.124.1). `CollectStatistics` relies on a hand-copied struct layout.
