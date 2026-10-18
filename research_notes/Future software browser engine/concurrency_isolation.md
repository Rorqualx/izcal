# Concurrency and Isolation Models for a New Browser Engine (as of October 2026)

Status labels: **[shipped]** in production use; **[prototype]** preview, experimental or early release; **[research]** academic or proposal stage.
Note on provenance: items marked "(fetched/searched Oct 2026)" were checked during this session. Items marked "(not re-verified Oct 2026)" are long-standing, widely documented facts cited to their canonical primary URL but not re-fetched this session. The report writer should treat those as lower confidence.

---

## 1. Structured concurrency, async Rust and work-stealing: fit for engine task scheduling

### Takeaway
Structured concurrency, where task lifetimes are lexically scoped and cancellation and errors propagate to the parent, is now mainstream in Swift, Kotlin, Python (Trio, then asyncio) and nearly finished in Java. Java's API is still in preview in JDK 26 (sixth preview).

Async Rust is usable, but dynamic dispatch of async trait methods is still not native in 2026: the Rust project only aims to settle the design in 2026. An engine should lean on concrete executors plus Rayon-style work-stealing for data-parallel phases such as layout, style and raster.

### Cited Findings
**Java (Loom)**
- **Structured concurrency is still preview-only.** JEP 525, "Structured Concurrency (Sixth Preview)", targets JDK 26 and is a preview API, not final. It was incubated in JDK 19 and 20 and has been in preview since JDK 21. **[prototype]** (fetched/searched Oct 2026) — [JEP 525](https://openjdk.org/jeps/525); [inside.java, Nov 2025](https://inside.java/2025/11/24/jep525-target-jdk26/)
- **JDK 26 changes are small.** They are:
  - `Joiner.onTimeout()`
  - `allSuccessfulOrThrow()` now returns a list
  - `anySuccessfulResultOrThrow` renamed to `anySuccessfulOrThrow`
  - `open(...)` now takes a `UnaryOperator`

  InfoQ calls these "minor but useful tweaks". The core model is unchanged. — [InfoQ, Jan 2026](https://www.infoq.com/news/2026/01/timeout-joiner-refinements); [InfoQ Java 26](https://infoq.com/news/2026/02/java-26-so-far)

**Swift**
- **Swift 6.2 (2025) makes concurrency more "approachable".** It adds:
  - an option for default `@MainActor` isolation per module
  - `nonisolated(nonsending)` by default, so nonisolated async functions run on the caller's actor instead of hopping to the global executor
  - `@concurrent`, an explicit opt-in to run on the background pool

  **[shipped]** (searched Oct 2026; secondary sources) — [InfoQ, Aug 2025](https://www.infoq.com/news/2025/08/swift62-approachable-concurrency); [Swift Forums asymmetry thread](https://forums.swift.org/t/am-i-doing-approachable-concurrency-right-this-asymmetry-feels-weird/83085)
- **Apple partly backed away from "everything concurrent by default".** With both settings on, most code runs on the main actor unless it explicitly opts out. This is a notable signal: unrestricted actor hopping caused too many data-race diagnostics ("sending X risks causing data races"). — [hmlong substack](https://hmlong.substack.com/p/swift-62-approachable-concurrency) (secondary; verify against swift.org)
- **Swift's structured concurrency base layer** is task groups and `async let` (SE-0304), with actors in SE-0306. (not re-verified Oct 2026) — [SE-0304](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md); [SE-0306](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0306-actors.md)

**Trio, Python and Kotlin**
- **Trio nurseries** are the canonical structured-concurrency design. They come from N. J. Smith's 2018 essay "Notes on structured concurrency, or: Go statement considered harmful". Python adopted the pattern in asyncio as `TaskGroup` (Python 3.11). **[shipped]** (not re-verified Oct 2026) — [vorpus.org essay, 2018](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/); [Python asyncio TaskGroup docs](https://docs.python.org/3/library/asyncio-task.html#task-groups)
- **Kotlin coroutines** enforce structured concurrency through `CoroutineScope`: a parent waits for its children, and cancellation propagates down the tree. **[shipped]** (not re-verified Oct 2026) — [Kotlin docs](https://kotlinlang.org/docs/coroutines-basics.html)

**Async Rust**
- **Async closures** (`AsyncFn`, `AsyncFnMut`, `AsyncFnOnce`) were stabilized in Rust 1.85 on 20 Feb 2025. **[shipped]** — [releases.rs 1.85.0](https://releases.rs/docs/1.85.0/); [InfoWorld](https://www.infoworld.com/article/3812600/async-closure-support-is-stable-for-rust-1-85.html)
- **Async fn in traits is stable but not dyn-compatible.** It has been stable since Rust 1.75 (Dec 2023), but traits with async methods cannot be used as trait objects. The 2026–27 project goal "Native async fn dynamic dispatch in traits" (accepted) only aims to *settle a design* in 2026, not to implement or stabilize it. Return-type notation (RTN) and the next-generation trait solver are prerequisites. **[prototype/design]** — [Rust goals 2026: AFIDT box](https://rust-lang.github.io/goals/2026/afidt-box.html); [Roadmap "Just add async"](https://rust-lang.github.io/goals/2026/roadmap-just-add-async.html)
- **Interim workarounds:**
  - the `dynosaur` proc-macro crate
  - `async-trait`, which boxes every call as `Pin<Box<dyn Future + Send>>`, one allocation per call

  — [Rust Project Goals Feb 2025 update](https://blog.rust-lang.org/2025/03/03/Project-Goals-Feb-Update/)
- **Tokio's multi-threaded runtime** uses a work-stealing scheduler, redesigned in 2019 with per-worker run queues and a LIFO slot for message-passing latency. Tokio is the de facto default executor. **[shipped]** (not re-verified Oct 2026) — [Tokio blog, "Making the Tokio scheduler 10x faster", Oct 2019](https://tokio.rs/blog/2019-10-scheduler)

**Rayon and Servo**
- **Rayon** provides Cilk-style work-stealing data parallelism (`join`, `par_iter`). **[shipped]** (not re-verified Oct 2026) — [rayon-rs/rayon](https://github.com/rayon-rs/rayon)
- **Servo still parallelizes layout with worker threads.** Even after moving layout onto the script thread in Feb 2024, the post says layout "can still spawn worker threads to parallelise layout work". **[shipped in Servo]** (fetched Oct 2026) — [servo.org blog, 28 Feb 2024](https://servo.org/blog/2024/02/28/gamepads-font-fallback-space-jam/)

**Chromium**
- **Chromium's scheduling model is sequences, not threads.** Code posts tasks to `SequencedTaskRunner`s, which run serially on a thread pool, rather than owning threads. The docs recommend "prefer sequences to physical threads". **[shipped]** (not re-verified Oct 2026) — [Chromium docs: Threading and Tasks](https://chromium.googlesource.com/chromium/src/+/main/docs/threading_and_tasks.md)

### Inferences
- **Two schedulers fit a Rust engine better than one.**
  - A structured, scoped task model for I/O and the event loop: Tokio or a custom single-threaded-per-agent executor, with sequences in the Chromium style.
  - Rayon-style fork/join for CPU-bound pipeline phases: style, layout, raster.

  Mixing both inside one global async runtime risks blocking executor threads.
- **Native dyn dispatch for async traits will not land before 2027 at the earliest.** Engine plugin and embedder interfaces should avoid `dyn` async traits, or accept boxing.
- **The web's event loop is already serial per agent.** Actor-style "one sequence per agent cluster" maps naturally onto the HTML event-loop model. Structured concurrency helps most for cancellation of fetches, navigations and workers.

### Gaps
- No primary 2026 source found that compares Tokio's executor share against smol, glommio, monoio or embassy.
- Kotlin's 2026 status was not re-verified.

---

## 2. Actor models in production: costs and benefits

### Takeaway
Actor and message-passing systems buy fault isolation and simpler reasoning: Erlang supervision, Orleans virtual actors, Chromium sequences and Mojo, Servo's constellation. They cost copying and latency at each boundary, and they make synchronous queries awkward.

Browsers already use a hybrid. Coarse actors (processes, the constellation, Mojo interfaces) handle isolation, and shared-memory threads handle the hot rendering pipeline.

### Cited Findings
- **Erlang/BEAM.** Isolated lightweight processes, message passing and supervision trees ("let it crash") were designed for telecom fault tolerance. **[shipped]** (not re-verified Oct 2026) — [Joe Armstrong PhD thesis, 2003](https://erlang.org/download/armstrong_thesis_2003.pdf)
- **Akka and Pekko.** Lightbend relicensed Akka from Apache 2.0 to the Business Source License in 2022. The community fork, Apache Pekko, continued under Apache 2.0. **[shipped]** (not re-verified Oct 2026) — [Lightbend blog](https://www.lightbend.com/blog/why-we-are-changing-the-license-for-akka); [Apache Pekko](https://pekko.apache.org/)
- **Microsoft Orleans.** "Virtual actors" (grains) are always addressable and activated on demand, and the runtime handles placement and failure. **[shipped]** (not re-verified Oct 2026) — [Orleans overview, Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/orleans/overview)
- **Swift actors** serialize access to mutable state, with compile-time data-race checking in Swift 6. Swift 6.2 moved defaults back toward main-actor isolation to reduce friction (see Q1). **[shipped]** — [InfoQ, Aug 2025](https://www.infoq.com/news/2025/08/swift62-approachable-concurrency)
- **Servo constellation.** The constellation is a central coordinator that owns pipelines (script plus layout per document) and routes messages between them and the embedder or compositor. It was historically designed for parallel, message-passing components. (not re-verified Oct 2026) — [Servo architecture talk notes (wiki)](https://redirect.github.com/servo/servo/wiki/Transcription---Notes-from-Servo-Architecture-talk-in-Suwon)
- **Servo is revisiting the script/layout split.** Its 2025 roadmap reportedly planned to split script and layout threads again, "without any global blocking". Phoronix's coverage is secondary, and no primary confirmation was found. — [Phoronix, Servo roadmap 2025](https://www.phoronix.com/news/Servo-Roadmap-2025)
- **Chromium Mojo IPC** works with message pipes and typed interfaces (`Remote`/`Receiver`) generated from `.mojom` IDL. Interfaces bind to a sequence. Sync calls are supported (`[Sync]`) but discouraged in the bindings docs because they block and can cause re-entrancy and deadlock. **[shipped]** (not re-verified Oct 2026) — [Mojo README](https://chromium.googlesource.com/chromium/src/+/main/mojo/README.md); [Mojo C++ bindings docs](https://chromium.googlesource.com/chromium/src/+/main/mojo/public/cpp/bindings/README.md)
- **Chromium process roles.** These are:
  - the browser process, which handles UI and routes input
  - one render process per site+tab (strictly separated on desktop, "when possible" on mobile)
  - the Viz process, which handles GPU compositing and draw, kept separate for GPU-driver fault isolation and security

  **[shipped]** (fetched Oct 2026) — [RenderingNG architecture, developer.chrome.com](https://developer.chrome.com/docs/chromium/renderingng-architecture)

### Inferences
- **The production evidence favours actors at coarse granularity.** Use them at process, document and agent-cluster level, plus GPU or network service boundaries. Fine-grained actors inside a layout or style pass would pay message costs on the hottest paths.
- **Mojo is the closest proven template for a typed, capability-like actor boundary inside a browser.** A Rust engine could mirror it with typed channels, sequence-affine endpoints and the sync-call discouragement policy.

### Gaps
- No quantitative per-message cost figures found for Mojo or the Servo constellation.

---

## 3. Where message boundaries hurt in browsers

### Takeaway
Web APIs force synchronous round trips between script and layout or style:
- `getBoundingClientRect`, `offsetWidth`
- `getComputedStyle`
- sync XHR

Servo abandoned its separate layout thread in Feb 2024, saying the web platform "almost always requires layout to run synchronously with script".

Chromium places its boundaries where the web *doesn't* demand synchrony: compositor-thread scrolling and animation, Viz, and cross-site OOPIFs. It pays roughly 9–13% memory (2019 data) for site isolation.

### Cited Findings
**Servo's layout move**
- As of 24 Feb 2024, Servo runs layout in the script thread instead of a dedicated layout thread. The stated reasons:
  - "the web platform almost always requires layout to run synchronously with script"
  - the change makes code simpler and more reliable
  - no expected performance loss
  - parallel layout workers are kept

  **[shipped in Servo]** (fetched Oct 2026) — [servo.org, 28 Feb 2024](https://servo.org/blog/2024/02/28/gamepads-font-fallback-space-jam/)

**Forced synchronous layout**
- All APIs that synchronously return layout metrics (`getBoundingClientRect`, `offset*`, `scroll*` and similar) force style and layout flush, or reflow. — [Paul Irish gist "What forces layout/reflow"](https://gist.github.com/paulirish/5d52fb081b3570c81e3a) (canonical list, not re-verified Oct 2026); similar compilation in [gist.github.com/geobde](https://gist.github.com/geobde/b7075d5abd0d32cce0509f8054ce58f6)
- Blink engineers have repeatedly patched redundant forced layouts in `getBoundingClientRect` and similar calls. Examples:
  - guarding against forced layouts for elements not in the active document (~2016)
  - cleaning compositing inputs before location APIs, which was later reverted due to failures

  — [Chromium CR 1839803007](https://codereview.chromium.org/1839803007); [CR 2647533002](https://codereview.chromium.org/2647533002); [revert CR 2667003003](https://codereview.chromium.org/2667003003)

**Chromium RenderingNG threading**
- **Main thread vs. compositor thread.** The main thread runs script, the event loop, the document lifecycle (style, layout, paint), hit testing and event dispatch. The compositor thread handles input, scrolling, animations, layerization and coordination of raster and decode.
- **Quote:** "Separating the main and compositor threads is critically important for performance isolation."
- **Input routing.** Input reaches the main thread only when needed, for example when listeners might call `preventDefault`.
- **Viz threads.** Viz has a GPU main thread and a separate display compositor thread, which stays responsive when drivers are slow.

**[shipped]** (fetched Oct 2026) — [RenderingNG architecture](https://developer.chrome.com/docs/chromium/renderingng-architecture)

**Site isolation and OOPIFs**
- **Each renderer is locked to one site, and cross-site iframes run out of process.** Same-site frames share a process. Origins can opt into finer isolation (origin-keyed agent clusters). **[shipped]** — [RenderingNG architecture](https://developer.chrome.com/docs/chromium/renderingng-architecture); [Chromium OOPIF design doc](https://www.chromium.org/developers/design-documents/oop-iframes/) (not re-verified Oct 2026)
- **Site isolation costs (2019 data).** Total memory overhead was 9–13% in practice, page-load-time change was 2.25%, and input latency changed little. Process consolidation optimizations were needed to hold overhead down. Press quotes from Reis cite "10–13%" memory overhead in real workloads. (dated 2019) — [Reis, Moshchuk, Oskov, USENIX Security 2019 (PDF)](https://www.usenix.net/system/files/sec19-reis.pdf); [PortSwigger summary](https://portswigger.net/daily-swig/google-charts-progress-in-developing-site-isolation-browser-technology); [eWeek](https://www.eweek.com/security/google-makes-site-isolation-the-default-setting-in-chrome-browser/)
- **Spectre forced process isolation.** V8 concluded in 2019 that in-process software mitigations cannot fully defend against Spectre, and that process isolation (site isolation) is the robust mitigation. (not re-verified Oct 2026) — [v8.dev "A year with Spectre: a V8 perspective", 2019](https://v8.dev/blog/spectre)

### Inferences
- **Script, DOM, style and layout for one event loop should share an address space and thread,** or at least be synchronously callable. Every successful engine (Blink, Gecko, WebKit, and now Servo) does this. Message boundaries belong at:
  - main → compositor (property-tree commits)
  - renderer → GPU/Viz
  - renderer → browser (navigation, storage, network)
  - cross-site frame → frame (OOPIF; already async by spec via `postMessage`)
- **Cross-origin frames are the best-fitting place for message boundaries.** The web platform already exposes them only asynchronously (`postMessage`, `WindowProxy` restrictions), so the site-isolation boundary costs memory but not semantics.
- **Same-origin synchronous access between frames (`frame.contentDocument`) is the hard constraint.** Same-site frames must stay in the same process.

### Gaps
- No 2025–2026 updated memory-overhead figure for site isolation found. Also not covered: Android partial site isolation numbers, and the Firefox Fission memory cost.
- Primary documentation for Servo's 2025–26 plan to re-split script and layout was not found (Phoronix only).

---

## 4. Capability systems

### Takeaway
Capability OSes are real but niche:
- Fuchsia ships on Nest Hub devices, but its team was cut in 2023 and its speaker rollout dropped.
- seL4 is deployed in defense and aerospace (DARPA HACMS, CASE, PROVERS via Collins).
- Capsicum ships in FreeBSD.

Chromium's Mojo handles behave like capabilities. Chromium on Fuchsia exposes the web engine as capabilities (FIDL protocols) via WebEngine and runners, but it is the same Chromium multi-process engine inside. No modern full browser is built on capability-OS principles. DarpaBrowser (2002) remains the main research precedent.

### Cited Findings
**Fuchsia**
- **Staffing and speakers.** 2023 Google layoffs cut 16% to over 20% of the ~400-person Fuchsia team. Fuchsia smart-speaker work was marked "unsupported", and speakers stay on CastOS. **(dated 2023)** — [9to5Google](https://9to5google.com/?p=549379); [Chrome Unboxed](https://chromeunboxed.com/fuchsia-will-not-be-coming-to-googles-smart-speakers-as-the-company-halts-its-rollout-efforts/)
- **Nest Hub.** Fuchsia shipped on Nest Hub (1st and 2nd gen) and Nest Hub Max. Fuchsia version 14 was rolling out to the Nest Hub preview program in 2023. **[shipped, dated 2023]** — [OSnews](https://www.osnews.com/story/137974/fuchsia-version-14-rolling-out-to-nest-hub-preview-program/); [PhoneArena](https://phonearena.com/news/cast-out-fuchsia-in-on-second-gen-next-hub_id147301)
- No 2026 primary source on Fuchsia product status was found.
- **Component framework runners.** A runner is a capability: components declare and route runner capabilities, and component_manager hands startup to runners over the `ComponentRunner` protocol. — [fuchsia.dev runner capabilities](https://fuchsia.dev/docs/concepts/components/v2/capabilities/runner)
- **Chromium on Fuchsia.** It implements the `fuchsia.web` FIDL API as WebEngine. Runners (a web runner and CastRunner) let Fuchsia open http(s) URLs. The WebEngine manifest exposes `fuchsia.web.Context`, `fuchsia.web.Debug` and `fuchsia.web.FrameHost` as capabilities. The current README describes:
  - no general-purpose browser UI
  - focus on stability, security and privacy for smart displays
  - few new Fuchsia-specific features planned

  **[shipped, maintenance mode]** — [Chromium fuchsia_web README (HEAD)](https://chromium.googlesource.com/codesearch/chromium/src/+/HEAD/fuchsia_web/README.md); [fuchsia_web at M109](https://chromium.googlesource.com/chromium/src/+/109.0.5414.119/fuchsia_web/); [WebEngine manifest shard](https://gitcode.com/openharmony-tpc/chromium_src/blob/master/fuchsia_web/webengine/web_instance-common.shard.cml)

**seL4**
- **DARPA programs.** Under DARPA HACMS, Collins Aerospace demonstrated an seL4-based cyber retrofit of autonomous military vehicles. Collins went on to lead DARPA CASE, and now leads DARPA PROVERS, which targets seL4 as an implementation platform. **[shipped in defense prototypes/fielded systems]** — [seL4 news 2025](https://sel4.systems/news/2025.html)
- **seL4 Summit 2025 (Prague, 3–5 Sep 2025)** included "Integration of seL4 in a Flight Vehicle Mission System" (Collins) and a panel on the business case for a verified kernel. — [seL4 Summit 2025 program](https://www.sel4.systems/Summit/2025/program.html)
- **Funded deployments (dated ~2024).** These include TII (UAE, since Dec 2021) and NCSC (UK, since Jan 2022). — [UNSW CS9242 lecture](https://cgi.cse.unsw.edu.au/~cs9242/24/lectures/07b-local.pdf)

**Capsicum and DarpaBrowser**
- **Capsicum.** It provides capability mode plus file-descriptor rights for FreeBSD (Watson et al., USENIX Security 2010). The paper included a Capsicum-sandboxed Chromium prototype. **[shipped in FreeBSD]** (not re-verified Oct 2026) — [Capsicum paper, USENIX Security 2010](https://www.usenix.org/legacy/event/sec10/tech/full_papers/Watson.pdf)
- **DarpaBrowser (Combex, 2002).** This was a capability-secure browser written in E/CapDesk that confined a renderer by object capabilities. David Wagner and Dean Tribble wrote a security review of it. **[research, 2002]** (not re-verified Oct 2026) — [Wagner & Tribble, "A Security Analysis of the Combex DarpaBrowser Architecture"](http://www.combex.com/papers/darpa-review/security-review.pdf)

**Mojo**
- Mojo message pipes and interface endpoints are unforgeable handles, passed by message. Holding a `Remote<T>` is the authority to call T, which makes Mojo a capability-like interface system. (not re-verified Oct 2026) — [Mojo README](https://chromium.googlesource.com/chromium/src/+/main/mojo/README.md)

### Inferences
- **The fit for an engine is internal capability discipline,** not a capability OS. That means typed, unforgeable channel endpoints granted per frame, worker or service (Mojo or FIDL style), with no ambient authority in renderer processes. This complements OS sandboxes (seccomp-bpf, Capsicum, macOS Seatbelt, Windows AppContainer) rather than replacing them.
- **Fuchsia's Chromium port is evidence about one layer only.** A capability-routed outer shell can wrap a conventional multi-process engine without changing web semantics. The engine internals were not capability-redesigned.

### Gaps
- Fuchsia device and product status in 2026 is not confirmed by any primary source.
- Apple's reported use of seL4 is unconfirmed.
- WASI capability specifics are covered under Q5.

---

## 5. Wasm Component Model as an internal engine ABI

### Takeaway
The Component Model is shared-nothing with a copying Canonical ABI. WASI 0.3, ratified 11 June 2026, made async native: `async func`, `stream<T>` and `future<T>` are in the ABI, with Wasmtime 46 enabling it by default.

Zero-copy forwarding exists only in the *planned* lazy-lowering ABI on the road to Component Model 1.0. Shared-everything threads are still an early-phase proposal.

Production users include Zed extensions (Wasmtime plus WIT), Fastly and Microsoft Hyperlight Wasm. All use components as *plugin or tenant* boundaries, not as hot internal engine ABIs.

### Cited Findings
**WASI 0.3 and component async**
- **Ratification.** The WASI Subgroup ratified 0.3.0 on 11 June 2026. Its features:
  - `async func` imports and exports, replacing WASI 0.2's `start-foo`/`finish-foo`/`subscribe` pattern
  - `stream<T>` and `future<T>` as owned handle types that transfer ownership across boundaries and cannot be borrowed
  - a single runtime-managed event loop

  **[shipped spec]** (fetched Oct 2026) — [Bytecode Alliance "WASI 0.3"](https://bytecodealliance.org/articles/WASI-0.3); [wasi.dev P3 release](https://wasi.dev/releases/wasi-p3)
- **Runtime support.** Wasmtime 45 ran the release candidate. Wasmtime 46 ships WASI 0.3.0 with Component Model async enabled by default. jco supports all of WASI 0.3. Guest toolchains (Rust, Go, JS, Python) were "in progress" at announcement. — [Bytecode Alliance "WASI 0.3"](https://bytecodealliance.org/articles/WASI-0.3)
- Some third-party blogs give conflicting 0.3 release dates (late 2025 or Feb 2026). The official date is 11 June 2026. — [codercops blog](https://blog.codercops.com/blog/wasi-0-3-native-async-io-webassembly-2026) vs. [wasi.dev](https://wasi.dev/releases/wasi-p3)

**Copy semantics and lazy lowering**
- **Copy per crossing today.** The host calls the caller's `cabi_realloc` to allocate memory, then copies values before returning to the caller. — [Bytecode Alliance "The road to Component Model 1.0"](https://bytecodealliance.org/articles/the-road-to-component-model-1-0); [Canonical ABI docs](https://component-model.bytecodealliance.org/advanced/canonical-abi.html)
- **Planned lazy ABI.** The callee returns opaque i32 "lazy value" handles, and the caller later invokes inlinable built-ins with a destination address. Lazy values "can be zero-copy forwarded between calls" and dropped if unused. **[research/planned]** — [The road to Component Model 1.0](https://bytecodealliance.org/articles/the-road-to-component-model-1-0)
- **Stream copies.** Stream read and write with non-zero buffers are completion-based, which avoids intermediate copies. Readiness-style APIs cost an extra copy. — [component-model PR #565](https://github.com/WebAssembly/component-model/pull/565/files)

**Shared-everything threads**
- **Phase 1, unconfirmed for 2026.** Shared-everything threads (champions Andrew Brown, Conrad Watt, Thomas Lively) was listed at Phase 1, but I could not confirm the phase as of Oct 2026. — [WebAssembly/shared-everything-threads](https://github.com/webAssembly/shared-everything-threads); [proposals list mirror](https://awesome.ecosyste.ms/projects/github.com%2FWebAssembly%2Fproposals)
- **It supersedes wasi-threads** and adds component-model built-ins for thread spawn. Shared WasmGC is being developed under the same umbrella (OOPSLA 2026 paper). **[research]** — [wasi-threads repo](https://github.com/webAssembly/wasi-threads); [OOPSLA 2026 paper](https://people.mpi-sws.org/~viktor/papers/oopsla2026-wasm.pdf)
- **Wasmtime and threads.** Wasmtime's stability page says threads support (fuzzing) will be fleshed out as shared-everything-threads advances. — [Wasmtime Wasm proposals stability](https://docs.wasmtime.dev/stability-wasm-proposals.html)

**Users**
- **Zed.** Zed extensions are Rust compiled to Wasm components, with WIT and wit-bindgen interfaces versioned per Zed release, executed via Wasmtime. **[shipped]** — [Zed blog "Life of a Zed Extension: Rust, WIT, Wasm" (summary)](https://www.plushcap.com/content/zed/blog/life-of-a-zed-extension-rust-wit-wasm); sources conflict on compile target, wasip1 per [BoundaryML](https://boundaryml.com/blog/how-to-write-a-zed-extension-for-a-made-up-language) vs. wasip2 per [openapps.pro](https://openapps.pro/packages/zed-extension-api)
- **Microsoft Hyperlight Wasm** (Mar 2025) is a Hyperlight micro-VM "micro-guest" that embeds Wasmtime as a no_std module to run Wasm components. It starts in ~1–2 ms, with a goal of under 1 ms. Hyperlight joined the CNCF Sandbox in Feb 2025. **[prototype in 2025]** — [Microsoft Open Source blog](https://opensource.microsoft.com/blog/?p=97273); [i-programmer](https://www.i-programmer.info/news/89-net/17936-microsoft-announces-hyperlight-wasm.html); [DevClass, Mar 2025 ("nascent project")](https://www.devclass.com/development/2025/03/27/microsoft-releases-experimental-hyperlight-wasm-micro-vms-that-run-wasm-apps/1623450); [The New Stack](https://thenewstack.io/hyperlight-wasm-azure-goes-the-final-wasi-mile/)

### Inferences
- **The Component Model fits extension, plugin and codec boundaries in an engine,** where calls are coarse and copies are acceptable. Examples: image or font decoders, PDF viewers, extensions.
- **It is a poor fit for the DOM↔script↔layout path,** where synchronous, fine-grained, shared-heap access dominates, until lazy lowering and shared-everything threads (with shared GC) mature.
- **WASI 0.3's single event loop and streams align well with engine I/O services** (fetch bodies, media streams).

### Gaps
- No evidence found of any browser engine using components as its *internal* ABI.
- Fastly Compute's specific component-model usage in 2026 was not verified this session.
- The current phase of shared-everything threads was not confirmed from the primary proposals table.

---

## 6. Rust-native and in-process isolation: RLBox, wasm2c, Hyperlight, and Spectre limits

### Takeaway
In-process software fault isolation is cheap and shipped: RLBox with wasm2c in Firefox since 2021. Micro-VMs (Hyperlight) give hardware isolation with millisecond startup.

Neither replaces process-based site isolation against Spectre. RLBox explicitly excludes side-channel defenses, and V8 concluded software cannot fully mitigate Spectre in-process. Use them for memory-safety containment of libraries, layered *inside* site-isolated renderers.

### Cited Findings
**RLBox in Firefox**
- **How it works.** Firefox 95 (Dec 2021) shipped RLBox sandboxing of third-party libraries: compile the library to Wasm, then back to native C via wasm2c, confining it to its own linear memory. RLBox supports both SFI and process backends. Mozilla's rationale was to avoid per-process memory and performance overhead. Mozilla planned to extend the approach to first-party code. **[shipped]** — [Mozilla Hacks, Dec 2021](https://hacks.mozilla.org/2021/12/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/); [Mozilla Attack & Defense blog](https://blog.mozilla.org/attack-and-defense/2021/12/06/webassembly-and-back-again-fine-grained-sandboxing-in-firefox-95/)
- **Spectre scope.** RLBox deliberately excludes side-channel and Spectre defenses in its initial deployments. This comes from secondary summaries and should be checked against the paper. — [emergentmind summary](https://www.emergentmind.com/topics/rlbox); primary paper: [Narayan et al., "Retrofitting Fine Grain Isolation in the Firefox Renderer", USENIX Security 2020](https://www.usenix.org/conference/usenixsecurity20/presentation/narayan) (not re-verified Oct 2026)

**Hardening Wasm against Spectre**
- Swivel (USENIX Security 2021) hardens Wasm against Spectre via compiler passes, with measurable overhead. **[research]** (not re-verified Oct 2026) — [Narayan et al., Swivel, USENIX Security 2021](https://www.usenix.org/conference/usenixsecurity21/presentation/narayan)
- V8's 2019 conclusion was that software mitigations are not a complete Spectre defense, which led to site isolation as the primary mitigation. (not re-verified Oct 2026) — [v8.dev/blog/spectre](https://v8.dev/blog/spectre)

**Hyperlight**
- Hyperlight provides hardware-virtualized micro-VMs with no guest OS, offering per-call isolation in ~1–2 ms startup. It was a CNCF Sandbox project as of Feb 2025, and early-stage, with "evolving API" per 2025 press. **[prototype]** — [Microsoft Open Source blog](https://opensource.microsoft.com/blog/?p=97273); [DevClass](https://www.devclass.com/development/2025/03/27/microsoft-releases-experimental-hyperlight-wasm-micro-vms-that-run-wasm-apps/1623450)

### Inferences
- **Recommended layering for a new Rust engine:**
  1. OS-process site isolation per site (mandatory for Spectre).
  2. Within each renderer, Rust memory safety for engine code.
  3. RLBox or wasm2c-style or Wasm-component sandboxes for C/C++ or untrusted codecs and parsers.
  4. Optionally, Hyperlight-class micro-VMs for heavyweight untrusted plugins.

  Layers 2–4 are cheaper but give no Spectre guarantees across sites.
- **Hyperlight's hardware boundary may resist Spectre better than SFI,** but it adds a VM-exit cost per crossing, and no browser has adopted it.

### Gaps
- No 2026 data found on whether Firefox's RLBox and wasm2c now include Spectre mitigations, or how many libraries are sandboxed.
- No published Spectre analysis of Hyperlight found.
- No browser engine found using micro-VMs for renderers.
