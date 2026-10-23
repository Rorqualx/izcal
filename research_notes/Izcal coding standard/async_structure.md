# Izcal coding standard: async, concurrency and code structure

Compiled 2026-10-18. Primary sources were preferred: maintainers' docs and posts, bug trackers, RFDs, papers, release notes, crate source. Several facts were **measured locally** on rustc/clippy 1.99.1 (2026-10-13) and cargo 1.99.1; those rows say "measured".
Confidence: **High** = primary source read and the claim checked in its text (or measured); **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only.

Scope: this strand covers async, concurrency and code structure. Error handling, type design, testing and performance are covered by sibling notes. Ideas only: Izcal copies no other browser's code (HGBA-2 M-1, Stage 1 D-S10).

## 0. Frame

**Problem.** Izcal has architecture specs but no coding standard. Contributors (human and agent) need rules for writing concurrent code and for laying out crates, and most of those rules should fail a build rather than depend on review.

**What is already decided** (the rules below must fit these):

| ID | Decision | Where |
|---|---|---|
| G3 | Only `hg-substrate-os` may name `std::time`, `std::thread`, `std::net`, `std::fs`, `std::process`, `rand::thread_rng`, `getrandom`, `Instant::now`. Enforced by `hg-graph-lint` and `clippy::disallowed_methods`/`disallowed_types` | Stage 1 §2.2 |
| G1–G9 | Layered crate graph (L0–L6 plus tools), graph-lint rules, `forbid(unsafe_code)` by default, spec anchors | Stage 1 §2 |
| B1 / §3.1 | Substrate is **synchronous traits plus a task model**: `Clock`, `Entropy` (named seeded streams), `Executor` (named sequences, `post`, `fork_join`). Async traits avoided in public interfaces because `dyn` async traits are not usable | Stage 1 §3.1, HGBA-3 B1/B3 |
| §3.2 | Simulation runs the whole multi-process browser in one OS process under one seeded scheduler and a virtual clock | Stage 1 §3.2 |
| B3 | Two schedulers: serial sequences (event loop, IO, navigation) and Rayon-style fork/join for pipeline phases; they never share threads. "Prefer sequences to physical threads" | HGBA-3 B3 |
| §3.3 / DET-1 | Phase purity makes Rayon interleaving irrelevant: workers read immutable inputs and write disjoint outputs; DET-1 compares parallel and sequential hashes | Stage 1 §3.3 |
| §4.1 | IPC endpoints are bound to one sequence; messages arrive as tasks; sync IPC only by allowlist | Stage 1 §4.1 |
| S-4c | Style/layout workers never allocate in the `cppgc` heap, never write traced fields, and read DOM pointers only while the script thread is blocked holding GC scopes | HGBA-2 §6 |
| HAL §5 | Devices are asynchronous; work returns fences | HGBA-2 §5 |

**Success criteria** (measurable later): every candidate rule in Section 11 has an enforcement mechanism named, and at least 70%† of them are enforced by a tool in T0 rather than by review.

---

## 1. How browsers organized concurrency

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | Chromium: "Sequenced execution (on virtual threads) is strongly preferred to single-threaded execution (on physical threads)", because "thread-safety is better achieved via `base::SequencedTaskRunner` than through managing your own physical threads". Exception: types bound to the UI or IO thread. A sequence guarantees only one task runs at a time, in posted order; one thread pool per process | — | [Chromium threading_and_tasks.md](https://chromium.googlesource.com/chromium/src/+/main/docs/threading_and_tasks.md) | read 2026-10-18 | High |
| 1.2 | Chromium discourages locks: "Prefer passing messages across sequences instead"; "Usage of locks is discouraged in Chrome"; "Sequences inherently provide thread-safety". Do not do expensive work on the main or IO thread; default task traits are "don't block", blocking needs `MayBlock` | — | same | read 2026-10-18 | High |
| 1.3 | Chromium cancellation is by `WeakPtr` (not thread-safe) or `CancelableTaskTracker`, which "cannot cancel tasks that have already started to run" | — | same | read 2026-10-18 | High |
| 1.4 | Chromium **bans** the C++ thread support library (`<thread>`, `<mutex>`, `<condition_variable>`, `<future>`, plus C++20 `<barrier>`, `<latch>`, `<semaphore>`, `<stop_token>`), `<chrono>`, and `<random>` engines, because they overlap with `base/synchronization`, `base/time` and `base::RandomBitGenerator`. Abseil's Mutex and time library are banned for the same reason | — | [Chromium C++ features](https://chromium.googlesource.com/chromium/src/+/main/styleguide/c++/c++-features.md) | read 2026-10-18 | High |
| 1.5 | Chromium's `TaskEnvironment` with `TimeSource::MOCK_TIME` mocks `Time::Now()`/`TimeTicks::Now()` and advances only when the main thread and thread pool are idle or on `FastForwardBy`. **Limit:** delayed tasks on unmanaged `base::Thread`s and custom task runners still use real time | — | [task_environment.h](https://chromium.googlesource.com/chromium/src/+/main/base/test/task_environment.h) | read 2026-10-18 | High |
| 1.6 | Firefox **Quantum DOM** (2016) planned to run frames in cooperatively scheduled user-space threads instead of more processes, switching only at JS safe points, to cut background-tab jank without the memory cost of processes (more than about 8 content processes was considered unacceptable). Prerequisite: label every task with its DocGroup; "a big undertaking" | — | [McCloskey, "Mozilla's Quantum Project"](https://billmccloskey.wordpress.com/2016/10/27/mozillas-quantum-project/) | 2016-10-27 | High |
| 1.7 | Quantum DOM scheduling in practice: TabGroup-per-cooperative-thread; because many tasks could not be grouped in time, **preemption was disabled whenever an unlabeled task arrived**. "Over 80% of tasks (weighted with frequency)" labeled by mid-2017 | 80% | [Mozilla Hacks, Bevis Tseng](https://hacks.mozilla.org/2017/06/an-inside-look-at-quantum-dom-scheduling/) | 2017-06-23 | High |
| 1.8 | The scheduler landed preffed off (`dom.ipc.scheduler`); bug reports show freezes and stuck loads when it was enabled (e.g. with NVDA) | — | [Bug 1350432](https://bugzilla.mozilla.org/show_bug.cgi?id=1350432), [Bug 1395558](https://bugzilla.mozilla.org/show_bug.cgi?id=1395558) | 2017 | Medium (search snippets of comments) |
| 1.9 | **Fate:** Bug 1485216, filed 2018-08-22: "CooperativeThreadPool, Scheduler and LabeledEventQueue are unused … never used as far as I can tell". Nathan Froyd: "Sure, we can remove it. Kind of unfortunate, but that's life." Olli Pettay: LabeledEventQueue "shows up in performance profiles". Removal landed in **Firefox 66**, resolved 2019-01-23 | — | [Bug 1485216](https://bugzilla.mozilla.org/show_bug.cgi?id=1485216) (read via REST API) | 2018-08-22 to 2019-01-23 | High |
| 1.10 | Firefox instead re-architected for **site isolation (Fission)**, motivated by Meltdown/Spectre (2018): "we needed to fundamentally re-architecture the security design of Firefox" | — | [Mozilla Hacks, Gakhokidze](https://hacks.mozilla.org/2021/05/introducing-firefox-new-site-isolation-security-architecture/) | 2021-05-18 | High |
| 1.11 | Firefox's later scheduler is `TaskController` ("new scheduler infrastructure", bug 1606706, 2020): tasks and dependencies run in inherited-priority order; its `TaskManager` may only manage main-thread tasks, and frequent reprioritization is costly | — | [Bug 1606706](https://bugzilla.mozilla.org/show_bug.cgi?id=1606706), [TaskController.h](https://searchfox.org/mozilla-central/source/xpcom/threads/TaskController.h) | 2020 | Medium |
| 1.12 | Servo: one **constellation** per instance manages content processes; a script thread manages many pipelines; "The high-level API is IPC-based", with non-IPC implementations for testing and single-process use | — | [Servo book, architecture](https://book.servo.org/design-documentation/architecture.html) | read 2026-10-18 | High |
| 1.13 | Servo moved layout **onto the script thread** in February 2024 because "the web platform almost always requires layout to run synchronously with script"; layout still spawns workers to parallelize | — | [Servo blog](https://servo.org/blog/2024/02/28/gamepads-font-fallback-space-jam/) | 2024-02-28 | High |
| 1.14 | Servo's own list of problems: third-party libraries with library-wide mutexes (fonts) limit parallelism; unbounded parallelism risks "overwhelming the system with too many threads"; IPC overuse caused **file-handle exhaustion** | — | [Servo book, architecture](https://book.servo.org/design-documentation/architecture.html) | read 2026-10-18 | High |
| 1.15 | WebKit replaced OS mutexes and spinlocks with `WTF::Lock` (1 byte, adaptive, waiters parked in a global `ParkingLot`): up to 64× smaller and 180× faster than `pthread_mutex`; ~10% faster JetStream, 5% Speedometer. Philosophy: fine-grained locks are fine; spinlocks are "generally not a good idea" | 64×, 180×, 10%, 5% | [Pizlo, "Locking in WebKit"](https://webkit.org/blog/6161/locking-in-webkit/) | 2016-05-06 | High |
| 1.16 | WebKit adopted **Clang Thread Safety Analysis** for `WTF::Lock` in 2021 (`WTF_GUARDED_BY_LOCK`, `WTF_REQUIRES_LOCK`), removing helpers (`holdLock`, `tryHoldLock`) that the analysis could not check | — | [webkit-dev, May 2021](https://lists.webkit.org/pipermail/webkit-dev/2021-May/031866.html), [Bug 226145](https://bugs.webkit.org/show_bug.cgi?id=226145) | 2021-05 | Medium |
| 1.17 | WebKit hazard: a `ThreadSafeRefCounted` object whose last ref drops on another thread, combined with `CanMakeWeakPtr` (weak pointers do not track cross-thread liveness) | — | [WebKit Bug 238871](https://bugs.webkit.org/show_bug.cgi?id=238871) | 2022 | Medium |

**Chronology: browser concurrency models**

| Year | Event | Why then | Fate |
|---|---|---|---|
| ~2013 | Servo constellation + per-pipeline script/layout tasks over channels | Rust's data-race freedom made parallelism-by-default the research goal | Kept constellation; layout thread folded into script (2024) |
| 2016 | WebKit `WTF::Lock`/`ParkingLot` | OS mutexes too big/slow for fine-grained locking | Survived; inspired Rust's `parking_lot` (idea, Medium) |
| 2016–17 | Quantum DOM cooperative scheduling | Memory cost of more processes; jank from background tabs | Never enabled by default; removed in Firefox 66 (2019) |
| 2018 | Spectre | Same-process isolation of sites no longer credible | Fission (process per site) shipped instead |
| 2019–20 | Chromium `ThreadPool` + sequences as the default model (rename date Medium) | Thread-safety via serial queues instead of locks | Current guidance |
| 2020 | Firefox `TaskController` | Priorities across one main thread | Current |
| 2024 | Servo layout on script thread | Web platform needs synchronous layout | Current |

**Implications:** Izcal's B3 design (sequences, message passing, fork/join for phases) is the model that survived in Chromium and that Servo converged on. The failed one is cooperative user-space threads inside one process: it needed every task labeled, was never turned on, and Spectre removed its premise. A rule that depends on every task carrying correct metadata must be enforced by the type system, not by convention.

---

## 2. Rust async: what is stable, what is not (October 2026)

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | `futures` 0.1 announced as a zero-cost abstraction: combined futures compile to a state machine | — | [Turon, "Zero-cost futures in Rust"](https://aturon.github.io/blog/2016/08/11/futures/) | 2016-08-11 | High |
| 2.2 | `async`/`.await` stabilized in Rust 1.39 | — | [Rust 1.39 announcement](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0/) | 2019-11-07 | High |
| 2.3 | Async fn in traits (AFIT) and RPITIT stabilized in **Rust 1.75**. Limits: such traits are **not dyn-compatible**; the `Send`-ness of returned futures must be fixed at trait definition, callers cannot add it; bare `async fn` in a public trait warns. Workaround: `trait-variant` (`#[trait_variant::make]`) | — | [Rust blog, Mandry](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/) | 2023-12-21 (1.75 shipped 2023-12-28, date Medium) | High |
| 2.4 | Rust 2024 edition and async closures (`async \|\| {}`) stabilized in 1.85 | — | [Rust 1.85 announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) | 2025-02-20 | High |
| 2.5 | 2026 project goal "Native async fn dynamic dispatch in traits": the 2026 goal "is only to settle on a *specific design*, not necessarily to implement it or even stabilize it". Timespan 2026–2027; phase 2 adds a nightly-only `std::preview::dyn_box!`. Status Accepted, funding Partial | — | [Rust goals 2026: afidt-box](https://goals.rust-lang.org/2026/afidt-box.html) | read 2026-10-18 | High |
| 2.6 | 2026 goal "Prepare TAIT + RTN for stabilization": "Full stabilization is blocked on the next-gen trait solver work", intended late 2026. An earlier RTN stabilization PR was closed because RTN can name opaque types in arbitrary positions | — | [Rust goals 2026: rtn](https://goals.rust-lang.org/2026/rtn.html), [rust#138424](https://github.com/rust-lang/rust/pull/138424) | read 2026-10-18 | High |
| 2.7 | **Measured** on stable rustc 1.99.1: `&dyn Trait` with an `async fn` method fails with E0038 "not dyn compatible … because method `get` is `async`"; `T: Fetch<get(..): Send>` fails with E0658 "return type notation is experimental" (issue #109417) | — | local compile, scratch test | 2026-10-18 | High |
| 2.8 | rustc lint `async_fn_in_trait` (warn by default) fires on `async fn` in a publicly reachable trait; `let_underscore_lock` is deny by default | — | measured: `rustc -W help` 1.99.1 | 2026-10-18 | High |
| 2.9 | `async-std` officially discontinued in 1.13.1; "all users and libraries" should migrate to `smol`; maintainers prefer to consolidate effort there | — | [async-std README](https://github.com/async-rs/async-std), [docs.rs](https://docs.rs/async-std/latest/) | 2025-02-21 | High |
| 2.10 | "Function colour": async functions can only be called from async functions, which splits every API in two; Go, Lua and Ruby avoid it with stackful threads/fibers | — | [Nystrom, "What color is your function?"](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/) | 2015-02-01 | High |
| 2.11 | **Scoped task trilemma**: a sound API gives at most two of concurrency, parallelizability and borrowing. `spawn` needs `'static`; `thread::scope`/`rayon::join` must join before returning; `select!`/`FuturesUnordered` cannot run children in parallel. Cause: any value can be leaked without running its destructor | — | [withoutboats](https://without.boats/blog/the-scoped-task-trilemma/) | 2023-04-08 | High |
| 2.12 | `moro` (structured async scopes) is experimental; "Parallel moro tasks cannot, with Rust as it is today, be done safely" | — | [moro README](https://github.com/nikomatsakis/moro) | read 2026-10-18 | High |
| 2.13 | `async-scoped`: only `scope_and_block` is fully safe; `scope` and `scope_and_collect` are `unsafe` because the returned stream/future can be forgotten | — | [async-scoped docs](https://docs.rs/async-scoped/latest/async_scoped/) | read 2026-10-18 | High |

**Chronology: Rust async**

| Date | Event | Limit it left |
|---|---|---|
| 2016-08 | futures 0.1 | Combinator-heavy, no borrowing across yields |
| 2019-11 | async/await in 1.39 | Function colour; executor not in std |
| 2020-12 | tokio 1.0 (Medium, from memory) | Ecosystem coupled to one runtime |
| 2022-08 | `std::thread::scope` in 1.63 (Medium) | Sync only; async scoped tasks still unsound |
| 2023-12 | AFIT/RPITIT in 1.75 | Not `dyn`; `Send` bounds fixed at definition |
| 2025-02 | async-std discontinued; async closures in 1.85 | — |
| 2026 | Goals: dyn async design only; RTN blocked on new trait solver | Still unstable in 1.99.1 (measured) |

**Implications:** Stage 1's decision to keep public substrate traits synchronous is confirmed by measurement. No Rust release before 2027 is likely to make `dyn` async traits usable, and RTN is not stable either. Async can be used inside a crate where static dispatch is enough, but it should not shape cross-crate contracts.

---

## 3. Structured concurrency outside Rust

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | Trio's nurseries: "the nursery block doesn't exit until all the tasks inside it have exited"; child exceptions are re-raised in the parent; siblings are cancelled. Unscoped `go`/spawn breaks abstraction, cleanup, error propagation and backpressure | — | [N. J. Smith](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/) | 2018-04-25 | High |
| 3.2 | Kotlin: coroutines form a parent/child tree; a parent waits for children; failure or cancellation of a parent cancels children recursively; new coroutines can only be launched in a `CoroutineScope` | — | [Kotlin docs](https://kotlinlang.org/docs/coroutines-basics.html) | read 2026-10-18 | High |
| 3.3 | Swift 6.2 (SE-0466) lets modules default to main-actor isolation because "Writing a single-threaded program is surprisingly difficult under the Swift 6 language mode" (false-positive data-race diagnostics) | — | [SE-0466](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0466-control-default-actor-isolation.md) | 2025 | High |
| 3.4 | Java structured concurrency still in its sixth preview in JDK 26 | — | [JEP 525](https://openjdk.org/jeps/525) (cited by HGBA-3, not re-read) | 2026 | Medium |

**Implications:** Every mainstream structured-concurrency design ties a child's lifetime to a lexical owner and propagates cancellation downwards. Swift's retreat shows that "concurrent by default" costs real effort; serial-by-default (sequences) is the cheaper default. In Rust the owner can be a sequence or a phase scope, which the trilemma (2.11) allows; a borrowing parallel async scope cannot be built safely.

---

## 4. Cancellation, blocking and locking hazards in async Rust

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | Tokio defines cancel safety as: dropping an unfinished future and recreating it must be a no-op. Not cancel-safe: `read_exact`, `read_to_end`, `read_to_string`, `write_all` (lose data); `Mutex::lock`, `RwLock::read/write`, `Semaphore::acquire`, `Notify::notified` (lose queue place) | tokio 1.53.3 | [tokio::select! docs](https://docs.rs/tokio/latest/tokio/macro.select.html) | read 2026-10-18 | High |
| 4.2 | Oxide RFD 400 (Rain Paharia): split operations into cancel-safe and cancel-unsafe steps (`Sender::reserve` then `permit.send`); mark cancel-unsafe APIs in name and docs; "this guide recommends avoiding `tokio::sync::Mutex`"; "task aborts should be avoided entirely"; move cancel-unsafe work into spawned tasks | — | [RFD 400](https://rfd.shared.oxide.computer/rfd/400) | published (date not shown) | High |
| 4.3 | Oxide RFD 609 **futurelock** (David Pacheco): a task polling future F1 stops polling F2, which holds a place in a lock or channel queue that F1 needs, so the task hangs forever. Incident: omicron#9259 (opened 2025-10-19) — every database-touching future joined the hang and "essentially every incoming HTTP request hung indefinitely"; found with DTrace | 1 incident, all requests | [RFD 609](https://rfd.shared.oxide.computer/rfd/609), [omicron#9259](https://github.com/oxidecomputer/omicron/issues/9259) | 2025-10 | High |
| 4.4 | RFD 609 rules: beware `&mut future` arms in `select!` combined with `await` in a handler or after the `select!`; spawn the future and select on its `JoinHandle`; never stop polling a future you started; "Using `send_timeout()` is *not* a mitigation"; a larger channel is not a fix ("there is no safe size to pick"); `join_all` cannot futurelock. It asks whether clippy could lint the pattern (no lint exists) | — | [RFD 609](https://rfd.shared.oxide.computer/rfd/609) | 2025 | High |
| 4.5 | **Conflict between two Oxide RFDs**: RFD 400 recommends creating a cancel-unsafe future once and `select!`ing over `&mut` to it; RFD 609 identifies that pattern (with an await in a branch) as the futurelock shape | — | RFD 400 vs RFD 609 | — | High (both read) |
| 4.6 | Tokio: "it is ok and often preferred to use the ordinary Mutex from the standard library in asynchronous code"; the async mutex is only for holding a lock across `.await` | — | [tokio::sync::Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html) | read 2026-10-18 | High |
| 4.7 | Rule of thumb: no more than **10–100 µs** between `.await` points; blocking IO → `spawn_blocking`; CPU work → Rayon with a oneshot back; forever-loops → dedicated thread | 10–100 µs | [Alice Ryhl, "Async: What is blocking?"](https://ryhl.io/blog/async-what-is-blocking/) | 2020-12-21 | High |
| 4.8 | Tokio unbounded channel: "send will always complete immediately"; only the bounded variant "provides backpressure" | — | [tokio::sync::mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) | read 2026-10-18 | High |
| 4.9 | Clippy has lints for guards held across `await`: `await_holding_lock` (std and parking_lot guards), `await_holding_refcell_ref`, and `await_holding_invalid_type` (configurable list `await-holding-invalid-types`), all in group `suspicious` | — | measured: `cargo clippy --explain`, `clippy-driver -W help` 1.99.1 | 2026-10-18 | High |
| 4.10 | Fuchsia netstack3 encodes **lock ordering in types** (`lock_order` crate: each lock has a level type; ordering via traits) to get a compile-time guarantee against lock-order deadlock; republished on crates.io as `lock_tree` | — | [netstack3 lock_ordering.rs](https://docs.fuchsia.dev/rust/src/netstack3_core/lock_ordering.rs.html), [lock_tree](https://rust-digger.code-maven.com/crates/lock_tree) | read 2026-10-18 | Medium (module docs only; mechanism not read) |
| 4.11 | `loom` exhaustively permutes executions under the C11 memory model but treats `SeqCst` loads/stores as `AcqRel` and skips some load-buffering executions, so a pass is not proof. `shuttle` is randomized (PCT), "not sound" but "scales to much larger test cases than Loom" | — | [loom](https://github.com/tokio-rs/loom), [shuttle](https://github.com/awslabs/shuttle) | read 2026-10-18 | High |

**Implications:** The worst async failures are not data races, which Rust prevents; they are lost progress (futurelock, dropped half-done operations) and invisible waiting. They come from three things Izcal can forbid structurally: async locks, `select!` over partially polled futures, and queues without bounds. No lint catches futurelock, so the rule must remove the construct, not ask reviewers to spot it.

---

## 5. Deterministic simulation: what made it work and what broke it

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | FoundationDB: "All database code is deterministic; accordingly multithreaded concurrency is avoided (instead, one database node is deployed per core)". "All sources of nondeterminism and communication are abstracted, including network, disk, time, and pseudo random number generator." Flow adds async/await-like actors to C++; "The production implementation is a simple shim to the relevant system calls" | — | [FDB paper, SIGMOD 2021](https://www.foundationdb.org/files/fdb-paper.pdf) §simulation | 2021 | High |
| 5.2 | FDB **limitation**: simulation "is also unable to test third-party libraries or dependencies, or even first-party code not implemented in Flow. As a consequence, we have largely avoided taking dependencies on external systems." Several bugs came from the OS contract being weaker than believed | — | same | 2021 | High |
| 5.3 | FDB "buggification": code offers the simulator places to inject unusual but legal behavior (spurious errors, delays, odd tuning values); swarm testing; `TEST(cond)` coverage macros show whether a rare condition was ever reached | — | same | 2021 | High |
| 5.4 | TigerBeetle is **single-threaded** by design: the workload is contentious and a single core suffices if used well; "keeping the system single threaded greatly simplifies the programming model and makes testing much more efficient". Determinism is a meta-principle; the VOPR runs a whole cluster on one thread | — | [TigerBeetle ARCHITECTURE.md](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/ARCHITECTURE.md), [performance.md](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/concepts/performance.md) | read 2026-10-18 | High |
| 5.5 | TigerStyle: "all loops and all queues must have a fixed upper bound"; no dynamic allocation after initialization; at least two assertions per function on average; "hard limit of 70 lines per function"; zero dependencies | 70 lines, 2 asserts/fn | [TIGER_STYLE.md](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md) | read 2026-10-18 | High |
| 5.6 | S2 ran DST on a Tokio codebase: current-thread runtime, paused clock, seeded Tokio RNG (which picks `select!` branches), turmoil for networking, and **mad-turmoil** (a madsim-derived fork) overriding libc `getrandom`, `getentropy`, `CCRandomGenerateBytes` and `clock_gettime`. A CI meta-test reruns a seed and compares TRACE logs. Problems: third-party crates as hidden threads, RNG and clocks ("every single one of them represents a possible source of multi-threading"), and `HashMap` randomization | 17 bugs found | [S2, "Deterministic simulation testing for async Rust"](https://s2.dev/blog/dst) | 2025-04-02 | High |
| 5.7 | Polar Signals chose plain **state machines without `async`** (`receive`/`tick`) for their new Rust database, so "there are no runaway futures spawned inside state machines" and the message bus owns scheduling, time and randomness. They recommend madsim for existing codebases. Costs: explicit state transitions, logic leaking into the untested production driver, mocking nondeterministic dependencies | — | [Polar Signals](https://blog.polarsignals.com/blog/posts/2025/07/08/dst-rust) | 2025-07-08 | High |
| 5.8 | madsim replaces tokio and friends via Cargo renames and `--cfg madsim`, and requires patched `quanta`/`getrandom`. turmoil runs hosts "within a single thread" with simulated net and fs crates | — | [madsim](https://github.com/madsim-rs/madsim), [turmoil](https://github.com/tokio-rs/turmoil) | read 2026-10-18 | High |

**Chronology: deterministic simulation**

| Year | System | Mechanism | What it taught |
|---|---|---|---|
| ~2010–2014 | FoundationDB Flow | Actor language, all IO behind interfaces, one thread | Determinism must be designed in; dependencies cannot be simulated |
| 2020s | TigerBeetle VOPR | Single thread, static allocation, bounded queues | Bounds and single-threading make simulation cheap |
| 2022–24 | madsim, turmoil | Replace tokio APIs / simulated net | Works for existing tokio code; leaks through dependencies |
| 2025 | S2 (mad-turmoil) | libc symbol overrides + tokio paused clock | Third-party crates and `HashMap` are the leaks |
| 2025 | Polar Signals | Sync state machines, message bus | Removing `async` from the core removes the leak class |
| 2026 | Izcal M0 | V8 predictable platform fork | Flags were not enough; the platform had to be owned (Stage 1 §3.3) |

**Implications:** Every successful system made the scheduler the *only* source of concurrency, time and randomness, and every one hit the same hole: code it did not own. Izcal's G3 covers `hg-*` crates only. Dependencies (Stylo, Rayon, WebRender, `std::collections::HashMap`) are where determinism will leak, so they need an inventory and a gate, as FDB's "avoid dependencies" and S2's experience show.

---

## 6. Data-parallel phases

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | Rayon: `reduce` application order "is not fully specified"; floating-point `sum` results "are not fully deterministic"; for `fold`, "The number of results is nondeterministic, as is the point where the breaks occur" | rayon 1.12.0 | [ParallelIterator docs](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html) | read 2026-10-18 | High |
| 6.2 | A known workaround fixes the grouping: fixed-size blocks folded serially, results gathered by indexed collect, merged in order, so grouping depends only on a block-size constant | — | [rustyml guide](https://docs.rs/crate/rustyml/0.14.0/source/guide/en/src/Chapter-06/6.3._Parallel_Reductions.md) | 2026 | Low (third-party crate docs) |
| 6.3 | Cilk++ reducer hyperobjects (Frigo et al., SPAA 2009 best paper) give parallel reductions whose result matches the serial order for associative operations | — | [SPAA 2009](https://dl.acm.org/doi/10.1145/1583991.1584017) | 2009 | Medium (paper not read) |
| 6.4 | `rayon-core` 1.13 `ThreadPoolBuilder` has `spawn_handler`, `use_current_thread` and `build_scoped`, so an embedder can create the worker threads itself | — | local source, `rayon-core-1.13.0/src/lib.rs` | 2026-10-18 | High |
| 6.5 | **Stylo 0.22**: `traverse_dom(traversal, token, pool: Option<&rayon::ThreadPool>)` takes the embedder's pool. Stylo also defines a lazy global `STYLE_THREAD_POOL` that, if touched, spawns its own threads with `std::thread::Builder`, sized `num_cpus::get() * 3 / 4`, capped at `STYLO_MAX_THREADS = 6` | 6 threads max | stylo-0.22.0 `driver.rs`, `global_style_data.rs` (downloaded from crates.io) | 2026-10-18 | High |

**Implications:** Phase purity (DET-1) is necessary but not sufficient: a pure phase that reduces floats or emits results in completion order is still nondeterministic. The rules must cover reduction shape and output order. Stylo can be driven entirely from the substrate's pool; the global pool is a trap that a hook should block.

---

## 7. Workspace and crate layout

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | For 10k–1M-line projects "the flat layout makes the most sense": crates one level under `crates/`, root is a virtual manifest, crate name equals folder name, internal crates `version = "0.0.0"`, automation in an `xtask` crate; "tree structure tends to deteriorate over time" | — | [matklad, "Large Rust Workspaces"](https://matklad.github.io/2021/08/22/large-rust-workspaces.html) | 2021-08-22 | High |
| 7.2 | Build time depends on graph shape: avoid long chains, prefer "a common vocabulary crate, a number of independent features, and a leaf crate". Proc macros block pipelining; `serde` derive near the base makes everything wait for `syn`; "do not put generic code at the boundaries between the crates"; `-D warnings` in RUSTFLAGS, not `#![deny(warnings)]` | — | [matklad, "Fast Rust Builds"](https://matklad.github.io/2021/09/04/fast-rust-builds.html) | 2021-09-04 | High |
| 7.3 | Crate counts in large Rust repos (directory entries): Zed `crates/` 250, uv `crates/` 92, Bevy `crates/` 65, Servo `components/` 40, rustc `compiler/` 80 | see left | GitHub contents API, measured | 2026-10-18 | High (count of entries; a few may be non-crate files) |
| 7.4 | Firefox links all libxul Rust into one generated staticlib (**gkrust**); "only a single `RustLibrary` may be linked into a binary"; all third-party crates vendored in `third_party/rust`, license-checked and `cargo vet`-audited | — | [Firefox source docs: Rust](https://firefox-source-docs.mozilla.org/build/buildsystem/rust.html) | read 2026-10-18 | High |
| 7.5 | rustc's `tidy` enforces 100-column lines (non-Rust files; Rust is left to rustfmt), no `TODO`/`XXX`, no unexplained `ignore` doc tests, and a **3000-line file limit** suppressible per file with `// ignore-tidy-filelength` | 100 cols, 3000 lines | [tidy style.rs](https://github.com/rust-lang/rust/blob/main/src/tools/tidy/src/style.rs) | read 2026-10-18 | High (header says "non-Rust files" but the extension list includes `rs`; Medium on whether the limit applies to `.rs`) |
| 7.6 | Generated code can be checked in and kept fresh by a test that regenerates it and fails on a diff: readable, goto-definition works, `src/` writable only in development | — | [matklad, "Self Modifying Code"](https://matklad.github.io/2022/03/26/self-modifying-code.html) | 2022-03-26 | High |
| 7.7 | Rust API Guidelines: casing per RFC 430 (C-CASE), `as_`/`to_`/`into_` (C-CONV), types `Send`/`Sync` where possible (C-SEND-SYNC), sealed traits (C-SEALED), private struct fields (C-STRUCT-PRIVATE), function docs cover errors, panics, safety (C-FAILURE), destructors never fail or block without alternatives (C-DTOR-FAIL, C-DTOR-BLOCK) | — | [API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html) | read 2026-10-18 | High |

**Implications:** Izcal's layer graph already gives the "vocabulary at the bottom, features in the middle, leaf binaries" shape. The open choices are a flat directory layout, where codegen output lives, and a file-size ceiling. Izcal has six-plus binaries (Stage 1 L6), so shared crates are linked repeatedly; generics at crate boundaries cost more there.

---

## 8. Cargo features, lints and dependency policy

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | "features should be *additive*. That is, enabling a feature should not disable functionality … A feature should not introduce a SemVer-incompatible change." Cargo builds a dependency with the **union** of features requested by all dependents. Mutually exclusive features "should be avoided if at all possible" | — | [Cargo book: Features](https://doc.rust-lang.org/cargo/reference/features.html) | read 2026-10-18 | High |
| 8.2 | Resolver 2 stops unifying features for targets not being built, build-dependencies and proc macros, and dev-dependencies not being built. Features are unified across all packages selected in one invocation; `Cargo.lock` is computed as if all features of all members were on | — | [Cargo book: Resolver](https://doc.rust-lang.org/cargo/reference/resolver.html) | read 2026-10-18 | High |
| 8.3 | Per-package or whole-workspace unification (`resolver.feature-unification`, RFC 3692) is **still unstable** (`-Z feature-unification`) in the cargo 1.99 docs. So `cargo build -p hg-renderer-main` and `cargo build --workspace` can compile dependencies with different features | — | [Cargo unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html) | read 2026-10-18 | High |
| 8.4 | `[lints]` and `[workspace.lints]` in Cargo.toml (inherited with `[lints] workspace = true`) arrived in Rust 1.74 | — | [Rust 1.74 announcement](https://blog.rust-lang.org/2023/11/16/Rust-1.74.0/) | 2023-11-16 | High |
| 8.5 | MSRV-aware resolver: opt-in in 1.84 (`resolver = "3"` or `incompatible-rust-versions = "fallback"`), default for edition 2024 | — | [Rust 1.84 announcement](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/) | 2025-01-09 | High |
| 8.6 | `-Z direct-minimal-versions` and `-Z min-publish-age` (refuse versions newer than a minimum age, e.g. `"14 days"`) are unstable in cargo 1.99 | — | [Cargo unstable features](https://doc.rust-lang.org/cargo/reference/unstable.html) | read 2026-10-18 | High |
| 8.7 | cargo-deny runs `advisories`, `bans`, `licenses`, `sources`. `bans` supports `multiple-versions`, `wildcards`, `deny` entries with `wrappers` (only named crates may depend directly) and `use-instead`, an `allow` list, per-crate `features` allow/deny/exact, `workspace-default-features`/`external-default-features`, and `build.allow-build-scripts` (empty list = no build scripts), `build.executables`, `build.bypass` with script checksums | — | [cargo-deny bans config](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html) | read 2026-10-18 | High |
| 8.8 | **arrayref attack**: compromised credentials published `arrayref` 0.3.10, `internment` 0.8.7 and `append-only-vec` 0.1.9 with one new dependency, `proc-macro1` (typosquat of `proc-macro2`), whose **build script** downloaded a payload; resolving the dependency and building was enough. Online about 86 minutes; the attacker also yanked 0.3.5–0.3.9 so Cargo's warnings pointed at the bad version | 86 min; RUSTSEC-2026-0260, -0265 | [Rust blog](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/), [RUSTSEC-2026-0265](https://rustsec.org/advisories/RUSTSEC-2026-0265), [The Hacker News](https://thehackernews.com/2026/08/rust-supply-chain-attack-puts-build.html) | 2026-08-20 | High (Rust blog title/summary via search; details via THN, Medium) |

### 8.9 Lint names verified on clippy 0.1.99 / rustc 1.99.1 (measured 2026-10-18)

Each name below was checked with `cargo clippy --explain` (a fake name returns "unknown lint") and grouped with `clippy-driver rustc -W help`.

| Lint | Group | Use in this standard |
|---|---|---|
| `clippy::disallowed_methods`, `disallowed_types`, `disallowed_macros` | style | G3 lists; also bans unbounded channels, global pools, `select!` (config keys `disallowed-methods` etc., each entry may carry `reason` and `replacement`) |
| `clippy::await_holding_lock`, `await_holding_refcell_ref`, `await_holding_invalid_type` | suspicious | No guard across `await`; config `await-holding-invalid-types` for Izcal's own guards |
| `clippy::let_underscore_future` | suspicious | Dropped futures |
| `clippy::async_yields_async` | correctness | — |
| `clippy::arc_with_non_send_sync` | suspicious | — |
| `clippy::large_futures` | pedantic | `future-size-threshold`, default 16384 bytes |
| `clippy::unused_async` | pedantic | — |
| `clippy::future_not_send` | nursery | Futures that must cross threads |
| `clippy::significant_drop_tightening`, `significant_drop_in_scrutinee` | nursery | Lock guards living too long |
| `clippy::non_send_fields_in_send_ty` | nursery | Manual `Send` impls |
| `clippy::iter_over_hash_type` | restriction | Iteration order of `HashMap`/`HashSet` |
| `clippy::mutex_atomic`, `mutex_integer`, `rc_mutex` | restriction | — |
| `clippy::mem_forget` | restriction | Leak-based soundness holes (2.11) |
| `clippy::float_arithmetic` | restriction | Candidate for reduction-heavy phase code only |
| `clippy::mod_module_files` / `self_named_module_files` | restriction | Pick one module-file style |
| `clippy::missing_docs_in_private_items` | restriction | Probably too strong |
| `clippy::too_many_lines` | pedantic | `too-many-lines-threshold`, default 100 |
| `clippy::cognitive_complexity` | restriction | — |
| `clippy::excessive_nesting` | complexity | `excessive-nesting-threshold`, default 0 (off) |
| `clippy::wildcard_imports` | pedantic | — |
| `clippy::redundant_pub_crate` | nursery | **Conflicts** with rustc `unreachable_pub` (it prefers `pub` inside private modules) |
| `clippy::allow_attributes_without_reason` | restriction | Every `allow` carries a reason |
| `clippy::multiple_crate_versions` (`allowed-duplicate-crates`), `wildcard_dependencies`, `negative_feature_names`, `redundant_feature_names`, `cargo_common_metadata` | cargo | Manifest hygiene |
| `clippy::missing_errors_doc`, `missing_panics_doc` | pedantic | Doc completeness |
| rustc `missing_docs`, `unreachable_pub`, `unnameable_types`, `unused_crate_dependencies`, `missing_debug_implementations`, `unsafe_code` | allow by default | Turn on in `[workspace.lints]` |
| rustc `async_fn_in_trait` | warn | Deny for public traits |
| rustc `let_underscore_lock` | deny | Already on |
| `rustdoc::broken_intra_doc_links`, `private_intra_doc_links` (warn), `missing_crate_level_docs`, `unescaped_backticks` (allow) | rustdoc | Docs |

**Limits of these lints** (from how clippy works; High): `disallowed_*` match specific function, method, type or macro paths, not whole modules, so banning "`std::thread`" means listing `spawn`, `Builder`, `scope`, `sleep` and so on. Clippy lints only workspace crates, never dependency code. Static items (such as Stylo's `STYLE_THREAD_POOL`) are not methods or types, so a path ban may not catch them; a text hook or graph lint is needed.

**Implications:** Most structural rules in this strand can be machine-enforced with existing tools (clippy, rustc lints, cargo-deny, `hg-graph-lint`). The gaps are dependency behavior (threads, clocks and `HashMap` seeds inside crates Izcal does not own), futurelock, atomics reasoning and lock ordering. Those need either a type-level design (lock levels, sequence-bound handles) or review.

---

## 9. Lessons learned

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Chromium | Made sequences the default, discouraged locks, banned std thread/time/random | A ban works when an in-house replacement exists and the ban is mechanical | G3 plus the extended ban lists in A1; the substrate is the replacement |
| Chromium | MOCK_TIME covers managed threads only | Any thread the scheduler does not own leaks real time | A3: the substrate creates every thread, including Rayon's and Stylo's |
| Firefox Quantum DOM | Needed every task labeled; preemption off for unlabeled tasks; never enabled; removed | A scheme that is correct only if all metadata is right fails at scale | Make ownership structural: a task can only be posted through a `SequenceHandle`, so it is labeled by construction |
| Firefox Fission / Spectre | Thread-level isolation inside a process lost its premise | Isolation is per process; concurrency inside a process is for speed, not safety | Keep I-1 process isolation; no rule here relies on threads for security |
| Servo | Layout thread merged into script thread; IPC overuse exhausted file handles; unbounded parallelism | Synchronous web semantics push work onto one thread; resource use must be bounded | Script-thread layout (Stage 1 §5.1); bounded queues and handle budgets (A6) |
| WebKit | Small fast locks, then compiler-checked lock annotations | Locks are acceptable when cheap and statically checked | Locks allowed for short critical sections; lock levels typed (A9) |
| Oxide | Futurelock hung every API request; tokio Mutex discouraged | Async locks and `select!` over partly polled futures cause hangs no lint finds | Ban async mutexes and `select!` outside the substrate (A8, A10) |
| async-std | Discontinued; ecosystem consolidated | Executors are ecosystem bets | Izcal owns its executor; no third-party runtime outside `hg-substrate-os` (A5) |
| FoundationDB | All nondeterminism behind interfaces; dependencies untestable, so avoided | Determinism is a dependency policy as much as a code rule | Nondeterminism inventory for dependencies (A14) |
| TigerBeetle | Single thread, bounded everything, 70-line functions | Bounds make behavior simulatable and tail latency predictable | Bounded queues (A6); function length lint (S11) |
| S2 | DST on tokio leaked through third-party crates and `HashMap` | The leaks are in code you did not write | Seeded hashers and dependency gate (A2, A14) |
| Polar Signals | Removed `async` from the core | No async in the core means no runaway futures | Substrate traits stay sync; tasks are run-to-completion closures (A4) |
| Rayon / Stylo | Reduction order and fold splits nondeterministic; Stylo has a global pool | Phase purity must include output order and reduction shape | A13 |
| Rust lang | `dyn` async and RTN unstable in 1.99 | Do not design public contracts around features expected "soon" | A4, with a re-evaluation trigger |
| arrayref | Build script in a new transitive dep ran malware | Build scripts are code execution at resolve time | S7: build-script allowlist; vendoring; minimum publish age |
| Cargo | Features unify by union; per-package unification unstable | A non-additive feature changes other crates' behavior silently | S6: additive only; test feature sets per binary |

---

## 10. First-principles notes for the synthesis author

These are inputs, not the spec. Inherited assumptions and whether to keep them:

| Inherited assumption | Keep or break | Why |
|---|---|---|
| "Use tokio for async" (Rust ecosystem default) | **Break** inside the engine | Izcal's executor is the substrate (B1); a second runtime is a second scheduler that simulation cannot see |
| "async/await everywhere for IO" | **Break** at contract boundaries, keep optionally inside a crate | `dyn` async unusable (2.7); sequences plus run-to-completion tasks already model IO completion; async inside a leaf crate can be driven by a substrate-provided `block_on`-free poller |
| "Locks are bad" (Chromium) | **Keep, softened** | Message passing between sequences by default; short std/parking_lot locks allowed inside a sequence-owned structure; WebKit shows locks are fine when cheap and checked |
| "Parallel by default" (early Servo) | **Break** | Servo and Swift moved back; serial sequences by default, explicit fork/join phases |
| "Many small crates speed builds" | **Keep with care** | True for parallelism and incremental builds; false when chains are deep or generics cross boundaries (7.2) |
| "Generated code goes in OUT_DIR via build.rs" | **Break** | Build scripts are an attack and cost surface (8.8, 7.2); checked-in generated code is reviewable and IDE-friendly (7.6) |

---

## 11. Candidate rules for the Izcal coding standard

Each rule says how it can be enforced. "graph" = `hg-graph-lint` on `cargo metadata`; "deny" = cargo-deny; "hook" = a custom T0 check (grep/AST, e.g. `ast-grep` rule); "review" = review only. Thresholds marked † are first-principles placeholders, not measurements.

### A. Async and concurrency

| ID | Rule | Enforced by |
|---|---|---|
| **A1** | **No ambient effects outside `hg-substrate-os`** (extends G3). Banned paths in every other crate: `std::thread::{spawn, Builder, scope, sleep, available_parallelism}`, `std::time::{Instant::now, SystemTime::now}`, `std::process::Command`, `std::net::*` types, `std::fs::*`, `std::env::{var, vars}`, `rand::thread_rng`, `rand::rng`, `getrandom::*`, `num_cpus::get`, `std::collections::hash_map::RandomState::new`, `HashMap::new`/`HashSet::new`/`Default` with `RandomState` | `clippy::disallowed_methods` / `disallowed_types` in workspace `clippy.toml` (each with `reason` and `replacement`) + graph (crates `rand`, `getrandom`, `num_cpus`, `tokio`, `async-std`, `smol` only reachable via `hg-substrate-os`) + deny `bans.deny` with `wrappers = ["hg-substrate-os"]` |
| **A2** | **Deterministic hashing.** Hash maps use a hasher type supplied by `hg-types`, keyed from `Entropy::stream(EntropyPurpose::HashSeed)`: seeded in simulation, random in production (keeps HashDoS resistance for web-controlled keys). Output-affecting code never iterates a hash map; use `BTreeMap`, an insertion-ordered map, or sort first | `disallowed_types` on `std::collections::hash_map::RandomState` + `clippy::iter_over_hash_type` (restriction) enabled in behavior crates (G9 list) |
| **A3** | **The substrate owns every thread and every task.** Work is posted only through `Executor::post` on a named sequence or through `fork_join()`. No detached spawn. Rayon pools are built by the substrate (`ThreadPoolBuilder::spawn_handler`); Stylo receives that pool through `traverse_dom`; nobody touches `style::global_style_data::STYLE_THREAD_POOL` or Rayon's global pool | `disallowed_methods`: `rayon::spawn`, `rayon::spawn_fifo`, `rayon::ThreadPoolBuilder::build_global`, `rayon::ThreadPoolBuilder::build` (outside substrate) + hook: text ban on `STYLE_THREAD_POOL` + sim check: thread count after startup equals the substrate's count |
| **A4** | **Public cross-crate contracts are synchronous.** Traits exported across crates use sync methods that return a handle, post a completion task to a `SequenceBound` sink, or return a HAL fence. `async fn` in traits only for crate-private, statically dispatched traits. No `#[async_trait]` (a boxed allocation per call) in hot paths. Re-evaluate when `dyn` async traits and RTN are both stable (trigger, Section 2) | rustc `async_fn_in_trait = "deny"` + deny `bans.deny = ["async-trait"]` with reviewed wrappers + review |
| **A5** | **No third-party async runtime in the engine.** `tokio`, `smol`, `async-std`, `futures::executor` only inside `hg-substrate-os` (and only if the network service needs them). Protocol libraries must be **sans-IO** (state machines fed bytes and time, like Polar Signals' design), so they run identically under the simulator | graph + deny `wrappers` + review of each new protocol dependency |
| **A6** | **Every queue is bounded.** Every channel, sequence queue and IPC endpoint has a declared capacity and an overflow policy (backpressure, reject with error, or drop-oldest with a counter). Unbounded constructors are banned. Default capacity per sequence: 1024 tasks† | `disallowed_methods`: `std::sync::mpsc::channel` (unbounded), `tokio::sync::mpsc::unbounded_channel`, `crossbeam_channel::unbounded`, `futures::channel::mpsc::unbounded`, `flume::unbounded` + `hg-ipc-codegen` rejects endpoints without capacity |
| **A7** | **Tasks run to completion quickly.** No task on a sequence blocks (no sync IO, no sleep, no waiting on another sequence). Budget: ≤100 µs† per task on the script and IO sequences (Ryhl's upper bound); longer work goes to `fork_join` or a blocking-allowed sequence created with an explicit `MayBlock` priority. The substrate's debug build reports tasks over budget | A1 bans + substrate long-task detector in debug and DST farm (reports, not fails, until calibrated) + review |
| **A8** | **No guard across a suspension.** No lock guard, `RefCell` borrow, GC scope guard (S-4c's `NoGarbageCollectionScope`/`DisallowGarbageCollectionScope` wrappers), or `SequenceBound` borrow is held across `.await` or across a posted-task boundary. Async mutexes (`tokio::sync::Mutex`, `RwLock`, `async_lock::Mutex`) are banned (RFD 400) | `clippy::await_holding_lock`, `await_holding_refcell_ref` = deny; `await_holding_invalid_type` = deny with `await-holding-invalid-types` listing Izcal's guard types; `disallowed_types` for async mutexes |
| **A9** | **Message passing between sequences; short locks inside one.** State shared across sequences crosses as messages or immutable `Arc` snapshots. Where a lock is unavoidable (caches, the data-plane flush), it is a std or `parking_lot` mutex held for a bounded critical section, and each lock has a declared level; acquiring a lower level while holding a higher one does not compile (typed lock levels, Fuchsia's idea, Izcal's own implementation) | type system (lock-level traits) + `clippy::significant_drop_tightening` (nursery, warn) + review |
| **A10** | **Cancellation is structural.** Each task belongs to an owner (sequence, document, navigation, fetch); dropping the owner cancels its children, and the cancellation is delivered as a task, not by aborting mid-poll. Every async or multi-step API documents its cancel safety in a `# Cancel safety` doc section. `select!`-style macros are banned outside `hg-substrate-*`; inside, they use `biased;` (deterministic branch order) and never poll a `&mut` future with an `await` in a branch (futurelock) | `disallowed_macros`: `tokio::select`, `futures::select`, `futures::select_biased` (allowlisted in substrate) + hook: `# Cancel safety` section required on `pub async fn` and on functions returning `impl Future` + review |
| **A11** | **Atomics carry a written ordering argument.** Every atomic operation has a comment naming the happens-before edge it relies on; `Relaxed` only for counters and statistics; lock-free structures live only in crates listed in `unsafe-inventory.toml` and have loom or shuttle tests | hook: atomic `Ordering::` without an `// ordering:` comment on the line or line above + G5 inventory + review |
| **A12** | **Thread affinity is in the type.** Objects bound to a sequence or to the `cppgc` heap's owning thread are `!Send` (or wrapped in a `SequenceBound<T>` handle), so the compiler rejects moving them. Futures and tasks that do cross threads must be `Send` | `clippy::future_not_send` (nursery) on crates whose futures cross threads + type system + `clippy::arc_with_non_send_sync` = deny |
| **A13** | **Deterministic phases.** Fork/join phases read immutable inputs and write disjoint, index-addressed outputs; results are gathered in index order, never completion order; reductions use a fixed block decomposition independent of thread count; no floating-point reduction whose grouping depends on scheduling | DET-1 (parallel vs sequential hash, Stage 1 §3.3) + `disallowed_methods` on Rayon `sum`/`reduce`/`fold` over float iterators in behavior crates (hook, since clippy cannot see the element type) + review |
| **A14** | **Nondeterminism inventory for dependencies.** Every L0 dependency is listed in `nondeterminism-inventory.toml` stating whether it spawns threads, reads clocks, uses OS randomness or `RandomState`, or does IO, and how Izcal neutralizes it (pass our pool, pass our clock, wrap, or "accepted: outside simulation"). New dependencies cannot merge without an entry. The DST farm's rerun-same-seed meta-test (S2's method) is the check | hook (every `cargo metadata` package has an entry) + DST farm meta-test (two runs, same seed, byte-identical trace) |
| **A15** | **Future size.** Futures stay below 16 KiB (clippy default); larger state is boxed explicitly | `clippy::large_futures` = warn |
| **A16** | **No silently dropped work.** Task handles and futures are `#[must_use]`; dropping one with `let _ =` is an error | `clippy::let_underscore_future` = deny; rustc `let_underscore_lock` (deny by default); `unused_must_use` = deny |

### S. Code structure

| ID | Rule | Enforced by |
|---|---|---|
| **S1** | **Flat layout.** Every crate lives at `crates/<crate-name>/` with folder name = package name; root `Cargo.toml` is a virtual manifest; internal crates `publish = false`, `version = "0.0.0"`; automation in `xtask` | hook (path = name; `publish = false` unless listed in a `published-crates` list) |
| **S2** | **Layers.** G1–G9 stand. Add: a crate may depend only on its own or lower layers; proc-macro crates (`cargo metadata` target kind `proc-macro`) only from an allowlist; no crate in L1–L3 depends on `serde` derive or `syn` (keeps the base of the graph fast to build, 7.2) | graph |
| **S3** | **Split by ownership and layer, not by size.** A new crate needs a reason: a layer boundary, a G-rule, an oracle (G1/G2), a separate owner, or a build-time win shown with `cargo build --timings`. No chains of thin re-export crates | review (+ timings report in T1) |
| **S4** | **Visibility.** Default private. `pub` only for items reachable from the crate root; `pub(crate)` otherwise. Choose rustc `unreachable_pub` and do **not** enable `clippy::redundant_pub_crate` (they conflict) | rustc `unreachable_pub`, `unnameable_types` = warn |
| **S5** | **Public API surfaces.** Internal crates have no semver promise; their public API is whatever compiles. Crates that are published or form the embedder/plugin API carry `cargo-semver-checks` in CI and sealed traits (C-SEALED) and private fields (C-STRUCT-PRIVATE) by default | CI tool on the published list + review |
| **S6** | **Features are additive and few.** No feature removes behavior or changes web-observable behavior except the ones the specs define (`ai`, G4). No mutually exclusive features; use runtime options or separate crates. No negative names (`no-*`). Because Cargo unifies features per invocation (8.3), CI builds each `hg-*-main` binary in its own invocation and runs `cargo hack --each-feature` on library crates | `clippy::negative_feature_names`, `redundant_feature_names` = deny; deny `bans.features` for critical deps; graph (G4); CI matrix |
| **S7** | **No build scripts by default.** `build.rs` is allowed only for crates in an allowlist (the `v8` fork and platform FFI crates), for workspace crates and dependencies alike. Codegen output (`hg-webidl-codegen`, `hg-ipc-codegen`) is **checked in** under `crates/<crate>/src/generated/`, regenerated by `cargo xtask codegen`, and a T0 test fails if regeneration produces a diff. Generated files start with a header naming the generator and input (and carry G9 anchors) | deny `build.allow-build-scripts` and `build.executables = "deny"` + T0 freshness test |
| **S8** | **Dependency policy.** All dependencies vendored for release; `cargo-vet` audits; deny `sources` = crates.io and the project's own git only; `licenses` = an MIT-compatible allow list; `advisories` deny; `wildcards = "deny"`; `multiple-versions = "warn"` with a reviewed `skip` list; no dependency version newer than 14 days† without a security exception (hook until `-Z min-publish-age` stabilizes) | cargo-deny all four checks + `cargo vet` + hook on `Cargo.lock` publish dates |
| **S9** | **Toolchain and MSRV.** The pinned toolchain is the MSRV; `rust-version` set in `[workspace.package]` and inherited; edition 2024 (resolver 3) | hook (`rust-toolchain.toml` and `rust-version` agree) |
| **S10** | **One lint configuration.** All lints live in `[workspace.lints]`; every crate has `[lints] workspace = true`; local `#[allow]` must use `#[expect(..., reason = "...")]` or carry a reason | hook (each manifest inherits) + `clippy::allow_attributes_without_reason` = deny |
| **S11** | **Size limits.** Functions ≤100 lines (clippy default; TigerBeetle uses 70) as a warning in hand-written code; files ≤2000 lines† hand-written (rustc tidy uses 3000) with a per-file suppression that must give a reason; generated code exempt | `clippy::too_many_lines` = warn; hook for file length |
| **S12** | **Module files.** One style: `foo.rs` + `foo/` (ban `mod.rs`) | `clippy::mod_module_files` = deny |
| **S13** | **Documentation.** Every public item documented; every crate has crate-level docs stating its layer, its G-rules, and its threading model (which sequence or phase it runs on); fallible and panicking public functions document `# Errors` / `# Panics`; async/multi-step APIs document `# Cancel safety` (A10); intra-doc links must resolve | rustc `missing_docs` = deny (generated code gets docs from the generator); `rustdoc::broken_intra_doc_links`, `private_intra_doc_links` = deny; `rustdoc::missing_crate_level_docs` = deny; `clippy::missing_errors_doc`, `missing_panics_doc` = warn |
| **S14** | **Naming.** RFC 430 casing; crates `hg-<area>`; conversions `as_`/`to_`/`into_`; sequences named `"<process>.<component>"` (the `&'static str` in `Executor::sequence`) so traces and DST logs are greppable | rustc default style lints + hook for crate prefix + review |
| **S15** | **No unused dependencies.** | rustc `unused_crate_dependencies` (noisy with dev-deps; may need `cargo-machete`/`cargo-udeps` instead) |

### What stays review-only

Futurelock-shaped code inside the substrate (A10), the correctness of atomics arguments (A11), whether a new crate is justified (S3), whether a dependency's nondeterminism entry is true (A14, checked by the DST meta-test only after the fact), and blocking work hidden behind an innocent-looking call (A7, partly caught at run time).

---

## 12. Flagged claims

| Claim | Status |
|---|---|
| Quantum DOM was abandoned *because of* Spectre/Fission | **Inference.** Primary sources show it was never enabled, task labeling was incomplete, it cost time in profiles, and it was removed in Firefox 66 (bug 1485216). No maintainer statement read here links the removal to Spectre. |
| Bug 1350432 comment that the preemption pref made Firefox "unusable" | Search snippet only (Medium). |
| tokio 1.0 in December 2020; `std::thread::scope` in 1.63 (August 2022); Rust 1.75 shipped 2023-12-28 | From memory; not re-read (Medium). |
| Chromium's `ThreadPool` rename and date | Not verified (Medium). |
| WebKit thread-safety-analysis adoption breadth | Mailing list and bugs, 2021; current coverage unknown. |
| Fuchsia `lock_order` gives compile-time deadlock freedom | Module doc claim; mechanism not read (Medium). |
| Cilk reducers give serial-equivalent results | Paper not read (Medium). |
| rustc tidy 3000-line limit applies to `.rs` files | Code reading suggests yes; header comment says non-Rust; Medium. |
| Oxide RFD 400/609 publication dates | Not shown on the pages; omicron#9259 dates the incident (2025-10-19). |
| arrayref download counts (2,285 vs 245M) | Sources measure different things (malicious version vs all versions). |
| Ryhl's 10–100 µs budget | A rule of thumb for servers; A7's 100 µs† is a placeholder until Izcal's performance lab measures task-length distributions. |
| HashDoS concern for seeded hashers | Design inference: a fixed seed is predictable; A2 keeps production seeds random. Not measured. |
| Stylo never touches `STYLE_THREAD_POOL` when the embedder passes a pool | Grep of stylo-0.22.0 found no other reference outside `global_style_data.rs`; not tested at run time. |
