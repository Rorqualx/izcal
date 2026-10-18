# HGBA Stage 1 architecture design: planning pass

*Planning-pass output for HGBA-2.1 (Section 6; current revision 2.1.6) and HGBA-3.1 (Section 8; current revision 3.1.5). Date: 14 October 2026. Status: draft 0.9.3 (0.9 = M0 results folded in; 0.9.1 = consistency pass; 0.9.2 = 17 October 2026: product name Izcal (D-S9), `v8-cppgc-ext` retired under HGBA-2.1.6 S-4e; 0.9.3 = 17 October 2026: license changed to MIT (D-S6), and Izcal copies no other browser's code (D-S10), so Servo's table code is not ported; D-S8 deferred). Section 11 defaults signed off by the project owner on 14 October 2026. Technical facts verified 14 October 2026; sources in `research_notes/Stage 1 design facts/facts.md`.*

HGBA-2.1 and HGBA-3.1 say **what** must be true and how each claim can fail. This document says **how the engine is built**: the process map, the crate graph, the substrate, IPC, the renderer pipeline, the shell features, the build and test infrastructure, the milestones and the team. It changes no target. Where it makes a design choice the specs leave open, it states the choice, the alternative and what would reverse it. Choices that belong to the project owner are collected in Section 11, each with a recommended default.

**Four verified facts change the specs, not only this design** (Section 12 lists the amendments):

1. **Servo's layout cannot run over another DOM.** Its `layout_api` crate was rewritten in April 2026, and layout still depends on the `script` crate's concrete node types. HGBA-3.1's plan to reuse Servo layout as the Stage 1 baseline for tables and floats does not hold. This design follows Blitz's route instead: Stylo + Taffy + Parley, with Servo used as a black-box reference engine.
2. **WebRender is OpenGL-only.** Servo's published fork (`webrender` 0.70) has no wgpu backend. HGBA-2.0's "WebRender in a wgpu-based GPU service" is two GPU stacks, not one.
3. **No sandbox-enabled V8 prebuilt exists.** `rusty_v8` publishes `_ptrcomp` archives for macOS and Linux but no `_sandbox` archive for any target, and the sandbox source build from the crates.io package is currently broken (rusty_v8#2070). The project builds V8 itself from a git checkout.
4. **V8 determinism needs C++ changes.** `rusty_v8`'s `PlatformImpl` intercepts only foreground task posting. Worker threads and both clocks stay with V8's default platform, so `Date.now` cannot be faked from Rust. A d8-style `PredictablePlatform` in `rusty_v8`'s C++ glue is required, which makes a project fork of `rusty_v8` the plan of record from M0, not a fallback.

Naming: crates use the prefix `hg-`. Crate-graph rules are G1–G8; accessibility repair classes are R0–R4 (HGBA-3.1.1 B10). "Stage 1" means HGBA-2.1's Stage 1, which HGBA-3.1 overlays with Stages A and B. Thresholds marked † are set by this document from first principles.

---

## 1. Process map

| Process | Count | Sandboxed | Launched when | Owns |
|---|---|---|---|---|
| **Browser** (broker) | 1 | No (trusted) | Always | Shell UI, navigation and session history (constellation responsibilities), frame tree, site-to-process map, capability grants, permission store, profile and policy, update client |
| **Renderer** | 1 per site (desktop) | Yes, zero ambient authority | Per site instance | V8 isolate(s), `cppgc` DOM, bindings, data plane, style, layout, paint, semantic plane |
| **GPU service** | 1 | Yes | First paint | wgpu device, WebRender, compositor, WebGPU |
| **Network service** | 1 | Yes (S-6 target; Stage 1 best effort per platform) | Always | HTTP/1.1/2/3, TLS (X25519MLKEM768), cookies, cache, CORS/CSP/HSTS/LNA enforcement, content-blocking engine |
| **Storage service** | 1 | Yes | First storage use | SQLite (WAL) for IndexedDB, Cache API, localStorage, cookies at rest |
| **Utility** | Pooled | Yes | On demand | Media and image decode (RLBox-style for C libraries), font parsing, **PDF parsing and rendering** |
| **Protected-media host** | 0 or 1 per protected tab | By the host engine's own multi-process sandbox | **Only** when a tab enters protected media mode (D-S7) | The OS's licensed web engine (WebView2 on Windows, WKWebView on macOS) running a page that needs DRM HGBA's engine lacks. Outside HGBA's renderer trust boundary; reachable only through the browser-mediated switch |
| **Shell renderer** | 1 | Yes, zero ambient authority | Always | The HTML shell (D-S2) as web content; reaches browser state only through the capability endpoints the broker grants it |
| **ML service** | 0 or 1 | Yes | **Only** if the AI switch or the accessibility-model setting is on | OS ML runtime sessions (HGBA-2.0 Section 4.5) |
| **Agent host** | 0 or 1 | Yes | **Only** if the AI switch is on | Agent principal, origin sets, tool registry (B9) |

**Isolation key.** Renderer processes and all browser-side state are keyed by **(site, container)**, where a container is a policy context (Section 6). On devices above a RAM threshold, the site component becomes the origin, so subdomains of one registrable domain stop sharing a process; that is the only case origin keying adds, because site keying already separates different registrable domains (Kilger et al., IEEE S&P 2026, exfiltrated data through same-site subdomains sharing a process). The threshold is a decision-log entry set by measurement against S-3, with containers multiplying the process count on top. The network service partitions **all** of its state by (top-level site, container), following Chrome's network-state partitioning rather than an enumerated list: HTTP cache, connection and socket pools, DNS cache, Alt-Svc, TLS session tickets, HSTS, revocation-check caches, reporting endpoints and service-worker registrations. Each of these is an axis on which an attacker who can measure timing could otherwise correlate two containers.

AG-0 is a property of this table: with defaults, the last two rows have count 0, and their crates are not linked into any process that does launch (Section 2.3, rule G4).

**Mobile and low-memory devices.** Below about 2 GB of RAM, HGBA-2.0 I-1 requires a documented reduced-isolation mode, not silent co-location. Stage 1 ships desktop only; Android is S-8 in Stage 2.

---

## 2. Crate graph

### 2.1 Layers

```
 L6  apps        hg-browser-main  hg-renderer-main  hg-gpu-main  hg-net-main  hg-storage-main  hg-utility-main
                 (hg-agent-main, hg-ml-main: built only with feature `ai`)
 L5  services    hg-shell  hg-constellation  hg-policy  hg-profile  hg-update
                 hg-net  hg-storage  hg-gpu  hg-media  hg-content-block
 L4  renderer    hg-script (event loop, V8 isolate, microtasks)   hg-paint   hg-semantic
                 hg-style (Stylo adapter)   hg-layout (driver; Servo is a black-box reference engine, not a crate, per fact 1)
 L3  document    hg-dom (cppgc nodes)   hg-bindings (generated)   hg-dataplane   hg-projection
 L2  contracts   hg-tree-traits   hg-ipc-gen (generated endpoints)   hg-types (geometry, ids, atoms)
 L1  substrate   hg-substrate (traits)   hg-substrate-os   hg-substrate-sim   hg-ipc (transport)
 L0  external    v8 (project fork), stylo, selectors, taffy, parley, html5ever, webrender, wgpu, rayon, ...
 tools           hg-webidl-codegen  hg-ipc-codegen  hg-dst  hg-wpt  hg-corpus  hg-graph-lint  hg-bench
```

### 2.2 Dependency rules (enforced by `hg-graph-lint` on `cargo metadata`, in every PR)

| Rule | Statement | Source |
|---|---|---|
| G1 | `hg-style`, `hg-layout`, `hg-semantic` and `hg-paint` depend on `hg-tree-traits`, never on `hg-dataplane` or `hg-dom` | HGBA-3.1 non-goal; makes the dual-implementation oracle possible |
| G2 | `hg-projection` and `hg-dataplane` share no project-local, non-std dependency other than `hg-tree-traits` and `hg-types` | HGBA-3.1 non-goal (projection and log application share no code) |
| G3 | Only `hg-substrate-os` may name `std::time`, `std::thread`, `std::net`, `std::fs`, `std::process`, `rand::thread_rng`, `getrandom` or `Instant::now` | B1. Enforced twice: graph lint, and `clippy::disallowed_methods`/`disallowed_types` in a workspace `clippy.toml` |
| G4 | No crate reachable from a default-feature `hg-*-main` binary depends on `hg-agent*` or `hg-ml*` | B10/AG-0 ("off means not loaded") |
| G5 | Every crate is `#![forbid(unsafe_code)]` unless listed in `unsafe-inventory.toml` with a named owner | HGBA-2.0 Section 4.8 |
| G6 | Only `hg-dom`, `hg-bindings` and `hg-script` may depend on `v8` (the project's fork) | Keeps the V8 surface auditable and the D2 migration path bounded |
| G7 | Renderer-side crates never depend on `hg-net`, `hg-storage` or `hg-gpu` internals, only on their generated IPC endpoints | HGBA-2.0 I-2 (zero ambient authority) |
| G8 | `hg-semantic-repair` depends only on `hg-tree-traits` and the semantic tree's read interface; it cannot name DOM mutation, style or data-plane write APIs | HGBA-3.1.1 B8 (the repair overlay never feeds back into content-observable state; A11Y-1) |

### 2.3 Why this shape

The layering puts the two things HGBA-3.1 says cannot be retrofitted (the substrate and the tree-access traits) at the bottom, so every later crate is born under them. G1 and G2 are the oracle rules; G3 is the determinism rule; G4 is the AI-off rule. All four are mechanical, so they cost nothing per review and fail loudly.

---

## 3. Runtime substrate (B1)

### 3.1 Interfaces

The substrate is a set of synchronous traits plus a task model. Async traits are avoided in public interfaces because `dyn` async traits are not yet stable in a usable form (HGBA-3.1 B3).

```rust
/// Time. Production: OS clocks. Simulation: virtual, advanced only by the scheduler.
pub trait Clock: Send + Sync {
    fn monotonic(&self) -> Instant;      // hg_types::Instant, not std
    fn wall(&self) -> SystemTime;        // for Date.now(), cookies, HTTP dates
}

/// Randomness. Every consumer gets a named, seeded stream so adding a consumer
/// does not perturb existing ones.
pub trait Entropy: Send + Sync {
    fn stream(&self, purpose: EntropyPurpose) -> Box<dyn RngCore + Send>;
    fn crypto(&self) -> &dyn CryptoRng;  // simulation: seeded; production: OS CSPRNG only
}

/// Serial task queues ("sequences", B3). The substrate owns every thread.
pub trait Executor: Send + Sync {
    fn sequence(&self, name: &'static str, priority: Priority) -> SequenceHandle;
    fn post(&self, seq: &SequenceHandle, task: Task, delay: Option<Duration>);
    fn fork_join(&self) -> &dyn PhasePool;   // Rayon in production, see 3.3
}

/// Byte transport. Only the network service holds one. Renderers never do (G7).
pub trait NetTransport: Send + Sync {
    fn connect(&self, target: &SocketTarget) -> Result<ConnId, NetError>;
    fn send(&self, conn: ConnId, bytes: Bytes);
    fn on_event(&self, conn: ConnId, sink: SequenceBound<NetEvent>);
}

pub trait Storage: Send + Sync { /* files in the storage service only */ }
pub trait Ipc: Send + Sync { /* Section 4 */ }

pub struct Substrate {
    pub clock: Arc<dyn Clock>, pub entropy: Arc<dyn Entropy>,
    pub exec: Arc<dyn Executor>, pub ipc: Arc<dyn Ipc>,
    pub net: Option<Arc<dyn NetTransport>>, pub storage: Option<Arc<dyn Storage>>,
}
```

DET-2 bounds the cost of this indirection at ≤1%†. If dynamic dispatch misses it, the hot interfaces (`Clock`, `Executor::post`) become generic parameters on the renderer's top-level type, which removes the vtable without changing any call site.

### 3.2 Simulation mode

`hg-substrate-sim` runs the **whole multi-process browser in one OS process**. Each simulated process gets its own substrate instance, and the IPC transport delivers messages through the scheduler. One seeded scheduler picks the next runnable task among all sequences of all simulated processes. Virtual time advances only when no task is runnable, jumping to the next timer. Faults are injected by the scheduler: network resets, slow and reordered bytes, IPC delay, process crash and restart, GPU-device loss (HGBA-2.0 R-2).

The network side replays a **recorded corpus archive**. Format: one archive per page load holding every request, response, timing and the seed. Stage 1 writes its own compact format with a WARC importer, so corpora recorded by other tools can be reused.

### 3.3 Where determinism comes from in each layer

| Layer | Determinism mechanism | Risk |
|---|---|---|
| Event loop, IPC, timers, network | Seeded scheduler and virtual clock | Low; this is the FoundationDB/turmoil model |
| Style, layout, paint, semantic phases | **Phase purity**: workers read immutable inputs and write disjoint outputs, so Rayon's interleaving cannot change results. DET-1 compares parallel and sequential hashes | Medium: any shared mutable cache inside a phase (e.g. a style-sharing cache with racy insertion order) breaks it. Stylo's sharing cache is per-thread; whether its *results* depend on which thread styled which element is a Stage A check |
| JS semantics | V8 is deterministic given the same inputs, except where wall time, randomness or GC timing is observable. Under the virtual clock, JIT speed does not matter: time advances only through the scheduler, so a faster or slower tier cannot change what `performance.now()` returns | See below |
| JS: `Math.random`, `Date.now`, `performance.now` | `performance.now` is the embedder's, so it reads the substrate clock. `Date.now` reads V8's platform clock, which `rusty_v8` cannot override today; the fork adds a `PredictablePlatform`-style override (as d8 does) that routes it to the substrate clock. `Math.random` is seeded with `--random-seed`, and the process-wide `v8::V8::set_entropy_source` is pointed at the substrate's entropy stream | Medium: needs the fork's C++ change, which is small but must be carried across V8 updates |
| JS: GC-observable behavior | `WeakRef`, `FinalizationRegistry` and cppgc finalization order depend on when GC runs. Simulation mode runs V8 with `--predictable` (which implies single-threaded GC, no concurrent recompilation and a fixed random seed) plus `--single-threaded`, and its platform runs worker tasks inline on the scheduler. All these flags exist in V8 15.2 | **Highest**: if V8 cannot make GC timing a pure function of allocation, DET-1 needs whole-VM determinism for this layer (HGBA-3.1 Q-3) |
| GPU, audio, media | Out of scope for simulation in Stage A. Paint is checked as a display-list hash; raster is checked by reftests and `rr` | Accepted |

**The first Stage A experiment** is therefore a V8 determinism probe: run a corpus page's script 1,000 times under the simulation substrate with `--predictable --single-threaded`, the forked predictable platform and a seeded entropy source, and compare DOM serialization hashes. It answers HGBA-3.1 Q-3 early, before the substrate is built around a false assumption. No published simulation-testing framework has been used with V8 or any browser engine, so this probe is first-of-kind.

**M0 result (14 October 2026).** The probe ran in M0 on all three platforms. **The fork's predictable platform makes V8 deterministic; the flags do not.** With it, all runs matched (1,000 of 1,000 in-process, 50 of 50 fresh processes, release and debug), and the hashes were identical on macOS, Linux and Windows. With V8's stock platform they diverged on every OS, even with `--predictable --single-threaded`. Design consequences: (a) the worker-task mode (inline or queued) is part of the determinism configuration; it changes results in release builds, so the recording format carries it as a **mandatory** field from M1, and the simulator refuses recordings with a missing or unsupported mode; (b) the fork is load-bearing for DET-1, recorded in HGBA-3.1.4 D10. Details: `m0/determinism_crossplatform.md`.

**Rust DST libraries** are models, not dependencies: `turmoil` 0.7 is active, `madsim` 0.2 is in maintenance, and `shuttle` 0.9 (randomized concurrency testing) is very active and useful for the substrate's own unit tests. None of them can reach inside V8, which is why the platform fork is needed.

### 3.4 Production recording

In production the same interfaces can log their inputs, which gives replayable bug reports. Recording is **off by default** and user-initiated per tab, because the log contains page content. Replays run in the simulator.

---

## 4. IPC (B2)

### 4.1 Model

- **IDL.** Interfaces are declared in `.hgidl` files: methods, typed messages, and handle-carrying fields (endpoints, shared-memory regions, files). `hg-ipc-codegen` generates Rust endpoint types, serializers and a Kani harness per message.
- **Endpoints are capabilities.** A `Remote<T>` can only be obtained by being sent one. Renderers start with exactly one endpoint, to the browser process, and receive the rest from the broker.
- **Sequence affinity.** An endpoint is bound to one sequence. Messages are delivered as tasks on it, so the event loop and IPC share one ordering model and the simulator controls both.
- **Synchronous calls** are an explicit IDL attribute with an allowlist reviewed by the security owner. Expected Stage 1 users: `document.cookie`, `localStorage` reads, and sync XHR. Everything else is async.
- **Serialization (VER-1).** A fixed, little-endian, length-prefixed wire format with no self-describing schema, so the generated parser is small and provable. Every message type gets a generated Kani harness in CI. EverParse-class proofs are a Stage A stretch goal for renderer-to-broker messages.
- **Bulk data** crosses as read-only shared memory, never as large messages.

### 4.2 Transports

| Backend | Mechanism | Status |
|---|---|---|
| OS, Linux | Unix domain sockets plus `SCM_RIGHTS`, `memfd` shared memory | Build |
| OS, macOS | Mach ports (send rights carry capabilities), `mach_vm` shared memory | Build |
| OS, Windows | Named pipes plus `DuplicateHandle` brokered by the browser process | Build |
| Simulation | In-memory queues delivered by the seeded scheduler | Build (`hg-substrate-sim`) |

**Why build.** No Rust equivalent of Mojo exists outside Chromium's tree. Servo's `ipc-channel` 0.23 passes its own channels and shared memory across processes on all three OSes but not arbitrary handles. Cap'n Proto RPC cannot pass file descriptors yet, `interprocess` 2.x dropped descriptor passing, and Fuchsia's FIDL bindings are not published. `ipc-channel`'s per-OS code is the best starting reference for the three OS backends, which are a few thousand lines each and sit on the Tier 1 verification list.

### 4.3 The constellation, rebuilt on this transport

HGBA-3.0 keeps the constellation's responsibilities and rebuilds the plumbing. In this design they live in `hg-constellation` inside the browser process: the frame tree, session history, navigation state machine, site-instance assignment (HGBA-2.0 I-1), process launch and crash recovery. Its state machines are written as pure functions over IPC events, so the simulator can drive them and property tests can check them (for example: no frame is ever committed into a process locked to another site).

---

## 5. Renderer pipeline

### 5.1 Script thread and event loop (B3)

One script thread per renderer runs one sequence per agent cluster. It implements the HTML event loop: task sources, microtask checkpoints (V8 microtasks in explicit mode, so checkpoints happen exactly where the HTML spec puts them), and the "update the rendering" steps. Frame timing comes from the GPU service as a vsync message, which in simulation is just another scheduled event.

### 5.2 DOM in `cppgc`

| Part | Design |
|---|---|
| Node header | `Member` links (parent, first child, last child, previous and next sibling), node type, flags, a `TracedReference` to the JS wrapper, and a **stable node ID** (generational: 32-bit slot plus 32-bit generation) |
| Rust-owned payload | Attributes, text data and per-type state live in plain Rust fields inside the GC object (spike item 4 pattern), dropped when the node is collected |
| Strings | Interned atoms for tag names, attribute names and common values, as in Servo's `markup5ever` atoms |
| Cleanup | A prefinalizer (the fork's `cppgc` API) releases the node's data-plane slot and semantic-plane entry before the node is swept |
| Wrapper lifetime | Wrapper ↔ node edges per spike item 3: the node holds a `TracedReference`, the wrapper holds the node through `Object::wrap` |
| Write barriers | Every `Member` write goes through generated setters, never raw stores. The debug-V8 CI configuration catches a missing barrier (HGBA-2.1 S-4c) |

**Slot reuse** was not tested by the spike. Generational IDs exist so that a stale ID held by the semantic plane or an agent can be detected rather than aliased onto a new node. A property test checks this.

### 5.3 WebIDL bindings

`hg-webidl-codegen` takes WebIDL from the specs (pinned via the WPT `interfaces/` directory, which mirrors the spec IDL) and generates the V8 backend: interface templates per realm, attribute accessors and operations, type conversions, overload resolution, `[CEReactions]`, `[SameObject]`, exceptions and the wrapper creation path. The front end (parser and resolved IDL model) is engine-agnostic, as HGBA-2.0 D2 requires. Two inputs exist: Mozilla's `WebIDL.py` parser, which Servo's Python generator uses and which is standalone (MPL-2.0, depends only on a vendored PLY), and the Rust `weedle` 0.13 parser, which gets occasional releases. **Choice: `weedle` for parsing plus a project-owned resolver in Rust**, so the build has no Python step, with `WebIDL.py` run in CI as a differential check on the resolved model for every pinned IDL file. The generated code is the largest single body of Stage 1 code, and **it is generated, not translated**, so its oracle is WPT `webidl/` plus `idlharness.js` tests for every interface, which exist before any interface is generated (ORC-1).

**Fast-call rule (from M0).** One of `rusty_v8`'s own tests (`test_fast_calls_pointer`) crashes with the sandbox on. Until the cause is known, generated bindings never pass raw pointers through V8's fast API calls. The rule is enforced at the generator by two checks in `hg-webidl-codegen`: a golden-file test on a fixed sample IDL, which catches generator regressions, and a check over the generator's output for the full pinned interface set (WPT `interfaces/`), which catches IDL patterns the sample does not contain.

### 5.4 Data plane and mutation log (B4)

```rust
pub struct DataPlane {
    gen:      Vec<u32>,              // generation per slot
    links:    Vec<[SlotId; 5]>,      // parent, first, last, prev, next (u32 each)
    tag:      Vec<Atom>,
    state:    Vec<ElementState>,     // hover, focus, checked, ... (bitflags)
    attrs:    AttrArena,             // interned; only attributes selectors can read (INC-3 trim)
    classes:  ClassArena,
    style_ref: Vec<Option<Arc<Locked<PropertyDeclarationBlock>>>>,
    dirty:    Vec<DirtyBits>,
}

pub enum Mutation {
    Insert { node: NodeId, parent: NodeId, before: Option<NodeId> },
    Remove { node: NodeId, old_parent: NodeId },
    SetAttr { node: NodeId, name: Atom, old: Option<AttrValue>, new: Option<AttrValue> },
    SetState { node: NodeId, old: ElementState, new: ElementState },
    SetInlineStyle { node: NodeId, block: Option<Arc<Locked<PropertyDeclarationBlock>>> },
    SetText { node: NodeId },
}
```

One design consequence is worth stating: **log entries carry old values**. Stylo's restyle invalidation needs a snapshot of an element's attributes and state before the change (its snapshot map). The log already holds those old values, so flush builds Stylo's snapshots directly from the log, without a second bookkeeping path.

Flush runs at the start of every style and layout pass and on every forced synchronous flush (HGBA-2.0 F1 contract, unchanged). `hg-projection` independently builds the same structure from the `cppgc` DOM for the equality oracle (G2).

### 5.5 Style

`hg-style` implements Stylo's `TElement`, `TNode`, `TDocument` and `TShadowRoot` (defined in `style/dom.rs`), plus `selectors::Element`, over `hg-tree-traits`. Stylo is on crates.io (`stylo` 0.22, `selectors` 0.41, both 30 September 2026). Blitz is the working precedent, but it implements no shadow DOM, so the `TShadowRoot` path is new work. Two implementations of those traits exist (G1): over the data plane for workers, and over the `cppgc` DOM for the script thread only, which is the dual-implementation differential. Stylesheets, CSSOM and inline style blocks are Stylo's own `Arc<Locked<...>>` data, as verified against Servo for HGBA-2.0.9.

### 5.6 Layout

| Formatting context | Stage 1 implementation | Oracle |
|---|---|---|
| Block, flex, grid, floats | Taffy 0.15 algorithms behind the `hg-layout` driver (tree traits `LayoutPartialTree`, `CacheTree` and others; floats on by default since 0.10) | Reference engines (Chrome, Firefox, Servo as a black box), WPT `css/` |
| Inline and text | Parley 0.12, with the driver owning inline formatting-context integration (Taffy leaves inline to the embedder) | Reference engines, WPT |
| Tables, fragmentation, writing modes, the long tail | **Built new** in the driver, ordered by WPT and corpus usage; tables first | Reference engines and WPT only. No in-process lockstep oracle exists for these |

The driver API is `layout(node, constraint_space) -> Arc<Fragment>` with fragments immutable and cached by constraint space (LayoutNG). Stage 1 implements caching and from-scratch layout only. The Spineless ordered dirty queue, durability tiers and demand-driven forced flush are Stage B (INC-1, INC-2, INC-4, FLUSH-1), added behind a switch and checked against from-scratch output on every frame in CI.

**Consequence of fact 1.** Servo's layout cannot be the in-process baseline, because it reads Servo's concrete DOM types. The table, fragmentation and writing-mode long tail is therefore new code on the Stage 1 critical path, and its oracle is weaker than HGBA-3.1 assumed: reference-engine differential and WPT (moderate strength), not lockstep equality (strong). Two mitigations: Servo is still usable as a **black-box reference engine** driven over WebDriver, which gives a Rust-lineage third opinion next to Chrome and Firefox; and Servo's table behavior can be studied through it. Servo's table code is **not** ported: Izcal copies no other browser's code (D-S10). Tables are written from the CSS 2 and CSS Table specifications instead. Under HGBA-3.1's EQ-1 these contexts are reported in the moderate oracle class.

### 5.7 Paint and pixel-optional rendering

Paint produces property trees (transform, clip, effect, scroll) and paint chunks, then translates them into a WebRender display list sent to the GPU service over shared memory.

**GPU stack (fact 2).** WebRender renders through OpenGL only. The Stage 1 GPU service therefore runs **two stacks**: WebRender on OpenGL for page raster and compositing (through ANGLE on Windows, as Firefox does, and native GL on macOS and Linux), and wgpu for WebGPU. Compositing WebGPU canvases means sharing textures between the two (platform shared-texture handles). HGBA-2.0's goal of one render graph for raster, compositing and WebGPU is not reachable in Stage 1. It becomes reachable only through Vello (`vello` 0.11, `vello_gpu` 0.3, both on wgpu), which P-2 already evaluates in Stage 2; P-2 now decides the single-stack goal as well as raster speed. The alternative of writing a wgpu backend for WebRender is rejected for Stage 1 as unbounded work in a component the project does not own. Pixel-optional mode (B7) stops after paint-chunk generation, which keeps the display-list hash available to DST while skipping raster and compositing. It applies only to agent tabs (AI switch on) and background tabs the shell marks as unobserved.

### 5.8 Semantic plane

`hg-semantic` builds the semantic tree from the data plane, computed style and fragments, with stable IDs taken from data-plane node IDs. Platform accessibility APIs are fed through AccessKit (`accesskit` 0.25, adapters for Windows UIA, macOS NSAccessibility, Linux AT-SPI and Android), which Servo and Blitz both use. The semantic plane produces AccessKit tree updates directly from its diffs. That serves the semantic plane's stable IDs well, since AccessKit is itself built around incremental tree updates keyed by node ID. Agent extras (provenance export, visibility flags, diffs for agents) are compiled only with feature `ai` (G4).

**Repair overlay (HGBA-3.1.1 B8 and B10).** Accessibility repairs live in `hg-semantic-repair`, a sparse map over the author tree keyed by stable node ID, isolated by crate rule G8. It is computed only when assistive technology is active and a repair class is enabled. Forced colors and contrast (R0) are style-system behavior and do not belong to this crate. R1 (AT-only language hint) is on; R2 (heuristic names) is opt-in; R3 is user-invoked per site; R4 (model-based) sits behind the accessibility-model setting. Each entry records class, rule, source nodes and provenance, and its sources must share the repaired node's origin and frame. AccessKit receives the author tree with overlay values applied; agents receive the author tree, with any repaired value tagged `repaired`. The reference-engine differential and WPT accessibility-mapping tests run on the author tree only. A "What the browser changed on this page" inspector lists every repair, and one toggle reverts them all.

---

## 6. Shell and browser features

HGBA-3.1 specifies the engine almost entirely. The pain-points review showed that most user-visible problems are decided in the shell. This section fixes the Stage 1 and 2 shell scope.

| Feature | Layer | Stage | Design | Pain points |
|---|---|---|---|---|
| Built-in content blocking | Engine hook in network service and style; shell owns lists | 1 | Network-level request blocking plus cosmetic filtering, using Brave's `adblock` crate (0.13, MPL-2.0). Not built on any extension API, so it is unaffected by extension-platform changes. Cosmetic filters are applied as a user-origin stylesheet, so they are WPT-neutral | Ads, trackers, page weight |
| Cookie-banner handling | Shell rules plus engine hook | 2 | Rule-based auto-reject following Firefox's cookie-banner reduction design; never auto-accept | Cookie banners |
| Notification prompts | Engine (user activation) plus shell (quiet UI) | 1 | Permission prompts require transient user activation; denied-by-default quiet UI; auto-revocation for high-volume, low-engagement grants (Chrome found under 1% of notifications get any interaction) | Notification spam (strong evidence) |
| Notification delivery controls | Shell | 2 | Per-site mute and snooze from the notification itself; opt-in per-site digest delivered on a schedule (no browser ships this yet); honors OS focus modes; never delays by default; no inferred urgency | Notification spam. Metric: post-grant revocation and complaint rates versus Stage 1 |
| Per-site resource meter | Shell plus browser process | 2 | CPU, memory, network and energy attributed per site, shown on request. Visibility only; the interventions with measured effects (heavy-ad limits, background timer throttling, discard) are engine defaults, not user budgets | Page weight, battery. Metric: none gated; usage reported only if telemetry is opted in |
| Autoplay | Engine | 1 | Audible autoplay blocked without user activation; muted autoplay allowed | Autoplay |
| Anti-fingerprinting | Engine | 1 baseline, 2 full | D-S1 hybrid. **Readback surfaces** (canvas, WebGL, audio, `measureText`-class metrics) get noise seeded by **(site, container, session)**, drawn from the substrate's `Entropy` stream, so two containers on one site cannot be linked by fingerprint, so DET-1 replays it exactly. **Discrete surfaces** (UA and client hints, screen and window metrics, timezone, languages, font enumeration, hardware concurrency, **WebGL and WebGPU adapter information**) are reduced or normalized, because random values there break sites or stand out. R0 accessibility preferences are never altered. Full uniformity (RFP-style) is a user opt-in | Fingerprinting. Metrics: PRIV-2 (linkability and residual entropy), PRIV-1 (CAPTCHA cost), WPT and corpus breakage under the P2 scope note |
| Third-party cookies | Network service | 1 | Partitioned by top-level site (CHIPS-style) by default | Tracking |
| Tab memory and bfcache | Browser plus renderer | 1 | **One memory policy owns both** tab discard and the back/forward cache. A bfcache entry is a frozen document that keeps its renderer, V8 context and data-plane state. Under memory pressure the policy evicts bfcache entries first, because losing one costs a reload, before discarding live background tabs, which lose state. A discarded tab's bfcache entries go with it. Entries also expire on their own timeout. Discard saves state through `visibilitychange`. Target BF-1 (HGBA-2.1.2) measures the hit rate | Tab overload, memory, navigation speed |
| PDF viewer | Utility process plus shell | 1 viewer, 2 editing | A Rust PDF renderer, or a C/C++ one in the RLBox-style library sandbox, running in a utility process (the Rule of 2: untrusted input, unsafe language or privilege, never both). Stage 2 adds local annotate, sign and redact | Documents |
| Protected media mode | Shell plus protected-media host | 1 (spikes M0, shell M7) | D-S7, Section 13.1a: automatic switch to the OS's licensed engine for pages that start encrypted playback HGBA's engine cannot handle; labelled, per (site, container) storage, switch-back, never for agent tabs | Paid content that will not play. Metric: per-service playback and resolution table from DRM-1 and DRM-2, re-run each release |
| Containers | Browser, network service, renderer keying | 1 key design, 2 UI | Isolation key (Section 1). Ships with a few named containers that have sensible defaults (for example personal, work, banking, shopping), editable, never a blank policy editor. Per-container policy: storage partition (always), block-list set, JavaScript mode, agent origin sets (HGBA-3.1 B9), network route. Routes are **user-supplied** (WireGuard config, SOCKS, Arti), not a vendor VPN service. A Tor route is labelled "hides your IP address, not your browser" and claims no Tor Browser anonymity, because the Tor Project recommends against Tor in any other browser | Identity separation, tracking. Metric: S-3 per-container memory |
| JavaScript modes | Engine plus shell | 2 | Three modes per site, inherited from the container: **off**; **same-site only** (scripts from the page's own site, meaning scheme plus registrable domain, with `eval` and `Function` refused per realm); and **full** (default). Chrome, Edge and Brave already ship per-site off; the same-site mode is new. Honest claims: off removes the JIT and binding attack surface and most active tracking, but not CSS fingerprinting, CSS exfiltration, form posts, tracking pixels or prompt injection. Same-site mode also trusts every subdomain of the site: a taken-over `evil.example.com` can serve scripts that `a.example.com` runs as same-site. Same-origin would close that gap but break the many sites that serve their own scripts from another subdomain, so the mode's description states the caveat instead | Security, tracking. Metric: corpus breakage of same-site mode, reported with the share caused by first-party scripts on CDN origins (which break under same-origin and same-site alike). User-invoked only (P2 scope note, class b) unless breakage is shown to be low |
| Save for offline | Shell plus engine | 2 | User-invoked. Default format is a frozen DOM-and-resources snapshot (simple, safe to open); the B1 replay archive is kept as an option for "exactly what happened". Reader rendering on top. Background refresh of saved pages or feeds is opt-in per source, because it makes requests the user did not make at that moment. One primitive, the B1 archive, serves both deterministic testing and offline reading | Offline reading |
| Tab-strip conveniences | Shell | 2 | Vertical tabs, split view and tab groups as **table stakes** built on the HTML shell, with no claim that they differentiate | Power-user expectations |
| AI switch | Shell plus process map | 1 | B10 exactly: off by default, per-profile switch, policy lock, no AI UI surfaces when off, survives updates | AI bloat |
| Accessibility repairs, rule-based | Engine (repair overlay) plus shell (settings, inspector) | 1: R0 and R1. 2: R2 opt-in, inspector | Section 5.8. R2 can become default-on only after A11Y-3 passes | Accessibility. Metrics: A11Y-1 to A11Y-4 |
| Accessibility model opt-in (R4) | Shell plus ML service | 2 | HGBA-3.1.1 B10: separate setting, local model, repairs recorded in the same overlay and inspector, disclosure per Q-10 | Accessibility |
| Global Privacy Control | Network service plus shell | 1 | `Sec-GPC: 1` sent by default. GPC is enforceable in California, Colorado and Connecticut, and California AB 566 requires browsers to offer an opt-out signal from 1 January 2027. Sending it by default is a P2 class (c) intervention with a recorded P1 analysis | Tracking, consent fatigue. Metric: honor rate on a corpus crawl (a 2025 study found 43–45%) |
| Default-browser handling | Shell | 1 | OS-sanctioned APIs only; never re-prompts after a "no" | Default-browser friction |
| Passkeys and credentials | Shell plus OS | 1 | WebAuthn through platform authenticators; no custom credential store in Stage 1 | Logins |
| Extensions | Wasmtime host (HGBA-2.0 W-1) | 2 | WASI 0.3 components; no extension code in renderers | Malicious extensions (D12 rationale) |
| Updates | Browser process | 1 | Signed, staged updates; settings migrations tested by AG-0's upgrade scenario | Trust |
| Usage telemetry | Browser process | 1 | **Opt-in** (D-S4); nothing sent without the user being able to see what is sent | Privacy |
| Crash reports | Browser process plus a separate crash-handler process | 1 | **Opt-out** (D-S4): a first-run disclosure, a prompt at each crash with "send" pre-selected (Firefox's model), and a one-click switch in settings. Minidumps only (thread stacks and registers), never full heap. The URL is **not deliberately collected**, and a best-effort filter removes common URL patterns, but stack memory can still hold page strings, including URLs, so the disclosure says so rather than promising exclusion. Reports are viewable before sending | Reliability |
| Recovery window | Native, per OS | 1 | D-S2 fallback: a minimal native window launched when the HTML shell fails to come up twice in a row. It can reset the profile, start without hardware acceleration and send the crash report, and nothing else. Never the default UI | Reliability |

**Out of scope for Stages 1 and 2:** UI chrome as differentiation (vertical tabs, spaces and split view ship in Chrome, Edge, Firefox and Vivaldi as of 2026; Arc's own data showed only 5.52% of daily users regularly used more than one Space); enforced per-site resource budgets; R3 structural repairs except when user-invoked; automatic overrides of scrolljacking, login walls or CAPTCHAs; a BitTorrent swarm as a shared cache (it reopens the cross-site leak that cache partitioning closed, now visible on the network); GPU hash verification; publishing browser state (media, meeting, focus) over MQTT or any channel by default; raw TCP or UDP sockets for web content; the browser as a window manager for local apps; claims that a Tor route gives Tor Browser anonymity; a browser-operated VPN service. Every default-on intervention in this table belongs to one of HGBA-3.1.1's P2 scope-note classes.

---

## 7. Build and test infrastructure

### 7.1 Build

- One Cargo workspace, pinned Rust toolchain, `cargo-vet`/`cargo-deny` for supply chain, vendored dependencies for release builds.
- **V8 is built by the project** (fact 3). CI builds the project's `rusty_v8` fork from a git checkout with submodules, in the sandbox configuration, for every target, and serves the archives to developers through `RUSTY_V8_MIRROR`. Developers never build V8 locally. Build machines need generous disk; `rusty_v8` publishes no per-build disk or time figures, so M0 measured them (table below). The fork also carries the `cppgc` APIs (HGBA-2.1 S-4e) and the predictable platform (Section 3.3), so `v8-cppgc-ext` is retired (HGBA-2.1.6 S-4e): with every build from source, the C++ is compiled inside the V8 build and no shim or separate ABI layer is needed.
- **M0 build facts** (`m0/M0_SUMMARY.md`). Sandbox-configuration builds of the fork, release and debug:

  | Platform | Release | Debug | Notes |
  |---|---|---|---|
  | macOS arm64 (M4, 10 cores) | 17.5 min, 161 MB archive | 28.6 min, 450 MB | Native arm64 Python and official LLVM 21+ libclang required (Homebrew's x86_64 builds fail) |
  | Linux x86-64 (huey, 6 jobs, shared) | 46 min, 202 MB | 63 min, 536 MB | libclang 21+ with **both** `LIBCLANG_PATH` and `CLANG_PATH` set, or the bindings step fails after the full compile; memory guards must watch the per-user cgroup limit, not `free` |
  | Windows x86-64 (laptop, 12 jobs) | about 20 min, 239 MB | 23.5 min, 533 MB | Debug from source works: `GN_ARGS=is_debug=true` overrides `rusty_v8`'s forced release setting, plus RUSTFLAGS linking `libcmtd`. Needs the VS Build Tools path set, the Windows SDK "Debugging Tools for Windows" component, git `core.autocrlf=false`, and job count capped to RAM (32 jobs exhausted 28 GB) |

  The fork must add `third_party/fadec` and `third_party/disarm` as submodules; their absence is rusty_v8#2070. Rebuilding after a change to `rusty_v8`'s own glue takes 15–40 s, because V8 itself is not recompiled. Windows release builds keep the V8 platform alive for the whole process until the teardown crash is root-caused (HGBA-2.1.4 S-4f). The farm runs the debug configuration on all three platforms. Thread-sanitizer runs on Linux work despite the 1 TiB sandbox reservation but need a suppression file for V8's uninstrumented code.
- Build caching with `sccache`. Release builds use LTO and PGO from the corpus.
- Three V8 configurations are built for every platform: **release-sandbox** (ships), **debug-sandbox** (heap verification, the S-4c oracle), **release-sandbox + TSAN** for Rust code (and TSAN V8 if it can be produced).

### 7.2 CI tiers

| Tier | Trigger | Budget† | Contents |
|---|---|---|---|
| T0 | Every PR | ≤20 min | fmt, clippy (with G3's disallowed lists), `hg-graph-lint` (G1–G8), unit and property tests, Kani harnesses for changed IPC messages, bindings tests on debug-V8, WPT smoke (≈2k subtests), 200 DST seeds, `v8-cppgc-ext` ABI test if V8 or features changed |
| T1 | Every merge | ≤2 h | Full WPT sharded (C-1, S-4d per-directory bound), test262 against the pinned V8 `.status`, projection equality and dual-trait differential on the fixed corpus sample, TSAN stress suite (S-4c), 10k DST seeds |
| T2 | Nightly | ≤12 h | 1M DST seeds, fuzzing campaigns, reference-engine differential (computed style and layout boxes vs Chrome and Firefox on generated inputs), corpus performance on all four reference devices, AG-0 harness |
| T3 | Weekly | ≤72 h | 72 h soak (S-4c(3)), full corpus energy (X-4), long fuzzing, PRIV-1 challenge-rate crawl (once privacy defaults exist) |
| Release gate | Per release | — | Every ship-blocker target in HGBA-2.1 Stage 1 and HGBA-3.1 Stage A/B, AG-0 fresh-install and upgrade scenarios |

### 7.3 Oracles as infrastructure

| Oracle | Harness |
|---|---|
| WPT | `hg-wpt`: an out-of-tree wptrunner product plugin (supported since March 2026; Ladybird's is a minimal template) driving the browser over a project WebDriver server, with expectations pinned per WPT revision. **The WebDriver server is therefore M2 infrastructure**, not a late feature |
| test262 | A d8-like shell built from the pinned V8 fork and run under V8's `run-tests.py`, gated on V8's own `.status` files. WPT's test262 test type, which runs test262 through WebDriver, is a second path that also exercises the bindings' realm setup |
| Reference-engine differential | Chrome and Firefox driven over WebDriver BiDi on generated DOM/CSS cases; extract computed styles and border boxes; compare with tolerance rules for known non-interop areas |
| Servo as a reference engine | Servo driven over WebDriver as a third black-box reference next to Chrome and Firefox. Not lockstep (fact 1) |
| Projection equality and dual-trait differential | Built into debug builds and the DST runner |
| Heap verification | Debug-V8 configuration |

### 7.4 DST farm

`hg-dst` runs seeds in parallel across machines. Each failure is minimized (shrinking the seed's fault schedule and input archive), deduplicated by crash signature, and filed with a one-line replay command. DST-1 compares its unique-bug yield per CPU-hour against coverage-guided fuzzing on the same surfaces, so both farms log CPU-hours and bug IDs in one database.

### 7.5 Fuzzing

- **Bindings and DOM:** a project-owned grammar fuzzer in the Domato style (Domato itself has been dormant since November 2024), run through the renderer in simulation mode so every crash is a replayable seed.
- **JS engine and embedding:** Fuzzilli works with any target that speaks its REPRL protocol and is built with sanitizer coverage. The renderer gets a REPRL mode, so Fuzzilli fuzzes V8 *through* the bindings, which is where the project's own bugs live.
- **Parsers and IPC:** `cargo-fuzz` targets for every IPC message type, URL, image, font and media container header (Tier 1 surfaces), registered with OSS-Fuzz.

### 7.6 Performance lab

Four reference devices (HGBA-2.0: x86-64 Windows laptop, Apple Silicon Mac, Linux desktop with Intel Gen12+ GPU, Android ≥4 GB later). The corpus is recorded archives (Section 3.2) of a published site list plus Speedometer 3, JetStream and MotionMark. Every merge to main gets an A/B run on one device; nightly runs all four. Metrics: p95 interaction latency, frame time, style and layout time (P-1), memory (INC-3, S-3), energy (X-4).

### 7.7 Security and upstream pipeline

| Upstream | Owner | Pin | Lag metric | Fork trigger |
|---|---|---|---|---|
| V8 / `rusty_v8` | `rusty_v8` owner (HGBA-2.1 D2) | Exact | Days behind Chrome stable V8 security fixes | Already forked from M0 (fact 4). Escalate to the owner at >2 weeks lag behind upstream, or if Deno stops maintaining `rusty_v8` |
| Fork's `cppgc` commits / upstream patch | Same owner | Rebased with every `rusty_v8` update | S-4e update procedure inside the security window | Not applicable: the APIs live in the fork (S-4e). If the rebase misses the window, V8 security commits are backported onto the pinned V8 (S-4e). The upstream patch is still offered and, if merged, shrinks the fork |
| Stylo, selectors | Style owner | Exact | Days behind Servo `main` security fixes | Planning-pass policy (HGBA-3.1 Section 8) |
| WebRender, wgpu | Graphics owner | Exact | Same | Same |
| Taffy, Parley, html5ever | Layout owner | Exact | Same | Same |
| Wasmtime (Stage 2) | Extensions owner | Exact | Bytecode Alliance advisories | Same |

A bot opens the V8 update PR within 24 h† of each Chrome stable release; T0 to T2 must pass; the security owner's SLA is to ship within one Chrome stable cycle (HGBA-2.1 D2).

### 7.8 Release engineering

Nightly, Beta and Release channels. Nightly carries the sampled projection-equality check (HGBA-3.1 B4) and reports mismatches as crashes. Symbol server and crash reporting follow D-S4 (opt-out minidumps with a per-crash prompt). Signed updates on all platforms.

---

## 8. Milestones and dependency graph

### 8.1 Milestones

| ID | Milestone | Exit criterion (demo plus measurement) | Depends on |
|---|---|---|---|
| M0 | Start conditions | DRM spikes DRM-1 and DRM-2 (D-S7); V8 build farm producing sandbox-configuration archives of the project's `rusty_v8` fork for three platforms; HGBA-2.1 (i)–(iii) met (spike rerun under sandbox on three platforms, exposure route = fork, debug-V8 CI live); `rusty_v8` owner named; team for M1–M2 staffed | — |
| M1 | Substrate and IPC skeleton | Browser, renderer and network processes start under both substrates; an IPC round trip runs in simulation with fault injection; V8 determinism probe on corpus pages (Section 3.3) reports (a synthetic-script probe already passed in M0); recordings carry the mandatory worker-task-mode field | M0 |
| M2 | DOM and first bindings | WebDriver server and `hg-wpt` product plugin running; html5ever (0.40) builds a `cppgc` DOM; `Node`, `Element`, `Document`, `Text`, `Event`, `EventTarget` generated; data plane plus projection equality on every flush; WPT `dom/nodes` passing at a recorded rate | M1 |
| M3 | Style | Stylo over `hg-tree-traits` with both implementations; dual-trait differential green on the corpus sample; WPT `css/` selectors and cascade subsets recorded | M2 |
| M4 | Layout and first pixels | Block, flex, grid and floats via Taffy, inline via Parley, tables new; WebRender (GL) display in the GPU service; reftests running | M3 |
| M5 | Network and navigation | Network service with HTTP/2 and 3, cookies, CORS and CSP; constellation navigation; site-per-process on desktop (S-1 probe suite); **Windows sandbox, process launch and IPC backend running** (D-S5), so Linux/macOS assumptions are caught before M6 | M1 (parallel with M2–M4) |
| M6 | Interaction | Input events, focus, forms, scrolling, `ResizeObserver`, custom elements; S-4c stress suite running | M4, M5 |
| M7 | Breadth | Images, fonts, media and PDF via utility processes; EME with Clear Key and native PlayReady (Windows); protected media mode in the shell (D-S7); storage service; bfcache with the shared memory policy (BF-1); Hardened profile (S-4a); content blocking, notification and autoplay policies; PQC for WebRTC DTLS | M6 |
| M8 | Stage 1 exit | Every HGBA-2.1 Stage 1 target and HGBA-3.1 Stage A target met or formally waived in the decision log | M7 |

### 8.2 Critical path

The critical path runs **M0 → M1 → M2 → M3 → M4 → M6 → M8**. Its longest segment is the DOM and bindings breadth inside M2 to M6: every web API surface is generated or written against `cppgc`, and WPT pass rate grows with it. Networking (M5) and infrastructure run in parallel tracks from M1. The second risk on the path is the layout long tail, which is new code (fact 1) and now sits inside M4 and M7.

### 8.3 Team and duration

HGBA-3.1 makes team scale an output of the planning pass. This design proposes **eight tracks** and a minimum staffing of **18–23 engineers**†:

| Track | Scope | Engineers† |
|---|---|---|
| Substrate and IPC | B1, B2, constellation, DST farm | 2–3 |
| JS, bindings and DOM | V8 embedding, codegen, DOM, event loop, web APIs | 5–6 |
| Style and layout | Stylo adapter, data plane, layout driver (Taffy, Parley, new long-tail layout, tables written from the CSS specifications, D-S10) | 3–4 |
| Graphics | Paint, WebRender, GPU service, HAL | 2 |
| Network and security | Network service, sandboxing per platform, VER-1 | 2 |
| Infrastructure | CI tiers, oracles, perf lab, fuzzing, release | 2–3 |
| Shell | UI, profile, policy, Section 6 features | 1–2 |
| Accessibility | Semantic plane, AccessKit integration, repair overlay, A11Y-1 to A11Y-4; plus a contracted accessibility researcher for the A11Y-3 study | 1–2, plus contractor |

**Duration.** No source supports a duration (HGBA-2.1, Section 6), so this document gives a method, not a date. The estimate is set after M2, from measured velocity: WPT subtests passing per engineer-month in the DOM track, extrapolated to the Stage 1 conformance targets, plus the fixed-scope items (substrate, IPC, sandboxing, CI). As a planning placeholder only, a first-principles guess for Stage 1 with the staffing above is **24–36 months**†. It is labelled as a guess, and it must be replaced at the M2 review.

---

## 9. Feature backlog by stage

| Stage | Engine | Shell | Agent (AI switch on) |
|---|---|---|---|
| 1 / A | Substrate, IPC, DOM, bindings, data plane, Stylo, layout driver (from-scratch plus caches), WebRender, network service, (site, container) isolation keying and network-state partitioning, bfcache, PDF viewer, Standard and Hardened profiles, semantic plane for platform accessibility | Tabs, navigation, settings, AI switch (off), content blocking, GPC, notification and autoplay policies, cookie partitioning, updates, passkeys, R0 and R1 accessibility | None shipped. Crates exist behind feature `ai` for development |
| 2 / B | Incremental layout (Spineless queue, durability, demand-driven flush), Vello Hybrid evaluation, Speculation Rules and prerender, JavaScript modes, network sandbox everywhere, library sandboxing, Android | Containers UI and per-container routes, save for offline, PDF editing, tab-strip conveniences, optional W-1 components (feeds, magnet handler with an IP-exposure warning, MQTT bridge with user-chosen topics, IRC), cookie-banner rules, full anti-fingerprinting model (R0 preferences exempt), extensions in Wasmtime, R2 repairs and inspector, accessibility model opt-in, notification delivery controls, resource meter, A11Y-3 study | Semantic diffs, provenance tags, agent principal with origin sets, pixel-optional tabs, native automation protocol |
| 3 / C | riscv64, energy targets, native Component Model (optional) | Sync (if wanted) | WebMCP tool registry (if the standard survives), deterministic snapshots for agents, AG-1 to AG-5 measured |

---

## 10. Risks specific to this design

| Risk | Where | Mitigation |
|---|---|---|
| V8 GC timing is not reproducible | Section 3.3 | Answered in M0 for a synthetic script (HGBA-3.1 Q-3): the fork's predictable platform gives identical hashes across runs, processes and operating systems; the worker-task mode is a mandatory recording field. The corpus-page probe in M1 confirms it with a DOM. Fallback, if that or a later V8 breaks it, is whole-VM determinism for the JS layer only |
| Long-tail layout (tables, fragmentation, writing modes) is new code with a moderate oracle | Section 5.6 | Write tables from the CSS 2 and CSS Table specifications, not from another engine's code (D-S10); three reference engines; report under EQ-1's moderate class |
| Project-built V8 | Section 7.1 | Build farm in M0; measure build cost; mirror archives to developers |
| Two GPU stacks in Stage 1 | Section 5.7 | Shared-texture interop tested on all platforms; P-2 (Vello) decides consolidation |
| Carrying a `rusty_v8` fork | Sections 3.3, 7.1 | Small, upstreamable commits (`cppgc` APIs, predictable platform, two submodules); S-4e update procedure inside the security window, with the security-backport rule as the fallback |
| Bindings codegen scale | Section 5.3 | Generated, not hand-written; idlharness tests first |
| Substrate indirection cost | Section 3.1 | DET-2; generic-parameter fallback |
| Integration cost of many upstreams | Section 7.7 | Owners, pins, lag metrics, fork triggers |

---

## 11. Decisions for the project owner (signed off 14 October 2026)

| ID | Question | Decision | Change from the proposed default, and why |
|---|---|---|---|
| D-S1 | Anti-fingerprinting model | **Hybrid, randomization by default**: seeded per-site, per-session noise on readback surfaces; reduced or normalized discrete surfaces; full uniformity (RFP-style) as a user opt-in | **Changed** from uniformity. HGBA's users are ordinary users avoiding ad-network tracking, not users needing anonymity from a global adversary, and RFP-style uniformity breaks fonts, canvas, WebGL, timezones and color profiles on real sites, which collides with P2. Seeded noise is deterministic given the seed, so DET-1 replays it. Discrete surfaces cannot be usefully randomized, so they are reduced instead. Effectiveness is measured by PRIV-2, not asserted |
| D-S2 | Shell UI technology | **The engine renders its own UI**, plus a minimal native recovery window | **Extended.** The HTML shell dogfoods the engine, but the browser must be recoverable when the engine is broken (Firefox's safe mode is the precedent). The recovery window (Section 6) is a Stage 1 requirement and never the default. **The shell is web content, not a privileged layer:** it runs in its own sandboxed renderer with zero ambient authority, and the capability endpoints the broker grants it are its only access to browser state. User theming is **CSS-only**, because script in the shell's renderer would inherit the shell's endpoints; scripted customization goes through W-1 components with their own declared capabilities |
| D-S3 | Extensions in Stage 1 | **None**; content blocking is built in; extensions arrive in Stage 2 via Wasmtime | Kept. Power users will find Stage 1 incomplete; that is accepted |
| D-S4 | Telemetry and crash reports | **Usage telemetry opt-in; crash reports opt-out** with first-run disclosure, a per-crash prompt and minidumps only | **Split.** Opt-in crash reporting yields an unrepresentative sample of the users who hit crashes. Minidump-only reports and a best-effort URL filter limit, but cannot guarantee to remove, page content a report can carry |
| D-S5 | First platforms | **Linux and macOS** first; Windows sandbox and process work by M5; Windows complete before M8 | Kept, with Windows pulled forward to M5 for sandboxing, process launch and IPC, which are the parts where Linux/macOS assumptions are expensive to undo. M0 already builds V8 and reruns the spike on Windows |
| D-S6 | License | **MIT** for all of Izcal's own code and documents, in both repositories (the `rusty_v8` fork was already MIT) | **Changed** on 17 October 2026 from the proposed MPL-2.0. The recorded MPL rationale (compatibility with Stylo and WebRender) does not require MPL: MPL is file-level, so an MIT project can depend on MPL crates. MIT matches the permissive new engines (Ladybird BSD-2, Blitz MIT OR Apache-2.0, Chromium BSD-3) and maximizes embedding and adoption. Consequences: (a) MPL-2.0 dependencies (Stylo, `selectors`, WebRender, Brave's `adblock`) stay MPL; binaries ship their license notices and point to their source; (b) no other browser's code is copied into Izcal at all, whatever its license (D-S10); (c) MIT has no explicit patent grant (accepted); (d) closed forks of Izcal are permitted |

---

## 12. Spec amendments required by the verified facts

**Applied on 14 October 2026** as HGBA-2.1.1 and HGBA-3.1.1, together with the accessibility-repair amendments from `reports/Feature design essays review.md` (repair classes R0–R4, repair overlay, A11Y-1 to A11Y-4, Q-10, and the P2 scope note).

| Spec | Section | Current text | Amendment |
|---|---|---|---|
| HGBA-3.1 | Section 5 reuse table, Section 11 | "Reuse Servo layout as Stage 1 baseline and oracle; replace per formatting context under gating" | Servo layout cannot run over a non-Servo DOM. Tables, fragmentation and writing modes are built new in the driver (porting Servo's table algorithm where useful; superseded on 17 October 2026, when D-S10 ruled out porting any browser's code); oracle is reference engines (Chrome, Firefox, Servo as a black box) plus WPT, which is the moderate oracle class |
| HGBA-3.1 | D1′ | "Reuse Servo's best components as the baseline and oracle" | Reused Servo components are Stylo, html5ever, WebRender and `ipc-channel` as a reference; Servo layout is reusable only as a black-box reference engine |
| HGBA-2.1 | Section 4.4, D3 | WebRender as rasterizer in a wgpu-based GPU service; one render graph | Stage 1 runs WebRender on OpenGL (ANGLE on Windows) plus wgpu for WebGPU, with shared-texture interop. One render graph becomes a P-2 outcome (Vello on wgpu) |
| HGBA-2.1 | Section 6 start condition (i), D2 | Spike rerun under `v8_enable_sandbox`; fork "if the patch is not merged by the Stage 1 midpoint" | No sandbox prebuilt exists and the crates.io source build is broken. The project builds V8 from a `rusty_v8` git checkout from M0, and the fork is the plan of record because determinism also needs it |
| HGBA-3.1 | Q-3, B1 risk | V8 determinism under `rusty_v8` unknown | Partly answered: flags exist (`--predictable`, `--single-threaded`, `--random-seed`); clocks and worker tasks need a forked predictable platform; M1 probe decides the rest. **Since answered in M0 for a synthetic script** (HGBA-3.1.4 Q-3); the M1 corpus-page probe remains |

---

## 13. Stage 1 scope completion

Sections 1–12 specify the engine. This section closes the gaps a shippable browser needs that those sections did not cover. Facts were checked on 14 October 2026 (`research_notes/Stage 1 design facts/scope_completion_factcheck.md`). Thresholds marked † are set by this document.

### 13.1 Owner decisions (D-S7 and D-S9 decided, D-S8 deferred)

**Excluded regardless of D-S7 (14 October 2026):** HGBA never ships an unlicensed or reimplemented DRM module (for example OpenWV) or extracted device credentials. Using a device identity the project is not licensed to hold is circumvention of a technical protection measure (17 U.S.C. §1201; InfoSoc Directive Article 6), breaches streaming services' terms, fails whenever the identity is revoked, and would end any prospect of a Widevine or PlayReady agreement. The only DRM paths are licensed modules: Widevine with Google's approval, or the PlayReady module that ships with Windows, where each streaming service's allow-list is the gate. Changing which license server a service uses does not change the browser's position, because the service chooses the license server.

**Clear Key is supported in Stage 1 regardless of D-S7.** It is the EME key system every implementation must support, it needs no licence, and it serves sites that run their own license servers for content they control. A project-generated key pair or certificate authority is lawful but cannot obtain licenses from Widevine or PlayReady servers, which accept only identities that chain to the DRM vendor's root; presenting a self-made identity as a vendor-provisioned one would be circumvention.

**Status (17 October 2026):** D-S7 is decided (13.1a), and D-S9 (name) is decided. D-S8 (funding) is deferred; planning and M0 engineering continue, but **Stage 1 (M1 onward) cannot be staffed until D-S8 is decided**, so M0's exit criterion "team for M1–M2 staffed" is blocked on it.

| ID | Decision | Options | Recommendation |
|---|---|---|---|
| D-S7 | **DRM** | Decided 14 October 2026: **a layered strategy built only on licensed components**, so no single company's refusal can remove playback on Windows or macOS (Section 13.1a) | — |
| D-S9 | **Project name** | Decided 17 October 2026: **Izcal**. It is the product name (UA token, client-hint brand, shell, update service, fork tags `v<upstream>-izcal.<n>`); HGBA remains the architecture and spec name, and crates keep the `hg-` prefix | Trademark clearance search before the first public binary |
| D-S10 | **Original implementation** | Decided 17 October 2026: Izcal is a new engine, not a derivative. **No code from another browser or browser engine** (Servo, Chromium and Blink, WebKit, Gecko and Firefox, Ladybird, and their forks) is copied, ported or translated into Izcal, by people or by AI tools, whatever its license. The specifications are the source; other engines are black-box references only (differential testing, WPT). Libraries used as dependencies are not copying: V8 (through the `rusty_v8` fork, whose changes are written against V8's public interfaces), Stylo, `selectors`, WebRender, html5ever, Taffy, Parley, wgpu, `adblock`, neqo and rustls. Enforced by the `provenance` check and the review (CONTRIBUTING.md) | — |
| D-S8 | **Funding and governance** | Donations and sponsorship (Ladybird); search-engine revenue share (Firefox); a paid product such as an enterprise or agent-runtime edition; grants; a corporate sponsor | **Deferred by the owner (14 October 2026); needed before M1 can be staffed.** Section 8.3's 18–23 engineers for 24–36 months† is roughly 40–70 engineer-years. Every option conflicts with something: search deals pull against privacy defaults, ads against content blocking, and a single corporate sponsor against Servo's lesson (activity fell about 95% within two years of losing Mozilla). The agent-runtime positioning (HGBA-3.1 Section 7) is the only revenue idea in the documents, and it is a bet on a market that has not formed |

### 13.1a DRM strategy (D-S7)

**Goal:** a user can play content they pay for, and no single licensor can take that away. **Constraint:** licensed components only (the exclusion above).

1. **Protected media mode (Windows and macOS, primary).** When a page actually starts encrypted playback with a key system HGBA's engine lacks (an `encrypted` event on a media element, or `MediaKeys` created and a license request generated), the tab switches automatically into the protected-media host. That host runs the page in the operating system's licensed engine: **WebView2** on Windows, with Widevine and PlayReady under Microsoft's license; **WKWebView** on macOS, with FairPlay under Apple's license. The precedent is Edge's IE mode. Rules:
   - A bare `requestMediaKeySystemAccess` capability probe never triggers a switch, because fingerprinting scripts call it routinely. Cross-origin iframes cannot switch the top-level page.
   - The tab is labelled ("Playing protected video using Windows' media engine" or "macOS's media engine"), offers one-click switch-back, and remembers the site per container. A per-site "never switch" override exists.
   - Storage is per (site, container): a WebView2 user-data folder or a `WKWebsiteDataStore` per identifier.
   - HGBA protections apply where the host allows: content blocking through request interception or `WKContentRuleList`, and permissions through host callbacks. Anti-fingerprinting, the semantic plane and agent features do not apply, and the label says so.
   - The user agent is the host engine's own (Edge or WebKit), which is truthful because that engine renders the page; an application-name suffix may identify HGBA. Agent tabs (B9) never use protected media mode.
   - Spikes DRM-1 (Windows) and DRM-2 (macOS) in `m0/drm-spike/` verify playback, resolution and container isolation per service before this is built into the shell.
   - **Status (14 October 2026):** both spikes are built, and their probe pages were run. The probe page queries which DRM systems each system web engine supports, without starting playback. **WebView2** (Windows 11) supports Widevine at the software security level, PlayReady including the hardware level, and Clear Key. **WKWebView** (macOS 15.6) supports FairPlay and Clear Key. Per-container storage and content blocking work on both. A capability-only probe correctly did not trigger a switch. **The signed-in playback tests are deferred by the owner**, so per-service playback and resolution remain unverified.
2. **Native PlayReady in HGBA's engine (Windows, M7).** EME against the PlayReady module Windows ships, through Media Foundation, as Firefox 132 does. Where a streaming service allow-lists HGBA, playback stays in HGBA's engine with every protection; protected media mode remains the fallback.
3. **Licensing track (from M0, no cost):** a Widevine license request to Google; a castlabs third-party-licensee quote (the only Linux route); a PlayReady terms enquiry to Microsoft; allow-listing requests to streaming services' partner teams, Netflix first.
4. **Linux:** no route avoids Google today. Linux ships Clear Key only, and onboarding says so plainly, until the licensing track succeeds.
5. **Regulatory track (deferred by the owner).** Recorded for later: an EU DMA Article 6(7) interoperability complaint (Android's DRM as an operating-system feature Google must offer on equal terms); a submission to the UK CMA under Google's mobile-platform Strategic Market Status, which covers browser engines; a coalition of affected browsers. No complaint about Widevine access has been filed with any regulator (research, 14 October 2026), so these are untested.

**Why it is permanent:** the Windows and macOS paths depend on components their owners already license to every user of their operating system, not on an agreement HGBA must win. The licensing track improves quality (native playback, Linux), and the regulatory track addresses the root cause; neither is on the critical path for Windows and macOS users.

### 13.2 Decided

| Area | Decision | Stage |
|---|---|---|
| **Codecs** | Royalty-free decoders shipped by the project: AV1 (dav1d), VP8 and VP9 (libvpx in the library sandbox), Opus, Vorbis and FLAC. **H.264, AAC and HEVC are decoded only through OS decoders** (Media Foundation, VideoToolbox, VA-API or GStreamer), because the 2010 MPEG LA waiver covers free internet *content*, not decoders: a browser shipping its own H.264 decoder owes royalties (100,000 units a year free, then $0.20 and $0.10 per unit, capped at $9.75M a year). WebRTC uses Cisco's OpenH264 binary, as Firefox does. AV1 stays in, even though Sisvel runs an AV1/VP9 pool and Access Advance announced an exploratory AV1/AV2 pool on 25 August 2026, because Chrome, Firefox and Edge ship AV1 and Safari decodes it in hardware. Dropping it would break YouTube and Netflix delivery without making VP9 any safer, since the same pools cover VP9. Dolby v. Snap (Delaware, 1:26-cv-00317, filed 23 March 2026) concerns HEVC; its AV1 claims are in Brazil, and all are pending. **A legal review of codec exposure is required before M7.** | 1 |
| **User agent** | A Chrome-compatible UA string with the project's own token appended, matching what every new engine does. Ladybird reported "degraded UIs, 'your browser is not supported' pages, network throttling, and outright HTTP 403 errors" before doing the same (January 2026); Servo presents as Firefox. Client-hint brands list the project's own brand, not "Chromium", because it is not Chromium. **Agent tabs add a distinct agent token** (HGBA-3.1 non-goal: agents are never disguised as human browsing). New target UA-1 runs the corpus with an honest UA every release and publishes the share of sites that break, so the compromise can end when the web stops discriminating | 1 |
| **iOS** | **Out of scope for Stages 1–2.** No major browser has shipped a non-WebKit engine under BrowserEngineKit in the EU, and Apple requires a separate EU-only app. Microsoft's Blink prototype for iOS (28.6% faster than Safari on one device) is still a prototype. Revisit if DMA enforcement changes | — |
| **Text and input** | Parley already provides font fallback (Fontique), complex-script shaping (HarfRust), line breaking and bidi (ICU4X), and IME composition. **All scripts render in Stage 1.** Platform IME integration (TSF on Windows, `NSTextInputClient` on macOS, the Wayland text-input protocol plus IBus or Fcitx on Linux) and composition events are **M6**, because they are web platform behaviour that WPT tests. The shell UI is English-only in Stage 1, with strings in Fluent format so translation needs no code changes | 1 |
| **TLS, HTTP and DNS** | `rustls` for TLS 1.3, with hybrid post-quantum key exchange (HGBA-2.0 Section 4.7). HTTP/3 uses **Mozilla's `neqo`**, which ships in Firefox, over `quinn` plus `h3`, because the `h3` crate calls itself "very experimental". DNS-over-HTTPS uses `hickory-dns`, with resolver policy in the network service | 1 |
| **Certificates** | `rustls-platform-verifier` is the **primary** verifier, so enterprise and user-installed roots work; Mozilla's root list (`webpki-roots`) is the fallback where the platform has none. **Certificate Transparency is enforced in Stage 1**: Chrome and Safari have required it since 2018 and Firefox enforces it on desktop from version 135. **Revocation uses a CRLite-style compressed filter**, since Let's Encrypt shut down OCSP on 6 August 2025 and the CA/B Forum made OCSP optional (SC-063). OCSP stapling is honoured when present but is not the mechanism. Stage 1 can start with CRLs pushed by the update service; CRLite-style filters follow in Stage 2 | 1 (CRLite filters 2) |
| **DevTools** | Console, Sources (debugger) and the profilers come from V8's inspector protocol (Runtime, Debugger, Profiler and HeapProfiler domains). **Elements and Network need the DOM, CSS, Network, Page and Overlay protocol domains, which the project implements**; that is engine work on the scale of the WebDriver server, and it shares code with it. Frontend: Chrome's `devtools-frontend` (BSD-3-Clause). The remote-debugging port is off by default | 1 (Console, Sources, Elements, Network), 2 (Performance, Application, Memory panels) |
| **Service workers** | Stage 1, since many sites need them. **Push** needs a push service: the project hosts its own **autopush** server (no permission was found to use Mozilla's hosted service). That is an operations commitment, so push and background sync are Stage 2 | SW 1; push 2 |
| **Permissions** | A permission store with per-site and per-container grants, prompts gated on user activation, and auto-revocation of unused grants. Camera, microphone, location, clipboard, notifications and File System Access. The `<geolocation>` element (Chrome 144) is adopted once it is standardized; the generic `<permission>` element ended as an origin trial (Chrome 126–143). The detailed design is written before M6 | 1 |
| **Storage** | SQLite in WAL mode behind IndexedDB, the Cache API and localStorage, in the storage service. Quota per (site, container), LRU eviction under pressure, `navigator.storage.estimate()` and `persist()`. The design is written before M5 | 1 |
| **Everyday features** | **Stage 1:** find in page, zoom, fullscreen, clipboard, drag and drop, a downloads manager with file-type and reputation checks, **import from Chrome and Firefox** (bookmarks, history and passwords), and a **basic password manager** stored in the OS keychain, because imported passwords need somewhere to live. **Stage 2:** autofill beyond passwords, printing and print-to-PDF, spellcheck through OS services, picture-in-picture | 1 / 2 |
| **Threat model** | One document per process (broker, renderer, GPU, network, storage, utility, shell renderer, agent host): its assets, what an attacker controls, the capabilities it holds and the mitigations. Written before M6 | 1 |
| **Security response** | A published vulnerability-disclosure policy, security advisories, an embargo process for V8 and upstream fixes (D2's one-Chrome-cycle window), and a bug bounty once funding allows | 1 (policy), 2 (bounty) |
| **Privacy and legal** | A privacy policy and GDPR and CCPA handling for crash reports and opt-in telemetry, written before M7. **Export controls:** encryption software falls under EAR category 5D002. Publicly available open source qualifies for the published-source exclusion (with notification where still required), but a release that bundles a closed DRM module is typically classified as mass-market encryption, so legal review comes before the first public binary. The project name is **Izcal** (D-S9, 17 October 2026). A trademark clearance search is still needed before the first public binary, because the name appears in the UA token, the shell and the update service | 1 |
| **Updates** | A TUF-based update client: signed metadata, rollback and freeze protection, key rotation. The project operates the update service, which also distributes revocation data | 1 |
| **Profiles** | Versioned profile format with forward-only migrations that are kept for at least a year, as Chromium does. AG-0's upgrade scenario exercises them | 1 |
| **Packaging** | Signed and notarized `.dmg` on macOS; a signed installer on Windows; `.deb`, `.rpm` and a tarball on Linux in **Stage 1**, because Linux is a first development platform (D-S5) and its users are the likeliest early adopters. Flatpak is Stage 2 through zypak-style sandbox redirection, as the Chromium Flatpak does; how strong that sandbox is remains disputed, so the Flatpak build documents its reduced isolation | 1 / 2 |
| **Shell** | The shell passes the same accessibility checks as a web page (it is one). Onboarding explains AI-off, containers, import and the DRM position (D-S7). Settings are a Stage 1 deliverable | 1 |
| **Embedding API** | Stage 2. Stage 1 fixes the capability endpoints and process model it will use | 2 |

### 13.3 Resource targets (Stage 1, measured from M2 in the performance lab)

| ID | Target | Threshold† | Method |
|---|---|---|---|
| RES-1 | Cold start to interactive | ≤ 1.5× stable Chrome on each reference device | Same machine, same corpus start page, median of 20 runs after a reboot |
| RES-2 | Warm start | ≤ 1.5× stable Chrome | Median of 20 runs |
| RES-3 | Memory | Total browser memory (all processes, proportional set size) for a fixed 10-tab corpus scenario ≤ 1.2× stable Chrome. Per-tab figures are not used, because process sharing makes them ill-defined | Median of 5 runs, including site-isolation overhead (S-3) |
| RES-4 | Installer size | ≤ 120 MB† per platform | Measured on each release |
| RES-5 | Energy | Per-corpus-run energy published, as HGBA-2.0 X-4, from Stage 1 rather than Stage 3 | Platform power counters |

Targets are relative to Chrome on the same machine, because no reliable absolute figures were found: the only published comparison is one commercial blog post, and a widely repeated "780 MB per tab" figure has no source.

### 13.4 New compatibility target

| ID | Target | Threshold / method | If missed |
|---|---|---|---|
| UA-1 | The honest-UA gap is known | Each release, the corpus runs with an honest UA (project token only) and with the shipping Chrome-compatible UA. The share of pages that break, block or degrade under the honest UA is published | No threshold. When the gap falls below 2%†, the decision log revisits the Chrome-compatible UA |
