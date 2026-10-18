# Stage 1 design facts (verified 2026-10-14)

Method: primary sources only. These include local crate source (`~/.cargo/registry/src/index.crates.io-*/v8-152.2.0/`), GitHub source and releases (via `gh api`), the crates.io API and docs.rs. Confidence: H = read in source or release metadata; M = strong secondary evidence or inferred from source; L = estimate.

---

## Q1. rusty_v8 prebuilt archives for pointer compression / sandbox

**Answer.** Pointer-compression prebuilts exist for only three targets. Sandbox prebuilts do not exist for any target.

- The crate is `v8` 152.2.0, released 2026-08-20 as the "Latest" GitHub release. It bundles **V8 15.2.124.1**, per `v8/include/v8-version.h`. The 152.x line was cut 2026-08-01. No 153.x had been released as of 2026-10-14.
- Cargo features, from `Cargo.toml`:
  - `default = ["use_custom_libcxx"]`
  - `v8_enable_pointer_compression`
  - `v8_enable_sandbox = ["v8_enable_pointer_compression"]`
  - `v8_enable_v8_checks`
  - `simdutf`
- **How `build.rs` names the archive:**
  - `prebuilt_features_suffix()` adds `_ptrcomp` when pointer compression is enabled, then `_sandbox` when the sandbox is enabled. With the sandbox feature on, the suffix is therefore `_ptrcomp_sandbox`.
  - The file name is `static_lib_name(format!("{features}_{profile}_{target}"))`. That gives `librusty_v8{features}_{release|debug}_{target}.a.gz`, or `rusty_v8…lib.gz` on Windows.
  - The download base is `https://github.com/denoland/rusty_v8/releases/download/v{version}/`.
  - It can be overridden with `RUSTY_V8_MIRROR` (supports `{tag}`, `{version}`, `{target}`, `{profile}`, `{features}` and `{file}` templates), `RUSTY_V8_MIRROR_TAG`, `RUSTY_V8_MIRROR_FALLBACK`, `RUSTY_V8_ARCHIVE` and `RUSTY_V8_ARCHIVE_SHA256`.
- **Release assets for v152.2.0.** The same set appears in v152.1.0, v152.0.0, v150.4.0 and v149.4.0.

  | Target | default (release) | `_ptrcomp` (release+debug) | `_ptrcomp_sandbox` |
  |---|---|---|---|
  | aarch64-apple-darwin (macOS arm64) | yes | **yes** | **no** |
  | x86_64-apple-darwin | yes | yes | no |
  | x86_64-unknown-linux-gnu | yes | **yes** | **no** |
  | aarch64-unknown-linux-gnu | yes | no | no |
  | x86_64-pc-windows-msvc | yes (release only) | **no** | **no** |
  | aarch64-pc-windows-msvc, *-linux-musl, riscv64, iOS | yes | no | no |

  None of these five releases has any asset with "sandbox" in its name.
- The README's "Experimental Features" section says the sandbox has no CI and no prebuilt archives. It requires `V8_FROM_SOURCE=1` or a custom-built archive. It also notes that the sandbox reserves about 1 TB of virtual address space and brings ArrayBuffer API limitations.
- **The sandbox source build is broken from the crates.io package.** Issue [#2070](https://github.com/denoland/rusty_v8/issues/2070) (opened 2026-08-21, still open) reports that `V8_FROM_SOURCE=1` with `v8_enable_sandbox` fails because the published crate lacks:
  - `third_party/disarm/src/parse.py`
  - `third_party/icu/common/icudtl.dat`
  - `third_party/rust/chromium_crates_io/vendor/...`

  The workaround is a full git checkout of `denoland/rusty_v8` at the release tag with submodules, plus `disarm` at the commit pinned in V8's DEPS. Built that way, it compiles and runs. Related issues: #1999 (ICU data) and #2035 (missing `v8/DEPS` on Windows).
- **What a `V8_FROM_SOURCE` build needs:**
  - From the README: Python 3, curl, and libclang 21.1+ for bindgen (`LIBCLANG_PATH`). On Linux it also needs glib-2.0 dev files.
  - `build.rs` downloads gn/ninja (`download_ninja_gn_binaries`), Chromium's clang (`clang_download`) and a Rust toolchain.
  - The published crate unpacks to 202 MB of source (v8/ 78 MB, third_party/ 114 MB).
  - **Estimate (L):** about 5–10 GB of disk including clang and the `out/` directory, and roughly 20–60 min for a release build on a 10-core laptop. Nothing in the README or issues states disk or time figures.
- **Architecture implication.** For a cppgc DOM with the V8 sandbox, plan on our own CI building the archive from a git checkout and serving it through `RUSTY_V8_MIRROR`/`RUSTY_V8_ARCHIVE`. This is needed on all three target OSes; Windows has no ptrcomp prebuilt at all. Pointer compression alone works off the shelf on macOS arm64 and Linux x64.
- **Sources:**
  - build.rs: https://github.com/denoland/rusty_v8/blob/v152.2.0/build.rs (`prebuilt_features_suffix`, `static_lib_name`, `artifact_url_candidates`)
  - Release: https://github.com/denoland/rusty_v8/releases/tag/v152.2.0
  - README: https://github.com/denoland/rusty_v8/blob/v152.2.0/README.md#experimental-features
- **Confidence:** H for assets and naming. L for the disk/time estimate.

---

## Q2. Platform customization and determinism in rusty_v8 / V8 15.2

### Platform constructors in `src/platform.rs` (v8 152.2.0)

- `new_default_platform(thread_pool_size, idle_task_support)`
- `new_unprotected_default_platform(..)`, which turns off PKU thread-isolated allocations
- `new_single_threaded_default_platform(idle_task_support)`. Its doc says it "must be used with the --single-threaded V8 flag".
- **`new_custom_platform(thread_pool_size, idle_task_support, unprotected, platform_impl: impl PlatformImpl)`**
  - `pub trait PlatformImpl: Send + Sync` has these methods: `post_task`, `post_non_nestable_task`, `post_delayed_task(.., delay: f64)`, `post_non_nestable_delayed_task`, `post_idle_task`.
  - Each method receives an owned `v8::Task` or `IdleTask`. The embedder calls `Task::run()` on the isolate thread.
  - The default method implementations run the task synchronously.
- `Platform::pump_message_loop`, plus `run_idle_tasks`, are also available.

### Limits of the custom platform

- In C++ (`binding.cc`), `class CustomPlatform : public v8::platform::DefaultPlatform` overrides only:
  - `GetForegroundTaskRunner`, wrapping it in a `CustomTaskRunner` that forwards to Rust
  - `GetThreadIsolatedAllocator`
- **Worker/background threads stay with DefaultPlatform.** That includes `CallOnWorkerThread` and `PostJob`.
- **Clocks are not overridable from Rust.** `MonotonicallyIncreasingTime` and `CurrentClockTimeMillis` are not overridden. `DefaultPlatform::CurrentClockTimeMillis()` returns `base::OS::TimeCurrentMillis()`.
  - `DefaultPlatform::SetTimeFunctionForTesting` exists in C++ and affects only monotonic time. It is not bound in rusty_v8.
- **Consequence:** fully deterministic task scheduling plus clocks requires one of:
  - (a) patching rusty_v8's `binding.cc` to add clock and worker-thread overrides. This is a small fork, about 50 lines. d8's `PredictablePlatform` (`v8/src/d8/d8-platforms.cc`) is the model: synthetic `MonotonicallyIncreasingTime` = counter × 1e-5, wall clock derived from it, worker tasks run on the foreground.
  - (b) `--single-threaded` together with `new_single_threaded_default_platform` or `new_custom_platform`, with Date intercepted at the JS level.

### Flags confirmed in V8 15.2.124 `src/flags/flag-definitions.h`

| Flag | Line | Effect / implications |
|---|---|---|
| `--predictable` | 4223 | Implies `single_threaded_gc`. Negates `concurrent_recompilation`, `memory_reducer`, `concurrent_sparkplug`, `lazy_compile_dispatcher`, `parallel_compile_tasks_for_{eager_toplevel,lazy}`, `stress_concurrent_inlining` and `maglev_*_on_background`. If `random_seed == 0` it sets `random_seed=12347`. |
| `--verify-predictable` | 4484 | Implies `predictable`. Also changes heap time behaviour (heap.cc). |
| `--single-threaded` | 4254 | "disable the use of background tasks". Implies `single_threaded_gc` and `wasm_sync_tier_up`, sets `wasm_num_compilation_tasks=0`, and negates concurrent recompilation, sparkplug, builtin generation and compile tasks. |
| `--single-threaded-gc` | 4271 | Negates concurrent/parallel marking, sweeping, compaction and scavenge, `concurrent_array_buffer_sweeping`, and `cppheap_concurrent_marking`. |
| `--random-seed` (int) | 3409 | Seeds `Math.random` per native context (`MathRandom::RefillCache` uses `v8_flags.random_seed` if non-zero) and the isolate RNG. |
| `--hash-seed` (uint64) | 3406 | "Fixed seed to use to hash property keys … with snapshots this option cannot override the baked-in seed". Needs `--rehash-snapshot` to take effect with a snapshot. |
| `--concurrent-recompilation` (use `--no-…`) | 1388 | |
| `--predictable-gc-schedule` | 4240 | Fixes semi-space sizes and `heap_growing_percent=30`; turns off memory reducer. |

### Injecting time and randomness

- **`Date.now()` / `new Date()`:** `JSDate::CurrentTimeValue` calls `V8::GetCurrentPlatform()->CurrentClockTimeMilliseconds()` (`src/objects/js-objects.cc:5880`). Determinism therefore needs either a platform clock override (C++ patch) or replacing `Date`/`Date.now` in the global before user scripts run.
- **`Math.random`:** `--random-seed=N`. There is **no** `Isolate::SetRandomNumberGenerator` in V8 15.2 headers. The only RNG hook is process-wide: `v8::V8::SetEntropySource`, exposed in Rust as `v8::V8::set_entropy_source(fn(&mut [u8]) -> bool)`.
- **`performance.now`:** not a V8 builtin. The embedder implements it, so it can use the simulated clock directly.
- **Time zone:** `v8::icu::set_default_time_zone("UTC")`, then `isolate.date_time_configuration_change_notification(TimeZoneDetection::Skip)`. These are new in recent rusty_v8 (see `src/icu.rs`). Locale: `icu::set_default_locale`.
- **Deno's approach:**
  - `deno run --seed N` pushes `--random-seed={seed}` into the V8 flags (`libs/cli_parser/src/convert.rs`) and also seeds its own RNG for `crypto.getRandomValues`.
  - Snapshots are built with `"--predictable --random-seed=42"` (`libs/core/runtime/setup.rs:179`).
  - Deno does not virtualize the clock.

### cppgc

- rusty_v8 exposes `v8::cppgc::{Heap, HeapCreateParams, Visitor, Traced, Member, WeakMember, Persistent, GcCell, GarbageCollected}` in `src/cppgc.rs`.
- `cppgc::Heap::enable_detached_garbage_collections_for_testing` exists.

**Sources:**
- https://github.com/denoland/rusty_v8/blob/v152.2.0/src/platform.rs
- https://github.com/denoland/rusty_v8/blob/v152.2.0/src/binding.cc (`class CustomPlatform`)
- https://chromium.googlesource.com/v8/v8/+/refs/heads/15.2.124/src/flags/flag-definitions.h, also read locally in the vendored copy
- https://github.com/denoland/deno/blob/main/libs/core/runtime/setup.rs

**Confidence:** H.

---

## Q3. Servo layout's DOM interface (2026)

- **Crate:** package `servo-layout-api` (lib name `layout_api`), located at `components/shared/layout/`. The old name was `script_layout_interface`.
- **The old trait set was removed.** PR #44114 ("script: Rewrite the layout DOM wrappers", merged 2026-04-12) deleted `wrapper_traits.rs` and the `ThreadSafeLayoutNode` / `ThreadSafeLayoutElement` pair. PR #44454 (2026-04-25) added a type-bundle trait to make parameterizing layout easier.
- **Current traits:**
  - `layout_node.rs`:
    - `LayoutNode<'dom>` requires `Copy + Debug + NodeInfo + Send + Sync` and has `type ConcreteTypeBundle`.
    - `DangerousStyleNode<'dom>` requires `TNode + NodeInfo`.
  - `layout_element.rs`:
    - `LayoutElement<'dom>`.
    - `DangerousStyleElement<'dom>` requires `TElement + selectors::Element<Impl=SelectorImpl>`.
  - `layout_dom.rs`:
    - `LayoutDomTypeBundle<'dom>`, with associated types `ConcreteLayoutNode`, `ConcreteLayoutElement`, `ConcreteDangerousStyleNode` and `ConcreteDangerousStyleElement`.
    - Aliases `LayoutNodeOf` etc.
  - The crate also exports `LayoutDataTrait`, `PseudoElementChain` and `LayoutDamage`.
- **`LayoutDom<'dom, T>` belongs to the `script` crate**, not to `layout_api`. The implementations live in `components/script/layout_dom/`: `ServoLayoutDomTypeBundle`, `ServoLayoutNode`, `ServoLayoutElement`, `ServoDangerousStyle{Node,Element,Document,ShadowRoot}`.
- **Can Servo layout run over a non-Servo DOM? Not without a fork.**
  - `components/layout/Cargo.toml` depends on `script` directly.
  - The code uses `script::layout_dom::ServoLayoutNode` concretely in `layout_impl.rs`, `dom.rs`, `layout_root.rs`, `context.rs`, `query.rs`, `flow/root.rs`, flexbox, table, taffy and accessibility. `LayoutThread` is not generic.
  - `layout_api` also pulls in `script_traits`, `paint_api`, `net_traits`, `fonts`, `servo-base`, `webrender_api` and others.
  - The trait seam exists and the stated goal is generic layout, but it is not finished. Using it means forking Servo, making layout generic over `LayoutDomTypeBundle`, and implementing all four traits plus the Stylo traits.
- **Projects doing this:** none found. Blitz is the practical route; it uses Stylo + Taffy + Parley, not Servo layout.
- **Sources:**
  - https://github.com/servo/servo/tree/main/components/shared/layout
  - https://github.com/servo/servo/pull/44114
  - https://github.com/servo/servo/pull/44454
  - https://github.com/servo/servo/blob/main/components/layout/Cargo.toml
  - https://doc.servo.org/layout_api/trait.LayoutDomTypeBundle.html
- **Confidence:** H for traits, paths and the dependency. M for "no projects" (Zulip not searched).

## Q4. Servo WebIDL codegen and Rust WebIDL parsers

- **Codegen location:** `components/script_bindings/codegen/`. Files: `codegen.py` (~398 KB), `configuration.py`, `run.py`, `Bindings.conf`. They were renamed from `CodegenRust.py` / `Configuration.py` in PR #38047 (2025-07). The codegen is written in Python.
- **Parser:** Mozilla's `WebIDL.py`, at `components/script_bindings/third_party/WebIDL/parser/WebIDL.py`.
  - Pinned upstream commit `cb16366db72f…`, with 9 local patches managed by `manage.py`, plus tests.
  - Depends only on PLY 3.10, vendored at `third_party/ply/` (BSD).
  - License MPL-2.0 (file-level copyleft).
- **Can the parser be reused for a V8 backend? Yes.**
  - It is standalone, produces a typed, validated AST, and has nothing SpiderMonkey-specific.
  - Only `codegen.py` is mozjs-specific; it would be replaced with a V8/rusty_v8 emitter.
  - Mozilla's and Chromium's IDL toolchains use the same family of parsers. Chromium's is `third_party/blink/renderer/bindings/scripts/web_idl`, using `blink_idl_parser`.
- **Rust WebIDL parsers:**

  | Crate | Latest | Published | Status |
  |---|---|---|---|
  | `weedle` | 0.13.1 | 2026-02-26 | Maintained in a minimal way. Repo moved to https://github.com/wasm-bindgen/weedle. Used by wasm-bindgen-webidl and webidl2wit. MIT. |
  | `weedle2` | 5.0.0 | 2024-01 | Dormant uniffi fork. |
  | `webidl` | 0.9.0 | 2019 | Dead. |

- **Confidence:** H.

## Q5. Stylo on crates.io, DOM traits, Blitz

- **Crates on crates.io**, all published from https://github.com/servo/stylo (tag v0.22.0):

  | Crate | Version | Date | License |
  |---|---|---|---|
  | `stylo` | 0.22.0 | 2026-09-30 | MPL-2.0 |
  | `selectors` | 0.41.0 | 2026-09-30 | MPL-2.0 |
  | `stylo_traits` | 0.22.0 | 2026-09-30 | MPL-2.0 |
  | `stylo_dom` | 0.22.0 | 2026-09-30 | MPL-2.0 |
  | `stylo_atoms` | 0.22.0 | 2026-09-30 | MPL-2.0 |
  | `stylo_static_prefs` | 0.22.0 | 2026-09-30 | — |
  | `stylo_malloc_size_of` | 0.22.0 | 2026-09-30 | MIT/Apache-2.0 |
  | `servo_arc` | 0.5.0 | 2026-09-30 | MIT/Apache-2.0 |

  - `stylo_config` (0.12.0, 2026-02) is superseded by `stylo_static_prefs`.
  - Servo itself uses a git revision of Stylo, not the crates.io releases.
- **Trait locations:** `style/dom.rs` in servo/stylo.
  - `NodeInfo` (line 42), `TDocument` (119), `TNode` (153), `TShadowRoot` (364), `TElement` (409).
  - `TElement` requires `selectors::Element<Impl=SelectorImpl> + ElementContext`.
  - `ElementContext` was formerly `AttributeProvider` (renamed 2026-06). It has a blanket impl for every `TElement`.
  - `selectors::Element` is in `selectors/tree.rs`.
- **Blitz** (`blitz-dom` 0.3.0-beta.3, 2026-10-12):
  - In `packages/blitz-dom/src/stylo.rs`, `type BlitzNode<'a> = &'a Node` implements `TDocument`, `NodeInfo`, `TShadowRoot`, `TNode`, `selectors::Element` and `TElement`.
  - `shadow_root()` returns `None`, so Blitz has no real shadow DOM.
  - It also has `impl DomTraversal<BlitzNode> for RecalcStyle` (drives the style traversal) and `RegisteredSpeculativePainters`.
  - Dependencies: `stylo` 0.22.0 (renamed to `style`), `selectors` 0.41.0, `cssparser` 0.38.0, `markup5ever` 0.40.0.
- **Sources:**
  - https://github.com/servo/stylo/blob/main/style/dom.rs
  - https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/src/stylo.rs
  - https://crates.io/crates/stylo
- **Confidence:** H.

## Q6. html5ever

- **Versions:** `html5ever` 0.40.1 (2026-09-14) and `markup5ever` 0.40.0 (2026-09-11). MIT OR Apache-2.0.
- **`TreeSink` takes `&self`.** Every method does, for example `fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>)`. The one exception is `fn finish(self) -> Self::Output`. Implementations therefore need interior mutability.
- **Associated types:** `type Handle: Clone; type Output; type ElemName<'a>: ElemName where Self: 'a;`
- **When it changed:** `&self` landed in **html5ever 0.28.0 / markup5ever 0.13.0 (2024-08-08)**, not 0.30. The `ElemName<'a>` associated type arrived in 0.29.0 (2024-09).
- **Source:** https://github.com/servo/html5ever/blob/main/markup5ever/interface/tree_builder.rs (spot-checked: `pub trait TreeSink` at line 133, `fn append(&self, …)` at line 196).
- **Confidence:** H.

## Q7. Taffy

- **Versions:** `taffy` 0.15.0 (2026-10-12), MIT. The previous release was 0.14.0 (2026-08-24).
- **Traits** (`src/tree/traits.rs`):
  - `TraversePartialTree` (`ChildIter`, `child_ids`, `child_count`, `get_child_id`)
  - `TraverseTree`
  - `LayoutPartialTree` (`get_core_container_style`, `set_unrounded_layout`, `compute_child_layout`, `resolve_calc_value`)
  - **New:** `LayoutContainingBlock`, which hoists absolutely positioned / out-of-flow boxes to their containing block (`add_hoisted_children`, `clear_hoisted_children`)
  - `CacheTree` (`cache_get`, `cache_store`, `cache_clear`)
  - `RoundTree`
  - `PrintTree`
  - `LayoutFlexboxContainer`
  - `LayoutGridContainer`
  - `LayoutBlockContainer`
- **Float and block support:**
  - Floats arrived in 0.10.0 (2026-03-31), with `FloatContext` and `BlockContext` behind the `float_layout` feature. That feature is now on by default. Releases 0.13 to 0.15 contain many float and block fixes.
  - Block layout is supported.
  - **Inline layout is not part of Taffy**; the embedder does it, e.g. with Parley. 0.15 adds `compute_block_align_content_offset` for embedders that lay out inline content outside Taffy.
- **Who uses it how:**
  - Servo uses `taffy` 0.14 with only the `grid` and `calc` features, i.e. for flex/grid only. Servo has its own block/inline layout.
  - Blitz pins Taffy to a git revision.
- **Source:** https://github.com/DioxusLabs/taffy/blob/main/src/tree/traits.rs
- **Confidence:** H.

## Q8. WebRender, Vello, Parley

- **WebRender can be used outside Servo/Gecko.** Servo's fork (https://github.com/servo/webrender, active) is published under the original crate names `webrender` and `webrender_api`:
  - Versions 0.68.0 (2026-01), 0.69.0 (2026-06) and **0.70.0 (2026-07-10)**. License MPL-2.0.
  - Before 0.68 there had been no release since 0.61 (2020).
  - It renders through OpenGL (via `gleam`).
- **No wgpu backend for WebRender exists**, upstream or in the Servo fork (confidence M, since this is absence of evidence). Servo's `wgpu-core` 30 dependency is for WebGPU. Servo's 2D canvas uses `vello_cpu`, with optional `vello`.
- **Alternatives (Linebender, Apache-2.0/MIT):**
  - `vello` 0.11.0 (2026-10-02)
  - `vello_cpu` 0.3.0
  - `vello_hybrid` 0.2.0
  - New: `vello_gpu` 0.3.0 and `vello_common` 0.3.0
  - Blitz renders through `anyrender` 0.14.0, which has vello, vello_cpu, vello_hybrid and skia backends.
- **Parley:** `parley` 0.12.0 (2026-10-09). Blitz uses it.
- **Architecture note:** the stack as stated ("WebRender/wgpu") does not exist off the shelf. Choose one:
  - WebRender on GL (or ANGLE on Windows)
  - Vello on wgpu
  - a WebRender fork with a new wgpu backend
- **Confidence:** H for versions; M for "no wgpu backend".

## Q9. Rust IPC/IDL for Mojo-like typed capability IPC

- **`ipc-channel` 0.23.0** (2026-09-01, servo/ipc-channel, active)
  - Platforms and transports:
    - Unix (Linux, Android, BSD, Illumos): `SOCK_SEQPACKET` + `SCM_RIGHTS`, with `memfd` shared memory
    - macOS/iOS: Mach ports and out-of-line memory
    - Windows: named pipes + `DuplicateHandle` + `CreateFileMapping`
    - An in-process fallback
  - Only its own types can travel inside a message: `IpcSender`, `IpcReceiver`, `IpcSharedMemory` (zero-copy `from_bytes`) and opaque channels. These are passed as fds, ports or handles.
  - There is **no public API to send an arbitrary `OwnedFd`, Mach port or `HANDLE`**.
  - There is no IDL; message types are serde types.
- **Mojo Rust bindings** exist **only inside Chromium**, at `mojo/public/rust/` (`c_mojo_api`, `system`, `bindings`, `mojom_value_parser`).
  - A `.mojom` file is turned into a Rust crate by the GN `mojom()` template. The bindings provide `PendingRemote<dyn Trait>`, `Remote` and `Receiver`.
  - They are GN-only and not on crates.io. They require vendoring `//mojo` core.
  - Source: https://chromium.googlesource.com/chromium/src/+/main/mojo/public/rust/
  - Confidence: H that they exist; maturity unknown.
- **Fuchsia FIDL Rust:** not on crates.io.
  - Host builds use `fuchsia_emulated_handle`, which emulates handles in-process; it is not real cross-process OS handle transport (M).
  - The only way to use it is to vendor the Fuchsia tree.
- **`interprocess` 2.4.5** (2026-10-10): local sockets and named pipes.
  - Unix `SCM_RIGHTS` support was removed in 2.0.
  - Windows has the `ShareHandle` helper.
  - No shared memory.
- **Cap'n Proto: `capnp` 0.27.2 and `capnp-rpc` 0.27.0.** Capability RPC with promise pipelining.
  - **No fd passing in the Rust implementation.** PR capnproto-rust#666, which adds it, is still open as of 2026-06. The C++ implementation does support fd passing.
- **Other crates:**

  | Crate | Version | What it gives you |
  |---|---|---|
  | `tarpc` | 0.38.0 | No fd passing |
  | `zbus` | 5.19.0 | fd passing on Unix (`OwnedFd`) |
  | `sendfd` | 0.4.5 | `SCM_RIGHTS` |
  | `passfd` | 0.1.6 | `SCM_RIGHTS`; stale |
  | `memmap2` | 0.9.11 | Memory mapping; combine with memfd + sendfd |
  | `shared_memory` | 0.12.4 | Shared memory; unmaintained |

- **Summary:**
  - Handle/fd passing and shared memory, cross-platform: only `ipc-channel` (restricted to its own types) and Chromium Mojo (in-tree only).
  - **Architecture implication:** a Mojo-like typed capability IPC in Rust has to be built in-house. One plausible design:
    - an IDL/codegen layer on top of an `ipc-channel`-style transport
    - a custom platform layer for generic handle passing: `SCM_RIGHTS` on Unix, Mach port descriptors on macOS, `DuplicateHandle` brokering on Windows
- **Confidence:** H.

## Q10. Deterministic simulation testing in Rust (2026)

| Crate | Latest | Status | Notes |
|---|---|---|---|
| `madsim` | 0.2.34 (2025-10-11) | Maintenance-only; last commit 2026-02 | Simulates tokio, tonic and etcd; overrides libc time and randomness. Used by RisingWave. |
| `turmoil` (tokio-rs) | 0.7.2 (2026-04-24) | Active; last commit 2026-10-05 | Simulates hosts, network, time and faults. Not fully deterministic alone. |
| `shuttle` (awslabs) | 0.9.7 (2026-10-14) | Very active | Randomized/PCT schedule exploration of concurrency. |
| `loom` | 0.7.2 (2024-04) | Releases stale | Exhaustive C11-memory-model concurrency checking. |
| `mad-turmoil` (S2) | 0.2.1 (2026-02) | Small | turmoil + seeded RNG + libc clock/getrandom symbol overrides. |

- **S2's recipe** (https://s2.dev/blog/dst): a single-threaded runtime plus `mad-turmoil`. Determinism is checked by running the same `DST_SEED` twice and diffing the logs.
- **DST with V8 or a browser:** no public framework found (M).
  - Nearby work: the RepliCore paper (arXiv 2607.02529), on reproducible scheduling of browser async runtimes; and `iv8-rs` (PyPI `ming-iv8-rs`), which claims "deterministic seeds, logical clocks" for V8. Both unverified.
- **Implication:**
  - Libc-symbol interposition (the madsim / mad-turmoil approach) **does not reach V8 internals**. V8 uses its own `base::OS` time and thread functions, plus its own platform threads.
  - So V8 determinism has to come from the Q2 mechanisms: a custom Platform with clock overrides, `--single-threaded` / `--predictable`, and `--random-seed` / `--hash-seed`.

## Q11. WPT with a custom product, and test262

- **Product registry:** `tools/wptrunner/wptrunner/products.py` (`BUILTIN_PRODUCTS`). Built-in products include `ladybird`, `servo`, `servo_legacy`, `wktr` and others.
- **Out-of-tree products are supported** (added 2026-03-14) through the Python entry-point group `wptrunner.products`.
  - A plugin returns a `Product` dataclass with: `browser_classes`, `check_args`, `get_browser_kwargs`, `get_executor_kwargs`, `env_options`, `get_env_extras`, `get_timeout_multiplier`, `executor_classes`, plus optional `run_info_extras`, `update_properties` and `add_arguments`.
  - `./wpt run` falls back to `GenericBrowserSetup`.
  - This means a custom browser no longer needs a fork of WPT.
- **Built-in module format:** a `__wptrunner__` dict. `browsers/ladybird.py` is the simplest template:
  - `LadybirdBrowser(WebDriverBrowser)` only overrides `make_command`.
  - Executors: `WebDriverTestharnessExecutor`, `WebDriverRefTestExecutor`, `WebDriverCrashtestExecutor`, and `PytestExecutor` for wdspec.
  - The `test262` test type is also run via `WebDriverTestharnessExecutor`. WPT gained test262 support on 2026-04-20 (#59336).
- **Executors available:**
  - `executorwebdriver` (WebDriver classic, plus `WebDriverBidiProtocol` with browsingContext, script, events, emulation, permissions and other BiDi parts)
  - `executormarionette`
  - `executorchrome`
  - `executorservo` (now WebDriver-based; it launches `servo --webdriver=PORT`)
  - `executorservolegacy` (old servodriver path)
- **How Ladybird runs WPT** (`Meta/WPT.sh`): `./wpt run --product ladybird --webdriver-binary=.../WebDriver` with `--webdriver-arg=--headless`, `--force-cpu-painting` and `--default-time-zone=UTC`.
- **Implication:** implementing a WebDriver(-BiDi) server gives WPT, wdspec and test262-in-browser coverage with essentially no WPT-side code.
- **test262 options:**
  - **V8's `tools/run-tests.py test262`** with d8. `test/test262/testcfg.py` prepends the harness plus `harness-adapt.js`, `harness-agent.js` and `harness-done.js`, and uses `--allow-natives-syntax` for `detachArrayBuffer`. An embedder can reuse it by shipping a d8-compatible shell.
  - **`test262-harness` 10.0.0** and **`eshost-cli` 9.0.0** (npm, both 2022). They need a host adapter.
  - **Deno does not run test262.** Its test suites are WPT, node_compat and unit tests.
  - **Boa's `boa_tester`** is in-repo and unpublished.
  - **WPT's own test262 type**, run via WebDriver.
- **Sources:**
  - https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/products.py
  - https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/browsers/ladybird.py
  - https://github.com/LadybirdBrowser/ladybird/blob/master/Meta/WPT.sh
  - https://chromium.googlesource.com/v8/v8/+/refs/heads/main/test/test262/testcfg.py
- **Confidence:** H.

## Q12. Fuzzilli and Domato

- **Fuzzilli** (googleprojectzero/fuzzilli, active) can target any process that implements REPRL; it is not limited to d8.
  - Profiles in `Sources/Fuzzilli/Profiles/`:
    - V8: `V8`, `V8Common`, `V8Sandbox`, `V8HoleFuzzing`, `V8Dumpling`
    - Others: JSC, SpiderMonkey, Duktape, JerryScript, njs, QJS, QtJS, XS, Serenity/LibJS
  - There is no browser target, but the protocol is engine-agnostic. A new target is a new Profile registered in `Profile.swift`.
- **What an embedder must do:**
  1. Implement REPRL over fds 100–103 (`CTRL_IN`, `CTRL_OUT`, `DATA_IN`, `DATA_OUT`):
     - Handshake: "HELO".
     - Loop: read "cexe", then a u64 length, then the script.
     - Execute, then write `(status & 0xff) << 8` to the control fd.
     - Reset state and call `__sanitizer_cov_reset_edgeguards()`.
  2. Build with `-fsanitize-coverage=trace-pc-guard` and link `Targets/coverage.c`. The edge bitmap lives in shared memory opened via `shm_open(getenv("SHM_ID"))`.
  3. Add a `fuzzilli()` builtin for crash self-tests. Use ASan with `abort_on_error=1`.
  4. Note: V8 sanitizer coverage is not supported on macOS, so fuzz on Linux.
- **Implication:** our engine's headless shell can be a Fuzzilli target that exercises V8 together with the DOM bindings. The bindings need SanitizerCoverage instrumentation on the Rust side too (`-C passes=sancov-module`, `-Cllvm-args=-sanitizer-coverage-trace-pc-guard`).
- **Domato** (googleprojectzero/domato) is dormant: last commit 2024-11-26, not archived.
- **Mozilla tools:**
  - Dharma: archived (2021).
  - Grizzly: active harness (2026-08).
  - Domino: no public repo.
- **Sources:**
  - https://github.com/googleprojectzero/fuzzilli/blob/main/Targets/README.md
  - https://github.com/googleprojectzero/fuzzilli/blob/main/Sources/libreprl/include/libreprl.h
  - https://github.com/googleprojectzero/domato
- **Confidence:** H.

## Q13. Brave adblock-rust

- **Crate:** `adblock` 0.13.3 (2026-08-20). License **MPL-2.0**. Repo https://github.com/brave/adblock-rust
- **Servo:** not integrated.
  - Open issue servo/servo#45934 "Integrate adblock-rust" (2026-06-24), linked to discussion #40859.
  - Open issue #48884 "Expose a content-blocker hook on WebViewBuilder" (2026-10-13).
  - Today, Servo embedders block requests themselves via `WebViewDelegate::load_web_resource`.
- **Verso:** archived (2025-10). No adblock.
- **Unverified claim:** issue #45934 says Ladybird and Firefox recently integrated adblock-rust (L).
- **Confidence:** H for crate details and Servo status.

## Q14. AccessKit

- **Versions** (all 2026-09-25, https://github.com/AccessKit/accesskit):

  | Crate | Version | Platform API |
  |---|---|---|
  | `accesskit` | 0.25.1 | core |
  | `accesskit_windows` | 0.35.1 | Windows UIA |
  | `accesskit_macos` | 0.27.1 | macOS NSAccessibility |
  | `accesskit_unix` | 0.24.0 | Linux AT-SPI |
  | `accesskit_android` | 0.9.0 | Android |
  | `accesskit_winit` | 0.34.1 | winit integration |
  | `accesskit_consumer` | 0.39.1 | — |

  - Licenses are MIT OR Apache-2.0, except `accesskit_winit`, which crates.io lists as Apache-2.0.
- **Servo uses it:** `accesskit` 0.24.0 and `accesskit_consumer` 0.35.0.
  - Code: `components/layout/accessibility/{accessibility_tree,accessibility_node}.rs` and `components/shared/layout/accessibility.rs`, plus the embedder and servoshell (egui-winit).
  - Still maturing: open issues #47799 (Orca), #46834 and #48669 (Linux), #43993 (WPT a11y tests).
- **Blitz uses it:** `accesskit` 0.25 through its own `packages/accesskit_xplat` wrapper over the platform adapters (not `accesskit_winit`).
- **Confidence:** H.

---

## Surprises that affect architecture decisions

1. **V8 sandbox has no prebuilt archives, and the crates.io source build is broken** (rusty_v8 #2070). Windows also has no pointer-compression prebuilt. Plan for our own CI build of the V8 archive from a git checkout of rusty_v8, served through `RUSTY_V8_MIRROR`.
2. **rusty_v8's custom platform only intercepts foreground tasks.** Clocks and worker threads still come from `DefaultPlatform`. Deterministic simulation needs either a small binding.cc fork (modelled on d8's `PredictablePlatform`) or `--single-threaded` plus JS-level Date overrides. `Isolate::SetRandomNumberGenerator` does not exist; use `--random-seed` (and `--hash-seed` with `--rehash-snapshot`).
3. **Servo layout cannot be reused over a foreign DOM today.** It depends concretely on the `script` crate, even after the April 2026 trait rewrite. Use Stylo (crates.io 0.22) + Taffy (0.15, floats now supported) + Parley (0.12), as Blitz does.
4. **"WebRender/wgpu" does not exist.** WebRender (crates.io 0.70, Servo-maintained) is GL-only. Choose GL WebRender, or Vello on wgpu.
5. **No off-the-shelf Rust Mojo equivalent.** `ipc-channel` cannot pass arbitrary handles; Mojo Rust is Chromium-only; capnp-rust fd passing is unmerged. Plan to build the IPC layer.
6. **WPT now accepts out-of-tree product plugins (2026-03) and runs test262 via WebDriver.** Implementing a WebDriver server early pays off.
7. **html5ever `TreeSink` has taken `&self` since 0.28** (2024), not 0.30. This changes nothing in practice.
