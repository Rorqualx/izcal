# Izcal coding standard

*Mandatory coding standard for Izcal's own Rust code, for engineers and AI agents. Date: 18 October 2026. Status: draft 0.2. Research notes: `research_notes/Izcal coding standard/` (`errors_types.md`, `async_structure.md`, `testing_performance.md`, plus `factcheck.md` and `critique.md`). Relates to: HGBA-2 M-1, P1, P2, P6, Section 4.8, S-4c, S-4e, I-1, I-2; HGBA-3 B1, B2, B3, DET-1, DET-2, VER-1, EQ-1, DST-1, ORC-1, INC-3; Stage 1 design Sections 1, 2.2 (G1–G9), 3, 4, 5.2, 5.3, 7.1–7.7, 8, 13.3 (RES-3, RES-4), D-S4, D-S10; DevTools spec G12; security scope draft 0.2: IPC-R1 to IPC-R17, FFI-R1 to FFI-R4, NET-R3, SH-R5, SEC-3, SEC-6, SEC-7, SEC-11, SEC-12, SEC-14, SEC-27, SEC-28, SEC-34, SEC-35; `CONTRIBUTING.md`; `tools/checks.sh`; `tools/deny.toml`.*

*Revision 0.1 (18 October 2026): first draft.*

*Revision 0.2 (18 October 2026): draft 0.1 was independently fact-checked (about 400 items, with every appendix config run on rustc and clippy 1.99.1 and cargo-deny 0.20.2; `factcheck.md`) and design-critiqued (37 findings: 8 high, 18 medium, 11 low; `critique.md`), and this revision applies both. Fact-check: tier O and T are exempted from G3 by generated `allow` levels and a per-crate `clippy.toml`, not a crate-level `#![expect]` that would fail CI, and tier O keeps every non-G3 ban; CS-V8-1 needs a new range binding in the fork; CS-ER-17 and CS-AS-4 get `ast-grep` checks because their lints miss the rule; a `doc` job makes the rustdoc lints run; a hook fails on `clippy.toml` warnings; the build-script allowlist is measured on a real L0 dependency set (65 crates); the tier S conflict with Stage 1 is an open question; eleven wrong external facts, the missing `std::fs`/`std::env` bans, the edition-2024 defaults and the † markings are corrected. Critique: the async model is decided (CS-D8); queues are split into web-visible, cross-process and in-process kinds; the token-cell decision gains soundness exit criteria; finalization gets a rule (CS-V8-9); T0 gets per-job budgets; Phase 1 is split so the standard is off M1's critical path; tier P gets a sanctioned `invariant!` path and a wrapping-arithmetic ban; a new section protects the gate from the agents it governs (CS-GA). The rules are cut from 100 to 65, with every retired ID mapped (Appendix C). Cross-references follow security scope draft 0.2 (PR #8), which conflicts with CS-D1 on the network service's panic strategy (Section 6.3). Every config block in Appendix A was re-run in a scratch workspace (A.7). Appendix B gives the disposition of every finding.*

Izcal's specs say what the engine must do. Nothing yet says how Izcal code fails, how its types prevent misuse, how it uses threads, how crates are laid out, what a test must prove, how a change shows it does not slow the engine down, or how the agents that write much of the code (Stage 1 Section 7.2 describes AI-assisted work as the norm) are kept from weakening the checks. A rule that a compiler, a generator or a hook checks is worth far more than one that depends on a reviewer noticing, so this standard states **65 rules**, each with a mechanism: a named lint, a `clippy.toml` or `deny.toml` key, a generator check, an `hg-graph-lint` rule, an `ast-grep` hook, a CI job, or an explicit "review only". The decisions that matter most: (1) **shipping and simulation binaries abort on panic**, and **declared crate trust tiers** decide where a panic is allowed at all: none in shared processes, where invariants go through one `invariant!` path and arithmetic is checked, never wrapped; (2) **errors are closed, typed values**, and the generators own the dangerous seams: bad IPC messages, web exceptions, V8 wrapper tags, JS conversions and GC finalization; (3) **the substrate owns every thread, clock, random number and queue**, engine code is written as run-to-completion tasks, and `async` is confined to an allowlist (CS-D8); (4) **tests are deterministic and never retried**, regressions are caught per PR by a fails-before-fix test and a deterministic instruction-count gate, and **the gate's own configuration is owned, listed and never edited by the agent it checks**.

## The rules on one page (if you read nothing else)

Lint-enforced rules announce themselves through the compiler. This list is the decisions a writer makes and the rules no tool fully checks.

1. **Know your crate's tier** (`tools/tiers.toml`; Section 6.2). Tier P (shared processes) never panics on any input; tier R (per-site processes) may panic only on broken invariants.
2. **Errors are values; invariants are `invariant!`.** Anything from outside the code returns `Result`. A broken internal invariant in tier P calls `hg_types::invariant!(cond, "literal")`, never `Err(Internal)` and never `BadMessage` (CS-ER-1, CS-ER-3).
3. **Checked arithmetic in tier P; never wrap to silence a lint** (CS-TY-12).
4. **No borrow lives across a call that can run script or GC**, and nothing in a GC payload's `Drop` touches another GC object (CS-TY-10, CS-V8-9).
5. **Only the substrate touches the OS, threads, clocks or randomness**, in tests too (CS-AS-1).
6. **Work is a closure posted to a sequence.** No `async` outside the allowlist; no task blocks; queues are substrate primitives; a shared process never blocks on a peer (CS-AS-4, CS-AS-6, CS-AS-7).
7. **Never hand-write a V8 or IPC seam**: tags, unwraps, conversions, bad-message handling and finalizers are generated (CS-V8-1, CS-ER-6).
8. **Unsafe needs an inventory entry, a SAFETY comment, and Miri** (CS-TY-13, CS-TE-13).
9. **No page or user data in panics, logs, traces, errors or exceptions** (CS-ER-13, CS-ER-14).
10. **Parse at the boundary into types that cannot be wrong** (CS-TY-3).
11. **Deterministic tests; every fix has a test that fails before it**, and the gate never retries (CS-TE-3, CS-TE-5).
12. **Spec-first tests for AI-written algorithms**, in a session that has not seen the implementation (CS-TE-17).
13. **Never make the gate pass by editing the gate**, and every suppression reason states a checkable fact (CS-GA-1, CS-GA-2).
14. **Hot paths have budgets**: type sizes, allocations and instruction counts (CS-PF-1, CS-PF-3, CS-PF-4).

Everything else is detail on how these are applied and checked.

---

## 1. The problem and who has it

**Who.** Three groups write Izcal code: the Stage 1 tracks (23–30.5 engineers, Stage 1 Section 8.3, plus the security scope's Security track), AI agents working from pinned specification text (Stage 1 Section 7.2, HGBA-3 Section 6), and reviewers who must accept or reject their output quickly. Each needs a rule it can follow without reading the architecture specs end to end, and a machine that says when it was broken. Agents need one more thing: a gate they cannot quietly change.

**What the existing specs already decide.** The standard fits and refers to these; it does not restate them (Section 6.12 lists the rules it leaves to them).

| ID | What it fixes | Where this standard uses it |
|---|---|---|
| Process map (Stage 1 Section 1; security scope 6.3a) | Broker and services shared by all sites; renderers per site; simulation runs the whole browser in one OS process | Trust tiers (6.2), panic policy (CS-ER-2, CS-ER-3) |
| G1–G9 | Layered crate graph; G1–G8 enforced by `hg-graph-lint`, G9 by `hg-anchor`; G3 ambient-effect ban; G5 `#![forbid(unsafe_code)]` plus `unsafe-inventory.toml`; G6 V8 surface | CS-AS-1, CS-TY-13, CS-ST-2 |
| Stage 1 Section 3, HGBA-3 B1, B3 | Synchronous substrate traits plus `Executor::post(seq, Task, delay)`; sequences for the event loop, fork/join for phases | CS-AS-3, CS-AS-4, CS-AS-12 |
| Stage 1 Section 4, VER-1, IPC-R1–R17 | `.hgidl`, generated endpoints, Kani harness per message; identity from bound endpoints (IPC-R2); a bad message terminates the sender (IPC-R5); no process trusts another (IPC-R17); non-`.hgidl` payloads are a named list (IPC-R16) | CS-ER-3, CS-ER-6, CS-TY-3, CS-TY-5 |
| Stage 1 5.2–5.3, S-4c, SEC-11, FFI-R1–R4 | `cppgc` DOM, generational node IDs, generated write barriers and bindings; Rust↔V8 rules | CS-TY-10, CS-V8-1 to CS-V8-9 |
| Stage 1 Section 7, DET-1, DST-1, EQ-1, ORC-1 | T0–T3 (T0 ≤ 20 min), oracles, DST farm, fuzzing, perf lab | Sections 6.9, 6.10, A.5 |
| M-1, D-S10 | Original code only; dependencies used unmodified; Izcal builds its own, following its specs | Section 7; other projects are cited as evidence only |
| SEC-6, SEC-14, SEC-27 | Attack-surface diff approval; cargo-deny, cargo-vet, 7-day cooldown, a recorded decision per new crate; checks run from the base branch | CS-ST-7, CS-GA-1 |

**Success, measurably.** Every rule names its mechanism; by the end of Phase 2, at least 70%† of rules have a mechanism running in CI (CS-T3); T0 stays within 20 minutes with each CS job inside a published budget (CS-T9); `main` is lint-clean on all three OSes (CS-T1); suppression reasons sampled after the fact are true (CS-T13).

## 2. History: what came before and why it was built that way

| Year | Project | What it did | Why that way | Fate and why |
|---|---|---|---|---|
| ~2008–2015 (published 2016) | Midori error model | Bugs **abandon** the process; errors are checked; overflow and OOM are bugs by default (OOM recoverable where opted in); ignoring a return value is a compile error ([Duffy](https://joeduffyblog.com/2016/02/07/the-error-model/)) | Cheap isolated processes made abandonment affordable | Cancelled; the bug/error split survives as Rust's `panic`/`Result` |
| 2010 onward | Google test sizes | Tests classified by allowed resources, enforced by the runner ([SWE at Google ch. 11](https://abseil.io/resources/swe-book/html/ch11.html)) | Nondeterminism poisoned a shared CI | Survives: hermeticity is enforced, not requested |
| ~2010–2021 | FoundationDB Flow | All nondeterminism behind interfaces, simulated cluster; avoided dependencies the simulator could not see ([FDB paper](https://www.foundationdb.org/files/fdb-paper.pdf)) | Distributed bugs did not reproduce | Survives as DST; its limit (dependencies) is Izcal's limit too |
| 2015 | RFC 560 | Overflow is a program error, checked in debug, wrapping in release ([RFC 560](https://rust-lang.github.io/rfcs/0560-integer-overflow.html)) | Cost in optimized builds | Default; Android turns checks on ([AOSP](https://source.android.com/docs/setup/build/rust/building-rust-modules/overview)) |
| 2015 | Servo experience report | Foreign values (SpiderMonkey's GC, custom allocators) were "one of our largest areas of unsafe code"; custom rooting lints ([arXiv 1505.07383](https://arxiv.org/abs/1505.07383)) | The GC sits outside Rust's guarantees | Later a `CanGc` token exposed borrow hazards ([Servo book](https://book.servo.org/design-documentation/script/garbage-collection-and-refcell.html)) |
| 2016–2019 | Firefox Quantum DOM | Cooperative scheduling inside one process; labeled tasks could be preempted, unlabeled ones lost that ([Mozilla Hacks](https://hacks.mozilla.org/2017/06/an-inside-look-at-quantum-dom-scheduling/)) | Memory cost of more processes | The unused CooperativeThreadPool and Scheduler were removed in Firefox 66; the LabeledEventQueue, which was in use, showed up in profiles ([bug 1485216](https://bugzilla.mozilla.org/show_bug.cgi?id=1485216)) |
| 2016 onward | Chromium sequences | "Prefer sequences to physical threads"; locks discouraged; `<chrono>`, the thread library and `<random>`'s engines banned ([threading](https://chromium.googlesource.com/chromium/src/+/main/docs/threading_and_tasks.md); [C++ features](https://chromium.googlesource.com/chromium/src/+/main/styleguide/c++/c++-features.md)) | Thread safety from serial queues, with an in-house replacement for each ban | Current; the model B3 adopts |
| 2017 | Firefox test-verify | New or changed tests run 10 + 5 times, then in chaos mode ([docs](https://firefox-source-docs.mozilla.org/testing/test-verification/index.html)) | New tests were the main source of new intermittents | Current |
| 2017 | V8 retires Octane | Octane-tuned optimizations "often had a detrimental effect on real-world scenarios" ([V8 blog](https://v8.dev/blog/retiring-octane)) | Microbenchmark overfitting | Real workloads over microbenchmarks |
| 2017–2021 | Fallible allocation | Firefox's MP4 parser moved to fallible allocation; Torvalds: panicking allocation "must simply not be used in the kernel" ([LKML](https://lkml.iu.edu/hypermail/linux/kernel/2104.1/09104.html)); `try_reserve` stable in 1.57 ([Rust 1.57](https://blog.rust-lang.org/2021/12/02/Rust-1.57.0/)) | Attacker-sized allocations became crashes | Standard practice for hostile sizes |
| 2019 | rustc-perf `instructions:u` | Small changes "(e.g. 0.3%) can be reliably detected with instruction counts" ([internals](https://internals.rust-lang.org/t/what-is-perf-rust-lang-org-measuring-and-why-is-instructions-u-the-default/9815)) | Wall time too noisy per PR | Survives; the Valgrind harness is now `gungraun`, Linux and FreeBSD only ([docs.rs](https://docs.rs/crate/gungraun/)) |
| 2023 | Chromium flake study | LUCI reruns up to 5 times; flaky tests revealed more than a third of regression faults ([Haben et al.](https://arxiv.org/pdf/2302.10594)) | Retries hide real failures | The gate must not retry |
| 2024-03 | Deno CVE-2024-27934 | Raw pointer in `v8::External` gave use-after-free and code execution ([GHSA-3j27-563v-28wf](https://github.com/denoland/deno/security/advisories/GHSA-3j27-563v-28wf)) | Lifetimes erased at the V8 boundary | Pattern to forbid (FFI-R1) |
| 2024-09 | Rust 1.81 | A panic escaping `extern "C"` aborts; `#[expect]` stable ([release](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/)) | C-unwind soundness ([RFC 2945](https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html)) | Current |
| 2025-10 | Oxide futurelock | A task stopped polling a future that held a mutex queue place; every API request hung ([RFD 609](https://rfd.shared.oxide.computer/rfd/609); [omicron#9259](https://github.com/oxidecomputer/omicron/issues/9259), opened 2025-10-19) | `select!` over `&mut` futures plus a FIFO async mutex; an unfair mutex would not reliably prevent it either | No lint exists; the construct must be removed |
| 2025-11-18 | Cloudflare FL2 outage | `unwrap()` on an exceeded 200-feature limit took down a shared proxy, 11:20 to 17:06 UTC ([Cloudflare](https://blog.cloudflare.com/18-november-2025-outage/)) | A limit with no defined failure, on generated config treated as trusted, in a process every customer shares | Remediation: "Hardening ingestion of Cloudflare-generated configuration files in the same way we would for user-generated input" |
| 2026-08 | arrayref supply-chain attack | arrayref 0.3.10, internment 0.8.7 and append-only-vec 0.1.9 were republished depending on the typosquat `proc-macro1`, whose **build script** fetched a payload; each was live for about 86–107 minutes ([Rust blog](https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/)) | Build scripts run at build time with the builder's rights | Build scripts are code execution |

**Technology mismatches: assumptions that no longer hold for Izcal.**
- *"Recover from a panic by unwinding"* (Servo's pipeline recovery). Izcal's recovery unit is the OS process; V8 callbacks are `extern "C"` and abort on an escaping panic anyway; continuing after a bug in a process that holds shared state is what Midori argued against.
- *"Stability rules from the Rust API Guidelines apply."* Izcal is one workspace built together. Internal crates have no downstream users, so `#[non_exhaustive]` would only remove the compiler error that forces every `match` to handle a new variant ([`exhaustive_enums`](https://rust-lang.github.io/rust-clippy/master/index.html#exhaustive_enums)).
- *"Use tokio for async."* Izcal's executor is the substrate (B1). A second runtime is a second scheduler the simulator cannot see.
- *"Generated code goes in `OUT_DIR` via `build.rs`."* Build scripts are an attack surface and a build cost; checked-in generated code is reviewable ([matklad](https://matklad.github.io/2022/03/26/self-modifying-code.html)).
- *"Flaky tests are retried."* With a simulated clock, scheduler and network, a test outside `hg-substrate-os` has no legitimate source of nondeterminism.
- *"Wall-clock benchmarks guard performance."* Link order and environment size can flip a speedup into a slowdown ([Mytkowicz et al.](https://sape.inf.usi.ch/publications/asplos09)).
- *"Code review catches what the tools miss."* When agents write the code and are told to make CI green, the tool configuration is itself part of the change.

## 3. Today: the landscape and its gaps

**What Izcal has (read 18 October 2026).** `tools/checks.sh` runs from the git hooks and in CI: `conflict`, `size`, `binary`, `secrets`, `rustfmt`, `shellcheck`, `actionlint`, `mdlinks`, `revision`, `provenance`, `security` (an `awk` heuristic for `// SAFETY:`, risky patterns, zizmor, cargo-deny advisories only) and the AI `review` (diff cut at `IZCAL_REVIEW_MAX_LINES=4000`). **Clippy runs nowhere**, although Stage 1 Section 7.2 lists it in T0. No Izcal Cargo workspace exists; the Rust on disk is spike code (`m0/`, `cppgc-spike/`, `v8-cppgc-ext/`, three of which declare standalone workspaces) and the `m0/rusty_v8` fork submodule with its own workspace. The `commit-msg` hook checks the subject and adds `Hooks-Skipped`; it does not check `Spec:` trailers yet.

**What the stock tools cannot see** (measured, A.7):
- Clippy lints only workspace code. Stylo's lazy global thread pool, hash seeds inside dependencies and dependency panics are invisible to it.
- `disallowed-*` entries for crates the linted crate does not depend on produce **no diagnostic**. A misspelled path into a reachable crate, or a module path, produces a **configuration warning that does not fail `-D warnings`** (exit 0). Both gaps need checks of their own (A.5 `clippy-config`, A.6 canary).
- `disallowed-*` match functions and types, not modules or statics, so a ban on "`std::fs`" must list every item, and OS-specific modules (`std::os::*`) need a hook.
- No lint catches a `RefCell` borrow alive across a re-entrant call ([clippy has only the `await` variants](https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_refcell_ref)), futurelock, a panic in `Drop`, or `wrapping_*` used to satisfy `arithmetic_side_effects` (which accepts it).
- `ffi_unwind_calls` fires only on calls to `extern "C-unwind"` functions and only in unwinding builds; `async_fn_in_trait` passes its own suggested fix, `-> impl Future + Send`.
- The rustdoc lints run only under `cargo doc`, never under `cargo clippy`.
- Clippy on one OS does not see `cfg(target_os)` code for the other two.
- Cargo rejects `[lints] workspace = true` combined with crate-level keys, and `panic` in a `package` profile.
- `allow-unwrap-in-tests` and friends do not cover helper functions at the root of an integration-test crate.

## 4. First principles

**Users' real tasks.** A contributor needs to know, while writing a function: may this fail, and how; may this panic; which thread am I on; may I block or `await`; may I add this dependency; what test proves this; will this slow the engine. A reviewer needs the answers visible in the diff. An agent needs rules it can check itself before submitting, and a clear line it may not cross.

**Platform primitives.** rustc and clippy lints at chosen levels, `#[expect(lint, reason)]`, `clippy.toml` (looked up from each crate's directory upward, so a crate can carry its own), Cargo profiles, cargo-deny, the type system (newtypes, typestate, `!Send`, lifetimes), Izcal's generators (`hg-ipc-codegen`, `hg-webidl-codegen`), `hg-graph-lint` over `cargo metadata`, `ast-grep` rules in the hooks, `CODEOWNERS`, and the simulator.

**Izcal's principles that bind this standard.** HGBA-2 P1 (security and privacy of the user) outranks HGBA-2 P4 (performance); HGBA-2 P2 (a faster wrong answer is a regression); HGBA-2 P6 (evidence over aspiration); M-1; HGBA-3 B1 (determinism designed in from day one, because retrofitting it "is very hard").

**Assumptions kept and broken.**

| Assumption | Decision | Reason |
|---|---|---|
| Panic = bug, `Result` = error | **Keep** | Midori, Rust practice ([Gallant](https://burntsushi.net/unwrap/)), kernel guidelines ([docs.kernel.org](https://docs.kernel.org/rust/coding-guidelines.html)) |
| `unwrap` is fine for invariants | **Keep, narrowed** | Tier R as `expect("invariant: …")`; tier P only through `invariant!`. For untrusted senders, "never use CHECK() to validate the message content" ([Mojo](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)); IPC-R17 makes every peer untrusted |
| "Less privileged" is the test for trust | **Break** | IPC-R17: no process trusts another; tier P bans panics on all input |
| One lint level for the codebase | **Break** | A panic costs different things in a renderer, the broker and a test |
| `#[non_exhaustive]` and sealed-for-semver | **Break** for internal crates | One build, no downstream users |
| Locks are bad | **Keep, softened** | Messages between sequences by default; short locks are fine when cheap and checked ([WebKit](https://webkit.org/blog/6161/locking-in-webkit/)) |
| `async` is how Rust does concurrency | **Break** for engine crates | The substrate is closures on sequences (Stage 1 3.1); Polar Signals removed `async` from its core for DST ([blog](https://blog.polarsignals.com/blog/posts/2025/07/08/dst-rust)) |
| Parallel by default | **Break** | Serial sequences, explicit fork/join phases (B3). Swift's SE-0466 adds an *opt-in* per-module main-actor default; the language default stays `nonisolated` ([SE-0466](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0466-control-default-actor-isolation.md)) |
| Coverage percentage as a gate | **Break** | "Low to moderate" correlation with fault detection once suite size is controlled ([Inozemtseva and Holmes](https://cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf)) |
| Mandated assertion density (TigerBeetle: an *average* of at least two per function) | **Break** | Density targets invite assertions written to meet a count; assertions stay encouraged (CS-ER-1) |
| Many small crates speed builds | **Keep with care** | True for parallelism; false for deep chains and generics at crate boundaries ([matklad](https://matklad.github.io/2021/09/04/fast-rust-builds.html)) |
| The checker's configuration is trusted | **Break** | It arrives in the same PR as the code (SEC-27 runs checks from the base branch, but their config comes from the PR) |

## 5. Lessons learned

Ideas and evidence only; no code and no template is taken from any project (M-1).

| Project | What happened | Lesson | How Izcal applies it |
|---|---|---|---|
| Cloudflare FL2 (2025-11-18) | `unwrap` on an exceeded limit in a shared proxy | Limits need defined failure; generated config is input | CS-ER-3, CS-ER-7 |
| Cloudflare FL1 (2025-12-05) | A Lua nil-index error in the older proxy; the Rust FL2 code did not have the bug ([Cloudflare](https://blog.cloudflare.com/5-december-2025-outage/)) | Configuration changes are input whatever the language | CS-ER-7 |
| Chromium Mojo | `ReportBadMessage()` "does not stop execution"; a missing `return` was crbug 1285384 | A safety call a handler can forget will be forgotten | CS-ER-6: the generated dispatcher reacts |
| Chromium, Firefox, Firecracker | Abort in shipped Rust: Chromium `-Cpanic=abort` (official builds `immediate-abort`, plus `-Zpanic_abort_tests`; host tools unwind) ([BUILD.gn](https://chromium.googlesource.com/chromium/src/+/main/build/config/compiler/BUILD.gn)); Firefox dev and release ([Cargo.toml](https://github.com/mozilla-firefox/firefox/blob/main/Cargo.toml)); Firecracker ([Cargo.toml](https://raw.githubusercontent.com/firecracker-microvm/firecracker/main/Cargo.toml)) | Production browsers do not rely on unwinding | CS-ER-2 |
| neqo, s2n-quic, Firecracker | Restriction lints, exempted in tests ([neqo](https://github.com/mozilla/neqo/blob/main/Cargo.toml); s2n-quic's `.clippy.toml`) | Per-crate lint tables are the mechanism for tiers | Section 6.2, A.2 |
| Deno | Raw pointers in `External`; resizable ArrayBuffers shrunk during async operations ([GHSA-c25x-cm9x-qqgx](https://github.com/denoland/deno/security/advisories/GHSA-c25x-cm9x-qqgx)) | The V8 boundary needs type design, not care | CS-V8-3, CS-TY-10 (FFI-R1, FFI-R2) |
| Servo | GC tracing panicked on live `RefCell` borrows; `CanGc` made the hazard visible | Make "may run script or GC" part of the signature | CS-TY-10 gates the borrow itself |
| Firefox crash reports | Rust panic reasons went into the crash-reason annotation (bug 1275780) and leaked web content (bugs 1502477, 1507465) | Panic text is telemetry | CS-ER-14 |
| Oxide RFD 400 and 609 | Async mutexes and `select!` over partly polled futures hung every request | Remove the construct | CS-AS-4, CS-AS-8 |
| S2 DST on tokio | Determinism leaked through third-party crates and `HashMap` ([S2](https://s2.dev/blog/dst)) | The leaks are in code you did not write | CS-AS-2, CS-AS-13 |
| rustc UI tests | Inline annotations alongside snapshots, because ".stderr files are usually auto-generated" ([dev guide](https://rustc-dev-guide.rust-lang.org/tests/ui.html)) | A golden file is never the only oracle | CS-TE-12 |
| LLM-written tests | Oracles encode actual, not expected, behavior, though they still out-detect EvoSuite's ([arXiv 2410.21136](https://arxiv.org/abs/2410.21136)) | Tests come from the spec and must fail without the change | CS-TE-3, CS-TE-17 |
| Chromium binary-size bot | A `Binary-Size:` footer bypasses the bot ([docs](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/speed/binary_size/android_binary_size_trybot.md)) | Budgets work when overriding them is explicit and reviewed | CS-PF-4, CS-PF-8, CS-GA-3 |
| WebKit, Firefox perf sheriffs | WebKit: zero tolerance, back out or fix at once ([WebKit](https://webkit.org/performance/)); Firefox: daily triage, a 1-business-day developer response ([sheriffing](https://firefox-source-docs.mozilla.org/testing/perfdocs/perf-sheriffing.html)) | The default action must be cheap and agreed in advance | CS-PF-5 |
| arrayref | A typosquat's build script ran a payload | Build scripts are code execution | CS-ST-7 |

## 6. Design: the standard

### 6.1 How to read and apply a rule

- **MUST** is mandatory and checked; **SHOULD** applies unless the PR says why not.
- **Lint levels.** CI runs `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, so every lint at `warn` or `deny` fails CI. `deny` also stops a local `cargo check` for **rustc** lints; `cargo check` does not run clippy lints at all. Clippy's default groups stay on.
- **Exceptions** are marked in code: `#[expect(clippy::<lint>, reason = "<rule ID>: <checkable fact>")]` on the smallest item. Never `#[allow]` (`clippy::allow_attributes`, deny; this was CS-ER-4). `#[expect]` warns when the lint stops firing, so stale suppressions fail CI. Where a lint fires on only one OS or feature set, use `#[cfg_attr(target_os = "…", expect(...))]` (verified, A.7). In tier P, internal invariants use `invariant!` instead of panic-lint expectations (CS-ER-3). Hook rules accept `// cs-allow(<ID>): <reason>` on the line, and the hook fails on a marker whose line no longer matches its rule. Every new exception of any kind is listed in the PR (CS-GA-3).
- **A reason states a fact someone can check**: what holds, and where it is established ("index < len: checked by the `if` on the line above"). "Safe", "needed", "invariant" or "false positive" alone are not reasons (CS-GA-2).
- **Enforcement keywords:** **lint** (A.1–A.2), **clippy.toml** (A.3), **deny** (A.4), **gen** (`hg-ipc-codegen`, `hg-webidl-codegen`), **graph** (`hg-graph-lint`), **hook** (an `ast-grep` or shell rule in `tools/checks.sh`), **CI** (a named job, A.5), **owner** (`CODEOWNERS` approval), **review**.
- **Generated code** follows its crate's tier and CS-ST-17.
- **Existing rules win.** Where this standard names a G-rule, IPC-R, FFI-R, S-4c or SEC target, the original is normative and this document only supplies the mechanism.

### 6.2 Trust tiers

A crate's tier is **declared** in `tools/tiers.toml`. `hg-graph-lint` computes the strictest tier of any binary that links the crate and **fails when the declared tier is laxer**, with "crate X is now linked by Y; declare tier P in its own PR". Tiers never change silently, and a tier change is a gate-file change (CS-GA-1). The crate lists below are expected values; `tiers.toml` is normative.

| Tier | Contains (expected) | Why | Panics | Lint table | G3 |
|---|---|---|---|---|---|
| **P** shared | Binaries for the broker, network, storage, GPU and ML services, agent host, automation gateway, updater and crash handler (security scope 6.3a), and what they link: `hg-navigation`, `hg-policy`, `hg-profile`, `hg-update`, `hg-net`, `hg-storage`, `hg-gpu`, `hg-content-block`, `hg-ipc`, `hg-ipc-gen`, `hg-types`, `hg-substrate`, and `hg-shell` if the broker links it (the broker owns the shell UI, Stage 1 Section 1) | A panic kills a process every site depends on | None reachable from any input (CS-ER-3); invariants via `invariant!` | Generated: base + tier P additions (A.2) | Applies |
| **R** per site | `hg-renderer-main`, `hg-utility-main` (CS-D7), the shell, prompt, DevTools and replay-host renderers, and what only they link: `hg-dom`, `hg-bindings`, `hg-script`, `hg-style`, `hg-layout`, `hg-paint`, `hg-semantic`, `hg-semantic-repair`, `hg-tree-traits`, `hg-dataplane`, `hg-projection`, `hg-devtools`, `hg-media`; and `hg-substrate-sim` (Q-CS-1) | A panic ends one site's process, which content can already do | `expect("invariant: …")` for internal invariants; web-reachable panics are reliability bugs found by fuzzing | Base (`[workspace.lints]`) | Applies |
| **O** OS substrate | `hg-substrate-os` only | The one crate allowed ambient OS effects (G3) | As tier P | Tier P table; its own generated `clippy.toml` drops only the G3, thread-pool, `select!` and bounded-channel entries, so `catch_unwind`, unbounded channels, async mutexes and `v8::External::new` stay banned (verified, A.7) | **Exempt** by that file |
| **T** tools and test infrastructure | `hg-webidl-codegen`, `hg-ipc-codegen`, `hg-graph-lint`, `hg-anchor`, `hg-wpt`, `hg-corpus`, `hg-bench`, `hg-dst` (the runner), `xtask`; `hg-lint-canary` uses the base table | Runs on developer and CI machines; never linked into a shipping binary (graph) | Allowed | Generated: base with `disallowed_*`, `unwrap_used`, `todo` and printing at `allow` (A.2) | **Exempt** by those `allow` levels |
| **Test code** in any crate | `#[cfg(test)]` modules, `tests/it/`, benches, fuzz targets | Cannot reach production | `unwrap`, `expect`, `panic!`, indexing allowed (`allow-*-in-tests`) | The crate's table | **Applies** |

Generators are tier T for panics, but their output is held to CS-ST-17 and their changes need the reviews CS-GA-1 names.

### 6.3 Conflicts resolved

| Conflict | Sources | Choice | Why |
|---|---|---|---|
| **Abort vs unwinding test frameworks** | 0.1 "abort in every profile"; Cargo builds tests with unwind ([profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)); `#[should_panic]` and proptest shrinking need unwinding | **Shipping and simulation binaries abort; test builds unwind, and test frameworks may rely on it; production code never does** (`catch_unwind` banned outside tests). `dev` unwinds too, so a `cargo build` and a `cargo test` share dependency builds (measured: with `dev` aborting, `cargo test` recompiled every dependency) | One panic model per artifact kind, and no double build in daily work |
| **CS-D1 abort vs security scope 0.2 NET-R3** | IPC-R5 (0.2): "`panic = "unwind"` with a `catch_unwind` boundary per connection in the network service"; SEC-28: "a panic in one connection never ends the service" | **Owner decision CS-D1**, recommendation abort: the network service holds every site's cookies, caches and sockets, so state after a panic is unknown in exactly the process where corruption is shared. If the owner chooses unwind for the network service, the mechanism is in CS-D1 | Two specs cannot both be normative on this |
| **IPC-R5's "message-derived" wording** | 0.1 proposed replacing it with the tier P lint set | **Already changed** in security scope 0.2 (outright denies with a reviewed allowlist). This standard supplies the allowlist as a generated file (CS-ER-3) | Both specs now agree |
| **"Less privileged" in 0.1's CS-ER-3 and CS-ER-6 vs IPC-R17** | IPC-R17: "'Less privileged' is not used as a test anywhere" | Tier P never panics on **any** input; every bad message is the sender's fault, whoever sent it (CS-ER-6) | IPC-R17 is normative |
| **Tier S "never linked into a shipping binary" vs Stage 1 and DevTools** | Stage 1 M1 exit: processes "start under both substrates"; DevTools G12: `hg-replay-main` constructs an `hg-substrate-sim` instance | `hg-substrate-sim` ships in the replay host, so it is held to tier R and G3; the `hg-dst` runner is tier T. Whether the browser, renderer and network binaries link it is Q-CS-1 for the Stage 1 owner | Facts on disk, and a seeded `Entropy::crypto()` should not sit in a binary that touches the network |
| **Web-visible task queues vs "every queue is bounded"** | 0.1 CS-AS-6 (1024 tasks); HTML task queues and timers are unbounded | Web-visible queues are bounded by **memory accounting**, as HTML allows: "User agents may impose resource limitations on scripts, for example CPU quotas, memory limits … the user agent may either throw a QuotaExceededError, abort the script without an exception, prompt the user, or throttle script execution" ([HTML](https://html.spec.whatwg.org/multipage/webappapis.html#killing-scripts)) (CS-AS-6) | A task count has no spec-conformant overflow action |
| **Wrapping arithmetic vs `arithmetic_side_effects`** | The lint accepts `wrapping_*`, `saturating_*`, `Wrapping` and `Saturating` (verified) | Banned in tier P without a `cs-allow` marker (hook, CS-TY-12) | A silent wrong length in the broker is worse than the abort it replaces |
| **`panic_in_result_fn` vs "assert your own state"** | The lint flags `assert!` in a `Result` function (verified) | Tier P asserts with `invariant!`, which the lint does not flag (verified); tier R keeps `assert!` | Removes the contradiction without a blanket exemption |
| **`#[non_exhaustive]`** | API guidelines, FIDL flexible types (for future members and unknown data) vs one-build workspace | None on internal types; no wildcard arms on Izcal enums in tier P and generated code (CS-TY-4). Published crates use it (CS-ST-14). IPC enums are closed (IPC-R4) | Exhaustive matching carries a new variant to every handler |
| **Test determinism vs G3** | G3; proptest seeds from OS randomness by default | G3 binds tests (clippy on all targets). Real OS, network or GPU only in `hg-substrate-os`'s tests, the perf lab and T2/T3 (CS-TE-2) | A flake outside the OS substrate is then a determinism bug |
| **G3 vs tools that need the OS** | G3 vs `hg-wpt`, `hg-graph-lint` | Tier T is outside G3 (never shipped, never simulated). Proposed as a G3 clarification (9.4) | G3 exists for engine determinism (B1) |
| **Instruction-count gate needs Linux; developers use macOS** | `gungraun` installs on Linux and FreeBSD only; SEC-27 (no self-hosted runner on `pull_request`) | GitHub-hosted Linux in CI; locally, `divan` wall-clock benchmarks that never gate (CS-D5) | Determinism over local convenience |
| **Dependency cooldown 14 vs 7 days** | async note; SEC-14 | **7 days**, SEC-14's number | One number, one owner |
| **RFD 400 vs RFD 609** | RFD 400 recommends `select!` over `&mut`; RFD 609 shows that shape causes futurelock | `select!` only in `hg-substrate-*`, with `biased;`, never over a `&mut` future with an `await` in a branch (CS-AS-4) | No lint finds it |
| **`unreachable_pub` vs `clippy::redundant_pub_crate`** | They contradict | `unreachable_pub` on; `redundant_pub_crate` stays off (nursery) | Only one can hold |
| **Function length 70 vs 100** | TigerBeetle; clippy default | 100 (`too_many_lines`); rustc's tidy suppresses its 3000-line file limit with `// ignore-tidy-file-filelength`, which needs no reason, but Izcal's file-length marker does | The default needs no configuration |
| **Hard size asserts vs rustc's advice** | rustc: "Don't copy this macro into your own code. Use a `#[test]` instead" ([docs](https://doc.rust-lang.org/stable/nightly-rustc/rustc_index/macro.static_assert_size.html)) | `const` asserts on 64-bit targets (CS-PF-1), deliberately against that advice | The toolchain and `Cargo.lock` are pinned, so a size change appears only in a bump PR, where the break is the signal |
| **`thiserror` vs no proc macros in L1–L3** | errors note; structure note | `thiserror` from L4 up and in tier T; L1–L3 hand-write `Display` and `Error` (CS-ER-10) | Proc macros stay off the base of the graph |
| **Workspace lints vs per-tier lints** | Cargo rejects mixing (measured) | Tier R inherits `[workspace.lints]`; P, O and T get generated full tables (CS-D9) | One source of truth without Cargo support for overrides |

### 6.4 Errors and panics (CS-ER)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-ER-1** | **Two channels, one policy per tier.** Conditions from outside the code (input, I/O, IPC, network, device loss, exceeded limits) MUST be `Result` values; panics and assertions mean a broken internal invariant. **Tier R:** invariants use `expect("invariant: <what holds>")` or `assert!`; `unwrap` only in tests; a panic reachable from web content is a reliability bug with a regression test, not a security issue. **Tier P:** invariants use `hg_types::invariant!(cond, "literal")` or `invariant_unreachable!("literal")`, which call one `#[cold]` function that aborts with the literal; cheap ones stay on in release, expensive ones use `debug_assert!`. DST, fuzzing and T0 run with debug assertions (`checked` profile). No assertion density is mandated | Midori's abandonment vs errors ([Duffy](https://joeduffyblog.com/2016/02/07/the-error-model/)); "correct Rust programs don't panic" ([Gallant](https://burntsushi.net/unwrap/)); `CHECK()` "is acceptable and often preferred" when the sender is trusted ([Mojo](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)) | lint: `unwrap_used` (warn, R), tier P table; the `invariant!` definition carries the only panic-lint `#[expect]` (verified); CI: DST and fuzzers report panics | Test code |
| **CS-ER-2** | **Shipping and simulation binaries abort.** `release`, `checked` and every profile that builds a shipping or DST binary set `panic = "abort"`. Test builds unwind (Cargo forces it) and `dev` matches them. `catch_unwind` and `resume_unwind` only in test code. In `hg-substrate-sim` a simulated process crash is a scheduler action that drops that process's state, never a panic; a real panic aborts the seed, and `hg-dst` (tier T) runs **one seed per child OS process**, records the abort signature, seed, fault schedule and replay command, and continues with the next seed. The simulation binary's crash handler is the one `set_hook` site there | V8 callbacks abort on escaping panics anyway ([Rust 1.81](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/)); package profiles cannot set `panic` (measured); DST replay is exact | graph (profiles used by shipping and DST binaries abort); clippy.toml `disallowed-methods` (stays on in tier O); `hg-dst` | `#[should_panic]`, proptest; CS-D1 option (b) if chosen |
| **CS-ER-3** | **Tier P never panics on any input.** The tier P table denies `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `string_slice`, `get_unwrap`, `arithmetic_side_effects`, `unwrap_in_result`, `panic_in_result_fn` and the cast lints. Prefer `checked_*` plus `?`. A broken internal invariant goes to `invariant!`, **never** to an `Err` that a generated handler would turn into `BadMessage` (the broker's bug would be blamed on, and kill, a renderer). `cargo xtask exceptions` writes every tier P `invariant!` and panic-lint `#[expect]` site into the generated `tools/coding-standard/tier-p-exceptions.toml`, which is the "allowlist file reviewed like the `unsafe` inventory" of IPC-R5 and SEC-7 (b). Dependency panics and `Drop` are outside lints, so the Kani harness (SEC-7 (a)) and the sequence fuzzer (IPC-R14) are the real check; IPC-R16 payloads (WebRender display lists, inspector JSON, TUF metadata) are parsed by dependency code and covered by their fuzz targets | Cloudflare 2025-11-18; a compromised sender "can trigger a `CHECK()` to crash the receiving process" ([Mojo](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)); quinn-proto panics and a rustls pre-authentication allocation in 2026 (security scope 3.3) | lint (tier P table); CI `exceptions-fresh`; owner (CS-GA-1); gen (Kani); CI (T2 fuzzing) | `invariant!` sites, counted by CS-T2 and sampled by CS-T13 |
| **CS-ER-6** | **IPC failure is generated: a bad message kills the sender, and errors are closed enums.** Generated handlers return `Result<_, BadMessage>`; the generated dispatcher closes the endpoint and asks the broker to terminate the sender; no handler can report and fall through. This holds for every peer (IPC-R17); a bad message from the broker itself is a broker bug that the receiving service records and exits on. Each `.hgidl` protocol declares one closed error enum: no free text, addresses, paths or backtraces cross a process boundary; an unknown discriminant is a bad message | `ReportBadMessage()` "does not stop execution of the current function" ([Mojo](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)); stack traces "can unintentionally leak privileged information" ([Fuchsia FIDL](https://fuchsia.dev/fuchsia-src/development/api/fidl)); IPC-R4, IPC-R5 | gen | None |
| **CS-ER-7** | **Every limit has an error path and a fallback; configuration is input.** Every capacity, preallocation or cap returns a typed error when exceeded, and the caller defines the fallback (reject, shed with a counter, or keep the last known-good value). Filter lists, blocklists, feature-flag and driver-blocklist files, component and update manifests are parsed under tier P rules with size caps and a fuzz target; a parse failure keeps the last known-good version | Cloudflare's 200-feature limit and its remediation; TigerBeetle: "put a limit on everything" ([TIGER_STYLE](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)) | hook `cs-unwrap-on-try` (`.unwrap()`/`.expect(` on `try_*`, `checked_*`); CI: fuzz target per format plus an oversized and a corrupt file test; review | `cs-allow` marker |
| **CS-ER-9** | **Fallible allocation for attacker-sized buffers.** Allocations sized by IPC, the network, a file, media, fonts, images, WebGPU or a JS length use `try_reserve`-style APIs with an explicit cap; IDL fields declare maximum lengths; where a spec defines the failure, map to it | `std` aborts on allocation failure ([handle_alloc_error](https://doc.rust-lang.org/std/alloc/fn.handle_alloc_error.html)); SEC-28's bounded pre-authentication allocations | gen (IDL maxima; bounded allocation in Kani); CI (fuzzing under an RSS limit); review | Small fixed sizes |
| **CS-ER-10** | **Library-style, deterministic errors.** Dedicated enums near their unit of fallibility, `source()` chains, std-style `Display`. `thiserror` from L4 up and in tier T; `anyhow` and `Box<dyn Error>` only in tier T; no `map_err(|_| …)` dropping the source in tier P. Values DST compares contain no addresses, backtraces or hash order; backtraces are captured only by the crash handler | Per-operation error types ([Jewson](https://sabrinajewson.org/blog/errors)); anyhow is for applications and captures backtraces under `RUST_LIB_BACKTRACE` ([README](https://github.com/dtolnay/anyhow)) | graph (`anyhow` in tier T only; no proc macros in L1–L3); lint `map_err_ignore`, `error_impl_error`, `result_large_err` (128 B default); clippy.toml `Backtrace::capture` | Crash handler, with `#[expect]` |
| **CS-ER-13** | **One generated mapping from internal errors to web exceptions.** `hg-webidl-codegen` emits, per member, the conversion to the exception the spec algorithm throws, with a fixed `&'static str` message; no internal `Display` reaches script; every network failure becomes the Fetch network error; cross-origin script errors are muted | WebIDL ([§2.8](https://webidl.spec.whatwg.org/#idl-exceptions)); Fetch network error ([Fetch](https://fetch.spec.whatwg.org/#concept-network-error)); HTML muted errors; HGBA-2 5.4 (`PolicyDenied` looks like `Unsupported`) | gen; CI privacy suite (CS-T11) | None |
| **CS-ER-14** | **Diagnostics go through one facade and carry no page or user data.** Engine crates never print. Logs, traces, crash keys and telemetry go through Izcal's own facade in `hg-substrate` (built, following this rule, not adopted): levels and categories, zero cost when compiled out (an allocation test), and in simulation an ordered trace event recorded in scheduler order. The substrate installs the one `log` adapter per process, so dependencies that log (Stylo uses `log`) reach the facade. Page-derived values (`Url`, `Origin`, header values, DOM text) are wrapped in `hg_types::Sensitive<T>`, whose `Debug` and `Display` redact in release builds and in crash and telemetry paths; credentials and keys use `Secret<T>` with no `Debug`, zeroized on drop, never in IPC error enums or traces (SEC-34). Telemetry fields come from closed enums. Panic and assertion messages are string literals naming the invariant (`invariant!` enforces this by type). Types that cannot print safely implement `Debug` by hand, showing IDs only | Firefox leaked web content through Rust panic messages in crash annotations (bugs 1275780, 1502477, 1507465); crash dumps are filtered (SH-R5, SEC-35); D-S4 | lint `print_stdout`, `print_stderr`, `dbg_macro`, `missing_debug_implementations`; clippy.toml `log::set_logger`, `set_hook`; hook `cs-panic-format-args` and `cs-sensitive-debug` (`{:?}` of a `Url`/`Origin`/`HeaderValue` outside `Sensitive`); review | Tier T and tests |
| **CS-ER-17** | **No panic crosses FFI and no FFI error is silent.** Every `extern "C"` callback from V8 is generated; its body returns `Result`; the shim turns an error into a pending JS exception or a documented failure value. No hand-written `extern "C-unwind"` | RFC 2945; Chromium's FFI doc suggests splitting functions into a `Result` core and a thin wrapper ([ffi.md](https://chromium.googlesource.com/chromium/src/+/main/docs/rust/ffi.md)) | gen; hook `cs-c-unwind` (A.6, verified); lint `ffi_unwind_calls` catches calls in unwinding (test and dev) builds only | None |

### 6.5 Type design (CS-TY)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-TY-1** | **Newtypes for every ID and unit; generational where lifetimes cross.** Node, frame, navigable, site instance, process, endpoint, sequence, request and device-epoch IDs; CSS px vs device px; byte lengths vs counts. No raw integer as an ID in a public `hg-*` signature; IDs have no `Default` or public `From<u32>`. IDs that outlive their object are slot plus generation (`u32` + `u32`, Stage 1 5.2); lookup returns `Option`; a slot whose generation would overflow is retired, never wrapped, so IDs are never reused (IPC-R8) | C-NEWTYPE ([checklist](https://rust-lang.github.io/api-guidelines/checklist.html)); Mojo: use structured types ([Mojo](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/security/mojo.md)); generations detect staleness ([West](https://kyren.github.io/2018/09/14/rustconf-talk.html)) | hook `cs-primitive-id-param` (a heuristic: it cannot see types); CI: stale-ID property test (Stage 1 5.2, SEC-3 (d)) | Wire-format code converts at the boundary |
| **CS-TY-3** | **Parse at the boundary; constructors validate.** Every trust boundary (IPC, network, files, config, JS→Rust) produces refined types (`Origin`, `Site`, validated URLs and paths, bounded strings, `NonZero*`) with private fields; parsers, `try_new` or `TryFrom` are the only constructors; `new` never fails; `From` never panics; builders only for many optional fields, with `build()` returning `Result`. Privileged code never takes a `String` it later parses (IPC-R3) | Parse, don't validate ([King](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)); C-VALIDATE | gen (IPC, bindings); lint `fallible_impl_from`; review | None |
| **CS-TY-4** | **Closed, exhaustive types.** Enums instead of `bool` flags or several `Option`s; unions instead of optional-field combinations; no parallel arrays; no `Count`/`Last` sentinels; no `#[non_exhaustive]` on internal types; no wildcard arm on an Izcal-owned enum in tier P and generated code. An external enum (`io::ErrorKind`, wgpu and Stylo types) is matched in one adapter module per dependency with an `#[expect]` reason "external enum" | IPC-R3, IPC-R4; Section 6.3 | lint `fn_params_excessive_bools`, `struct_excessive_bools`, `wildcard_enum_match_arm` (tier P); hook `cs-non-exhaustive`; gen | Published crates (CS-ST-14) |
| **CS-TY-5** | **Authority is in the type.** Process site-lock, endpoint binding, navigation commit and capability attenuation use typestate with sealed markers, so a transition consumes the old state; traits that mint or carry authority (handle kinds, rights, `GarbageCollected` wrappers, the write side of tree-access traits) are sealed; `CallerIdentity` can be constructed only in `hg-ipc`, and no authorization API takes `TargetOrigin` or `NavigationUrl` (IPC-R2, SEC-3 (a)) | Typestate ([Biffle](https://cliffle.com/blog/rust-typestate/)); C-SEALED; HGBA-2 I-1; IPC-R1 (rights only narrow) | review; gen for `CallerIdentity` | None |
| **CS-TY-9** | **`Send` and `Sync` are part of the contract.** Types tied to an isolate, a `cppgc` heap or a sequence are `!Send`/`!Sync` by construction, with a compile-fail test; cross-sequence access goes through `SequenceBound<T>`; `unsafe impl Send/Sync` only in inventoried modules with `// SAFETY:` | C-SEND-SYNC; S-4c (HGBA-2: the `cppgc` heap's thread rules); Chromium's sequence affinity | lint `non_send_fields_in_send_ty`, `arc_with_non_send_sync`; CI compile-fail tests; G5 | None |
| **CS-TY-10** | **No borrow survives a call that can run script or GC.** Functions that may re-enter script or trigger `cppgc` GC take `&mut ScriptToken`. In `hg-dom`, `hg-bindings` and `hg-script`, script-reachable mutable state lives in `TokenCell<T>`, whose borrows are tied to the token, so a re-entrant call cannot compile while a borrow lives; `RefCell` is not used there, and tier P code that uses `RefCell` uses `try_borrow_mut` and returns an error. GC objects are reached through handles and stable IDs, never references held across script or GC; phase-pure work (style, layout, paint) may borrow immutable inputs for the phase. The pointer from `Object::unwrap` becomes, in generated code, a `!Send` guard borrowed from the current scope. Rust never holds a slice into an `ArrayBuffer` or `SharedArrayBuffer` across a call that can run script or GC: it copies, or pins and re-checks length and detachment (FFI-R2). The soundness conditions are CS-D3's M2 exit criteria | Servo: a `borrow_mut()` alive across a `CanGc` call is "a panic waiting to be triggered" ([Servo book](https://book.servo.org/design-documentation/script/garbage-collection-and-refcell.html)); rusty_v8: the pointer must stay "on the stack" ([docs.rs](https://docs.rs/v8/latest/v8/struct.Object.html)); Deno CVE-2023-28445; GhostCell (Yanovski et al., PACMPL 5 ICFP, Article 92, [project](https://plv.mpi-sws.org/rustbelt/ghostcell/)) | rustc borrow checker; gen (bindings pass the token); clippy.toml `await-holding-invalid-types`; hook `cs-refcell-in-dom`; CI: GC-stress and re-entrancy fuzzing, SEC-11 (b) suite | Fields never reachable from script, with `cs-allow` |
| **CS-TY-12** | **Explicit integer widths; checked, never wrapped, arithmetic in tier P.** `u32` for slots and generations; `usize` only for in-memory indexing. In tier P and generated code: no `as` between integers (`From`/`TryFrom`); arithmetic uses `checked_*` plus `?`; `wrapping_*`, `saturating_*`, `overflowing_*`, `Wrapping` and `Saturating` need `// cs-allow(CS-TY-12): modular by design` (hashes, sequence numbers, checksums). Integer unit newtypes in tier P expose `checked_add` and similar, not `Add`; float-based unit types (`CssPx`, `DevicePx`) are listed in `arithmetic-side-effects-allowed`. Tier P builds with `overflow-checks = true` in release; tier R per CS-D2 | RFC 560; Android checks overflow; `arithmetic_side_effects` accepts wrapping (verified) | lint (tier P table); hook `cs-wrap-in-p` (verified); clippy.toml `arithmetic-side-effects-allowed` (verified); per-package profile | `cs-allow` with the reason |
| **CS-TY-13** | **Encapsulated `unsafe` with written contracts.** Non-inventoried crates carry `#![forbid(unsafe_code)]` (G5). Inventoried code sits in modules listed in `unsafe-inventory.toml`, each with `#[expect(unsafe_code, reason = "G5: <entry>")]` (verified), and the graph lint checks the module paths match. `// SAFETY:` above every block states the precondition, why it holds here, and what would break it; `# Safety` on every `unsafe fn`; one unsafe operation per block; no unsafe API behind a safe wrapper. Unsafe diffs need the inventory owner (`CODEOWNERS` routed from the inventory) | Chromium requires a safety comment for all unsafe code ([unsafe.md](https://chromium.googlesource.com/chromium/src/+/main/docs/rust/unsafe.md)); edition 2024 already rejects unsafe attributes and `extern` blocks without `unsafe` | graph (G5, module paths); lint `unsafe_code`, `unsafe_op_in_unsafe_fn` (deny), `undocumented_unsafe_blocks` (deny), `multiple_unsafe_ops_per_block`, `missing_safety_doc`, `ptr_as_ptr`, `ref_as_ptr`, `transmute_undefined_repr`; owner | `undocumented_unsafe_blocks` replaces the `awk` heuristic for workspace Rust |

### 6.6 The Rust↔V8 boundary and FFI (CS-V8, CS-FFI)

The FFI crates are `hg-dom`, `hg-bindings` and `hg-script` (G6). These rules are the mechanism for SEC-11 and FFI-R1 to FFI-R4.

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-V8-1** | **Generated wrapper tags, checked twice.** `hg-webidl-codegen` assigns each WebIDL interface a tag in the embedder range, 1 to 0x5fff (V8-internal tags start at `kFirstV8InternalTag = 0x6000`; tags are 15-bit), by **Izcal's own scheme**: depth-first pre-order, so each interface's descendants form one contiguous range and unwrapping as `I` checks `[id(I), id(last descendant of I)]`. V8 requires only that "all (transitive) child classes … have type ids in the same range, and that there are no unrelated types in that range"; its own example puts the parent at the end of its range, so pre-order is one valid choice. The generator asserts uniqueness and containment at build time. **The fork needs a new binding:** rusty_v8's `Object::unwrap` and the fork's `v8__Object__Unwrap` (`m0/rusty_v8/src/binding.cc` L4836) build `CppHeapPointerTagRange(tag, tag)`, an exact-tag check, so a range `unwrap(first, last)` is added to the fork (S-4e divergence, offered upstream). After unwrap, generated code compares the object's stored interface ID before producing `&T` | [v8-sandbox.h](https://chromium.googlesource.com/v8/v8.git/+/main/include/v8-sandbox.h) (read in v8 152.2.0); `gin` performs "a second type check to assert that we indeed have the expected type before casting" ([wrappable.h](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/gin/wrappable.h)); type confusion is the top V8 bug class (security scope 3.1) | gen (build-time assertion); fork test of the range binding | The second check is dropped only if the V8 sandbox is mandatory in every shipped build and the perf lab shows a cost (decision-log entry) |
| **CS-V8-3** | **No raw pointers in V8 objects** (FFI-R1). No `v8::External`, internal field or embedder slot holds a raw Rust pointer; links go through `cppgc` wrapping, stable IDs, or the typed `External` wrapper (index into a per-isolate table with a type tag and generation) | Deno CVE-2024-27934 | clippy.toml `v8::External::new` and the internal-field setters (stays on in tier O; checked by the canary, since `v8` paths are silent elsewhere); G6; SEC-11 (a) | The wrapper module, with `#[expect]` |
| **CS-V8-5** | **JS values reach Rust only through generated WebIDL conversions**, returning `Result<T, TypeError or RangeError>`; no unchecked cast on `Local` handles in hand-written code | WebIDL conversion algorithms | gen; hook `cs-unchecked-local-cast` | None |
| **CS-V8-9** | **Finalization discipline.** Rust payloads stored in GC objects implement a sealed `GcPayload` marker, emitted by the generator or a derive that rejects payloads containing `Member`, GC handles, `TokenCell`, `SequenceBound`, thread-affine types, or a non-trivial `Drop` not on an allowlist. A payload's `Drop` never touches another GC object or a node-ID lookup that reaches the heap, never allocates in the `cppgc` heap, never posts script work and never takes a lock above the lowest level. Prefinalizers are generated and only release data-plane and semantic-plane slots by ID (integer operations). Until Q-CS-4 settles which thread runs destructors, payload `Drop` must be safe on any thread | The fork's `RustObj::~RustObj()` calls the Rust drop, so `Drop` runs during sweep; `CppHeap` defaults to `sweeping_support = kIncrementalAndConcurrent` (`v8-cppgc.h`); a prefinalizer "may access the whole object graph, irrespective of whether objects are considered dead or alive" and runs on the creating thread (`cppgc/prefinalizer.h`) | gen; compile-fail tests; CI: debug-V8 S-4c stress suite extended with sweeping over poisoned memory | Allowlisted `Drop` types (plain buffers), with a reason |
| **CS-FFI-1** | **Izcal-authored C++ and FFI declarations.** For Izcal's own code in the `rusty_v8` fork (the predictable platform, the `cppgc` additions, the range unwrap) and any project C or C++: the fork's clang-format style; warnings as errors; no exceptions; no `abort` reachable from embedder input; a SAFETY-style comment on every function that takes a raw pointer from Rust; every `extern "C"` declaration has a layout and ABI test (`static_assert(sizeof/alignof)` on the C++ side matched by Rust `const` asserts, generated where possible); ASan and UBSan builds of the fork's tests in T1; review by the `rusty_v8` owner; each commit stays upstreamable (`CONTRIBUTING.md`). Upstream code keeps upstream's conventions | Hand-declared bindings make a signature mismatch silent undefined behavior; DET-1 depends on the predictable platform | CI (fork jobs, T1 sanitizers); owner; review | Upstream code |

### 6.7 Async and concurrency (CS-AS)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-AS-1** | **No ambient effects outside `hg-substrate-os`** (extends G3, in tests and benches too). Banned elsewhere: `std::thread` spawning, scoping, sleeping and parking; the `std::time` clocks; `std::process::Command` and `exit`; `std::net` sockets and name resolution; every `std::fs` function and type; `std::env` variables, arguments and directories; any `std::os::` path; `rand`'s thread RNG, `getrandom`, `num_cpus`. A dependency API that hands Izcal a banned type (a std `HashMap`, an `Instant`) is used through one adapter module per dependency with one `#[expect]` | Chromium bans with in-house replacements; FDB abstracts "network, disk, time, and pseudo random number generator" ([paper](https://www.foundationdb.org/files/fdb-paper.pdf)) | clippy.toml (A.3); hook `cs-std-os`; graph (`rand`, `getrandom`, `num_cpus`, `tokio`, `smol` reachable from `hg-*` only via `hg-substrate-os` and the canary, dev-dependencies included); CI: clippy `--all-targets`, `hg-lint-canary` | Tiers O and T |
| **CS-AS-2** | **Deterministic hashing.** Hash maps use `hg_types::HashMap`, `HashSet`, `IndexMap` and `IndexSet`, aliases whose hasher reads **one key per OS process**, set once at startup by `hg-substrate-os` (random, which keeps HashDoS resistance) or by the simulator (from the seed), stored in a `OnceLock` in `hg-types`: the one sanctioned piece of global state there. Never a key per map, since map-creation order differs across fork/join workers. Output-affecting code never iterates a hash map; benchmarks use the simulator key | S2's DST leaked through `HashMap` ([S2](https://s2.dev/blog/dst)) | clippy.toml std, `hashbrown`, `indexmap` and `ahash` types (verified); lint `iter_over_hash_type` (fires through the alias) | The alias definitions, with `#[expect]` |
| **CS-AS-3** | **The substrate owns every thread.** Work is posted only through `Executor::post` on a named sequence or `fork_join()`; no detached spawn. Rayon pools are built by the substrate; Stylo receives that pool through `traverse_dom`; nothing touches Stylo's `STYLE_THREAD_POOL` (which spawns threads when its computed count exceeds 1, capped at 6) or Rayon's global pool | Chromium's mock time does not cover threads it does not own ([task_environment.h](https://chromium.googlesource.com/chromium/src/+/main/base/test/task_environment.h)); Stylo 0.22 source | clippy.toml Rayon entries; hook `cs-style-thread-pool`; CI: thread count after startup equals the substrate's | `hg-substrate-os` |
| **CS-AS-4** | **The async model (CS-D8).** A `Task` is a `FnOnce` closure that runs to completion on its sequence; completions arrive as posted tasks or `SequenceBound` sinks. `async fn`, `async` blocks and `.await` appear only in crates on the allowlist in `tools/tiers.toml` (initially `hg-substrate-os` and `hg-substrate-sim`; a protocol driver in `hg-net` if Q-CS-2 needs one), where futures are `!Send`, sequence-local and polled only by a substrate poller. Public cross-crate traits are synchronous: no `async fn`, `-> impl Future` or `Pin<Box<dyn Future>>` in a `pub` trait, and no `async-trait`. No third-party runtime or executor (`tokio`, `smol`, `futures::executor`) outside `hg-substrate-os`. `select!`-style macros only in `hg-substrate-*`, with `biased;`, never over a `&mut` future with an `await` in a branch; every async API there documents `# Cancel safety`. Cancellation is structural: every task has an owner (sequence, document, navigation, fetch), and dropping the owner cancels its children as a task, never mid-poll. **JS Promises and microtasks belong to V8 and are never bridged to Rust futures** | Stage 1 3.1 (sync traits); `async fn` in traits is not `dyn`-compatible (E0038 reproduced); `dyn` async traits are nightly-only work in 2026 with no stable date ([goals](https://goals.rust-lang.org/2026/afidt-box.html)); RTN "is blocked on the next-gen trait solver" ([goals](https://goals.rust-lang.org/2026/rtn.html)); futurelock (RFD 609); async-std discontinued ([README](https://github.com/async-rs/async-std)) | hook `cs-async` and `cs-pub-trait-future` (verified); lint `async_fn_in_trait` (deny), `unused_async`, `large_futures`, `let_underscore_future`; clippy.toml `select!`, `block_on`, `LocalPool`; deny `async-trait`, `tokio`, `smol`, `async-std` | Allowlisted crates; re-evaluated when `dyn` async traits and RTN are stable, or when a dependency the network service needs cannot run sans-IO |
| **CS-AS-6** | **Queues by kind.** (a) **Web-visible queues** (HTML task sources, timers, message ports, `postMessage`) are bounded by **memory accounting** against the renderer's budget; exceeding it is handled as HTML permits (throttle, or end the script without an exception), and at the hard limit is a renderer OOM: a tier R crash of the page's own process. (b) **Cross-process endpoints:** a sender in a tier P process **never blocks** on a peer; on overflow it drops with a counter or declares the peer unresponsive and asks the broker to terminate it. The `.hgidl` declares which, per endpoint direction, with a capacity, and `hg-ipc-codegen` rejects an endpoint without both. (c) **In-process queues** are substrate primitives (`post`, `SequenceBound`, the phase pool): std, crossbeam, flume, futures and tokio channels are banned outside `hg-substrate-*`, and unbounded ones everywhere. A sequence's default capacity is 1024 tasks†, for (c) only | Unbounded channels give no backpressure; Servo exhausted file handles through IPC overuse ([Servo book](https://book.servo.org/design-documentation/architecture.html)); `sync_channel`'s `send` blocks, which would break CS-AS-7; HTML resource limits ([HTML](https://html.spec.whatwg.org/multipage/webappapis.html#killing-scripts)) | clippy.toml channel constructors (verified; unbounded ones stay on in tier O); gen (capacity and overflow policy); CI: flood tests in simulation | None |
| **CS-AS-7** | **Tasks run to completion quickly.** No task blocks (no sync I/O, sleep or wait on another sequence). Budget ≤ 100 µs† per task on script and I/O sequences; longer work goes to `fork_join` or a `MayBlock` sequence. The one sanctioned wait is the sync-IPC allowlist (`document.cookie`, `localStorage` reads, sync XHR; Stage 1 4.1, IPC-R13, with nested sync forbidden), whose time the long-task detector attributes separately | Ryhl's 10–100 µs between awaits ([blog](https://ryhl.io/blog/async-what-is-blocking/)) | CS-AS-1 bans; long-task detector in debug builds and DST (reports only, until calibrated); review | `MayBlock` sequences; the sync-IPC allowlist |
| **CS-AS-8** | **No guard across a task boundary.** No lock guard, `RefCell` borrow, GC-scope guard or `SequenceBound` borrow is moved into a posted task or held across `.await` in an allowlisted crate. Async mutexes are banned | Tokio: the std mutex is "often preferred" in async code ([docs](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)); RFD 400 avoids `tokio::sync::Mutex` | lint `await_holding_lock`, `await_holding_refcell_ref`, `await_holding_invalid_type` with Izcal's guard types; clippy.toml async mutexes (stays on in tier O) | None |
| **CS-AS-9** | **Messages between sequences; short locks and argued atomics inside one.** Shared state crosses as messages or immutable `Arc` snapshots. An unavoidable lock is a std or `parking_lot` mutex with a bounded critical section and a declared level; taking a lower level while holding a higher one does not compile (typed lock levels, Izcal's own). Each atomic operation has an `// ordering:` comment naming the happens-before edge; `Relaxed` only for counters; lock-free structures only in inventoried modules, with loom tests | Chromium discourages locks; WebKit: "rarely an excuse for not having one, or even multiple, fine-grained locks"; netstack3 types its lock order ([docs](https://docs.fuchsia.dev/rust/src/netstack3_core/lock_ordering.rs.html)); loom treats `SeqCst` as `AcqRel` and can miss executions ([README](https://github.com/tokio-rs/loom)) | types; lint `significant_drop_tightening`; hook `cs-atomic-ordering-comment`; G5; review | SHOULD for lock levels until the type exists (Phase 2) |
| **CS-AS-12** | **Deterministic phases.** Fork/join phases read immutable inputs and write disjoint, index-addressed outputs; results gather in index order; reductions use a fixed block decomposition independent of thread count; no floating-point reduction whose grouping depends on scheduling | Rayon: `reduce` order "is not fully specified"; float `sum` "not fully deterministic" ([docs](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html)); DET-1 | CI: DET-1 parallel-vs-sequential hash; hook `cs-float-par-reduce` (heuristic); review | None |
| **CS-AS-13** | **Nondeterminism inventory for simulated dependencies.** Every third-party package linked into code that runs under `hg-substrate-sim` (a subgraph `hg-graph-lint` computes) has an entry in `nondeterminism-inventory.toml`: threads, clocks, OS randomness or `RandomState`, I/O, and how Izcal neutralizes each. Entries are **pre-filled** by an `ast-grep` scan of the package source for those APIs, so a human confirms rather than invents, and re-scanned on every version bump. Other packages get the generated default "outside simulation" | FDB avoided dependencies it could not simulate; S2's leaks were third-party | graph (entries present and scan-fresh); CI: same-seed rerun meta-test (CS-T5) | None |

### 6.8 Structure, build and generated code (CS-ST)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-ST-1** | **Flat layout.** Every crate at `crates/<name>/`, folder name = package name; a virtual root manifest; internal crates `publish = false`, `version = "0.0.0"`; automation in `xtask`; `foo.rs` plus `foo/`, no `mod.rs` | "Tree structure tends to deteriorate" ([matklad](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)) | graph; lint `mod_module_files` | Published crates |
| **CS-ST-2** | **Layers, and crates split by ownership, not size.** G1–G9 stand. A crate depends only on its own or lower layers; proc-macro crates only from an allowlist; no L1–L3 crate depends on `syn` or a derive macro. A new crate needs a reason: a layer boundary, a G-rule, an oracle, a separate owner, or a build-time win shown with `cargo build --timings`; no chains of thin re-export crates | Proc macros block pipelining; "do not put generic code at the boundaries between the crates" ([matklad](https://matklad.github.io/2021/09/04/fast-rust-builds.html)) | graph; review; T1 timings report | Allowlist entries are reviewed decisions |
| **CS-ST-6** | **Features are additive and few.** No feature removes behavior or changes web-observable behavior except those the specs define (`ai`, G4); no mutually exclusive or `no-*` features | Cargo unifies features by union ([Cargo book](https://doc.rust-lang.org/cargo/reference/features.html)); per-package unification is unstable | lint `negative_feature_names`, `redundant_feature_names`; graph (G4); CI (T1): one build per `hg-*-main`, `cargo hack --each-feature`, clippy per binary with default features | None |
| **CS-ST-7** | **Build scripts only for allowlisted crates.** `build.rs`, in the workspace and in dependencies, only for crates on `[bans.build] allow-build-scripts`; each addition is a SEC-14 (d) recorded human decision with a reason and a reviewer, under `CODEOWNERS` (CS-GA-1). Crates whose build runs scripts or ships executables (the `v8` crate, Stylo, which runs Python and mako at build time, `aws-lc-sys`, `freetype-sys`) get a `bypass` pinned to their build script's SHA-256, so a changed `build.rs` is rescanned. The list starts from the real graph: **65 crates** on a probe of 12 L0 dependencies plus the test tools (A.4, measured), not "the `v8` fork and a few FFI crates" | The arrayref attack ran through a build script; cargo-deny scans compile-time crates ([docs](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html)) | deny `[bans.build]` (A.4, passing on the probe graph); owner | Reviewed entries |
| **CS-ST-9** | **Toolchains are pinned and bumped on a cadence.** `rust-toolchain.toml` pins stable, which is the MSRV (`rust-version` in `[workspace.package]`); edition 2024, resolver 3. `rust-toolchain-nightly.toml` pins one dated nightly, used only by Miri, cargo-fuzz and loom jobs, bumped together with stable. A bot opens a stable bump every second release† (about 12 weeks), including the lint diff, and drops misfiring nursery lints with reasons. Exact-pinned L0 dependencies (Stage 1 7.7) get a monthly† bump PR per owner; security bumps are immediate | MSRV-aware resolver default in edition 2024 ([Rust 1.84](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/)); unpinned nightlies make jobs drift | hook `toolchain` (the versions agree); CI bot | The fork keeps its own pin |
| **CS-ST-10** | **Tiers are declared; lint tables are generated.** `tools/tiers.toml` declares each crate's tier (6.2). Tier R crates use `[lints] workspace = true`; tier P, O and T crates carry a full `[lints]` table, and tier O its own `clippy.toml`, written by `cargo xtask lints` from `tools/lints.toml`. No crate sets lint levels by hand | `[workspace.lints]` since 1.74; Cargo rejects mixing (measured); clippy reads a crate's own `clippy.toml` first (verified) | CI `lints-fresh`; graph (declared ≥ computed tier; every manifest inherits or carries the generated table) | None |
| **CS-ST-13** | **Documentation where it pays.** Every crate root documents its layer, tier, G-rules and threading model (which sequence or phase it runs on). Item docs, `# Errors` and `# Panics` are required in L1–L2 contract crates, tier P and published crates; `# Safety` and `# Cancel safety` everywhere they apply; intra-doc links resolve. Widened only if the M2 review shows a need | C-FAILURE; boilerplate risk in fast-moving internal crates | lint `missing_crate_level_docs`; tier P table `missing_docs`, `missing_errors_doc`, `missing_panics_doc`; rustdoc `broken_intra_doc_links` (deny) in the `doc` job | Generated code gets docs from its generator |
| **CS-ST-14** | **Naming and API conventions.** RFC 430 casing; crates `hg-<area>`; `as_`/`to_`/`into_` conversions; `try_` for fallible twins; `Id` suffix for ID newtypes; per-operation errors named `<Op>Error`; unit suffixes (`_px`, `_bytes`, `_ms`) where a newtype is not used; no `bool` parameters in public signatures (CS-TY-4); sequences named `"<process>.<component>"`. Internal crates make no semver promise; crates on `published-crates.toml` (embedding API, Stage 2) run `cargo-semver-checks` and use sealed traits, private fields and `#[non_exhaustive]` | C-CASE, C-CONV, C-SEALED, C-STRUCT-PRIVATE | rustc style lints; graph (prefix); CI (published list, release); review | None |
| **CS-ST-17** | **Generated code.** (a) Output is checked in under `crates/<crate>/src/generated/`, one file per interface or protocol, regenerated by `cargo xtask codegen`; T0 fails if regeneration changes anything; files start with a header naming the generator and input, and carry G9 anchors (CS-D4). (b) `**/src/generated/**` is `linguist-generated` with `merge=izcal-regen` in `.gitattributes`; a conflict is resolved by taking either side and regenerating. (c) Generated tier P code passes the tier P table with only the expectations listed in `tools/lints.toml` (for example `too_many_lines`, `missing_docs`); a generator may not emit any other. (d) A generator change carries a golden-output diff on a fixed IDL sample in the PR, and needs the security owner when it touches IPC or V8 seams. (e) Output is byte-identical across OSes (`codegen-fresh` on all three). (f) The AI review and `cs-patterns` skip generated files; they review the generator | Checked-in generated code with a freshness test ([matklad](https://matklad.github.io/2022/03/26/self-modifying-code.html)); `hg-bindings` is "the largest single body of Stage 1 code" (Stage 1 5.3) | CI `codegen-fresh` (three OSes); graph (generated expectations from the list); owner | None |
| **CS-ST-18** | **Platform code is contained and linted on every platform.** `cfg(target_os)` appears only in `hg-substrate-os`, `hg-ipc` transport modules, the sandbox and `hg-gpu` backends, one module per OS (`os/linux.rs`, `os/macos.rs`, `os/windows.rs`). Clippy and tests run on Linux, macOS and Windows | Clippy on Linux does not lint macOS or Windows code paths | hook `cs-cfg-os`; CI: clippy on three GitHub-hosted OSes (Linux required in T0; macOS and Windows in T0 in parallel, falling back to T1 if runner cost requires) | None |

### 6.9 Testing (CS-TE)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-TE-2** | **Three sizes, one layout.** *Small:* in-crate `src/**/tests.rs` via `#[cfg(test)] mod tests;`, pure, no substrate. *Medium:* under `hg-substrate-sim` with an explicit seed, in `tests/it/sim/`, bounded by simulated time or scheduler steps, never by wall-clock timeouts. *Large:* real OS, network or GPU, only in `hg-substrate-os`'s tests (`tests/it/os/`), the perf lab and T2/T3. At most one integration-test binary per crate, `tests/it/main.rs`, with modules declared `#[cfg(test)] mod …;` so the test exemptions apply (measured). `cargo nextest` is the runner | Google's sizes; merging integration tests cut Cargo's own suite's compile time about 3× ([matklad](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html)); process-per-test ([nextest](https://nexte.st/docs/design/why-process-per-test/)) | graph (paths, one `tests/` root); review | None |
| **CS-TE-3** | **Every bug fix carries a regression test that fails before the fix.** The test's header links the issue and, for web-visible behavior, the spec step or WPT test. The PR body carries `Test: <path>` or `Test: none — <reason>` (CS-GA-3). The trigger is a PR that closes an issue labeled `bug` or carries `Fixes:`, not a branch prefix | LLVM requires tests for fixes ([policy](https://llvm.org/docs/DeveloperPolicy.html)); squash merges drop per-commit trailers | CI `pr-metadata`; CI `test-fails-on-base` (affected crates' test binaries only) | `Test: none — <reason>` (counted by CS-T8) |
| **CS-TE-4** | **Web-visible behavior is tested by WPT first.** A behavior change flips a pinned WPT expectation or adds a WPT test (upstreamed when it does not depend on Izcal internals); expectation changes are reviewed, never auto-blessed | Servo: "Your PR won't get accepted without a test" ([Servo book](https://book.servo.org/contributing/testing.html)); M-1 part 4 | CI: `hg-wpt` fails on unexpected PASS and FAIL; review | Internals with no observable behavior |
| **CS-TE-5** | **No retries; quarantine with an expiry.** The nextest CI profile sets `retries = 0` in T0 and T1. A test that fails then passes is a determinism or test bug, filed automatically; a wall-clock termination is counted as an infrastructure failure, not a flake. A flaky test goes into `tests/quarantine.toml` with an issue, owner and expiry (14 days†); quarantined tests still run in T1/T2; an expired entry fails T0; forced quarantine at ≥ 2%† failures over 7 days† | Flaky tests revealed more than a third of regression faults ([Haben et al.](https://arxiv.org/pdf/2302.10594)), so a retry discards evidence; tests lose value near 1% flakiness ([SWE at Google](https://abseil.io/resources/swe-book/html/ch11.html)); Firefox's thresholds and 2-business-day clock ([policy](https://wiki.mozilla.org/Sheriffing/Test_Disabling_Policy)) | CI (`.config/nextest.toml`; `quarantine-expiry`); owner (quarantine is a gate file) | None |
| **CS-TE-6** | **New and changed medium tests run under extra seeds before they count.** T0 runs each added or modified medium test over 20 extra seeds†, in shuffled order. Small tests are rerun only where they share statics | Firefox test-verify; process-per-test makes reruns of pure small tests nearly worthless | CI `test-verify` | None |
| **CS-TE-8** | **Failures become permanent inputs.** Proptest regression files, minimized fuzz crashes and failing DST seeds are committed (`proptest-regressions/`, `fuzz/corpus-regressions/`, `dst/seeds/`) and replayed in T0 | SQLite replays historical fuzz cases on every `make test` ([SQLite](https://www.sqlite.org/testing.html)); proptest persistence ([book](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html)) | CI; review | None |
| **CS-TE-10** | **Properties come from spec invariants.** Prefer proptest strategies (which are "aware of simple constraints") to type-based generation; T0 256† cases, T2 ≥ 10k†; a property test cites the spec statement it encodes; its seed comes from the run's test seed; `bolero` where the same harness should also run under Kani. Cross-component invariants are checked by producer and consumer | [proptest](https://proptest-rs.github.io/proptest/proptest/vs-quickcheck.html); [bolero](https://docs.rs/crate/bolero/latest) | review; `hg-anchor` | None |
| **CS-TE-11** | **Every parser of untrusted bytes has fuzz targets**, raw-bytes and structured, registered with OSS-Fuzz (Stage 1 7.5), including every IPC-R16 payload in its receiver and a sequence fuzzer per endpoint (IPC-R14) | Structure-aware fuzzing ([Rust Fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html)) | graph (crates tagged `parser` have a `fuzz/` target); CI: T0 `cargo check` of fuzz targets and corpus replay, T1 builds | None |
| **CS-TE-12** | **Snapshots never stand alone.** `insta` for diagnostics, serialized formats and protocol output; each snapshot test also asserts the key property; nondeterministic fields are redacted; CI fails on `.snap.new` or unreferenced snapshots; never the oracle for web-visible behavior | rustc's redundancy "helps avoid mistakes since the .stderr files are usually auto-generated" | CI (`INSTA_UPDATE=no`); review | None |
| **CS-TE-13** | **Unsafe and concurrency primitives are model-checked.** Inventoried modules run their pure-Rust unit tests under Miri in T1 (FFI behind fakes; Miri's experimental `-Zmiri-native-lib` is not relied on), with more seeds nightly. Hand-written concurrency primitives get loom tests with preemption bound 3. Pure-Rust unsafe blocks SHOULD have a Kani harness | Miri cannot run FFI by default ([Miri](https://github.com/rust-lang/miri)); a bound of 2–3 "is enough to catch most bugs" ([loom](https://docs.rs/loom/latest/loom/)) | CI `miri`, loom (pinned nightly, CS-ST-9) | V8-calling tests (debug V8 and ASan) |
| **CS-TE-15** | **Adequacy reports, never targets.** CI shows diff coverage from small tests and, on PRs touching G9 behavior crates, `cargo mutants --in-diff` survivors. Uncovered changed lines in behavior crates need a sentence in the PR. No repository-wide gate; mutation reports become blocking for AI-generated code if EQ-1's rates diverge | Coverage targets become ceilings ([SWE at Google](https://abseil.io/resources/swe-book/html/ch11.html)); Google surfaces mutants in review ([paper](https://research.google/pubs/state-of-mutation-testing-at-google/)) | CI (T1, report only) | None |
| **CS-TE-17** | **Tests for AI-generated algorithms come from the spec, not the code.** For generated code implementing web-visible behavior, the spec-derived tests land first, in an earlier commit or PR marked expected-fail, written in a session that has not seen the implementation; CI shows they fail there and pass after the implementation; the test commit's `Claude-Session` differs from the implementation's (CS-D6). This is Izcal's addition to ORC-1, which requires oracles before code | LLM-generated oracles encode actual behavior ([arXiv 2410.21136](https://arxiv.org/abs/2410.21136)) | CI `pr-metadata` (session trailers, expected-fail history), `test-fails-on-base`, CS-TE-15; review | None |

### 6.10 Performance (CS-PF)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-PF-1** | **Size budgets for hot types.** Types instantiated per node, box, fragment, style, task or IPC message carry `#[cfg(target_pointer_width = "64")] const _: () = assert!(size_of::<T>() == N);`; growth edits the budget in the same PR with a reason | [Rust Performance Book](https://nnethercote.github.io/perf-book/type-sizes.html); INC-3 | rustc (const assert); review of the hot-type list | None |
| **CS-PF-3** | **Allocation budgets as tests.** Key operations (restyle of N nodes, layout of a cached subtree, IPC encode/decode, a frame with no DOM change) have deterministic tests counting allocations through the Rust global allocator, with an upper bound | [heap allocations](https://nnethercote.github.io/perf-book/heap-allocations.html); INC-3's counting allocator | CI (T0, path-filtered) | None |
| **CS-PF-4** | **Deterministic per-PR perf gate.** Hot CPU-bound kernels (selector matching, cascade, layout driver, display-list build, IPC codecs, URL and other parsers) have `gungraun` instruction-count benchmarks on GitHub-hosted Linux. An increase above 1%† fails unless the PR body carries `Perf: <reason>`, which is listed for review (CS-GA-3). Benchmarks are deterministic: hashing uses the simulator key (CS-AS-2); kernels run single-threaded; base and head are measured in the same job on the same runner image; benchmark crates do not link `v8`, which G1 and G6 make possible (style and layout depend only on `hg-tree-traits`) | `instructions:u` detects ~0.3% changes; the `Binary-Size:` footer model | CI `perf-gate` (path-filtered to benchmarked crates); CI `pr-metadata` | `Perf:` in the PR body, reviewed |
| **CS-PF-5** | **Backout first for lab regressions.** A sustained regression in a lab metric (P-1, RES-*) is bisected automatically and reverted by default; the author may land a fix within 2 business days† instead. Alerts are triaged within 1 business day† | WebKit: back out or fix at once; Firefox: daily review and a 1-business-day developer response; 3–7% of alerts invalid ([Mozilla](https://blog.mozilla.org/performance/2021/05/26/performance-sheriff-newsletter-april-2021/)) | CI (bisection bot); process | None |
| **CS-PF-6** | **Evidence for speed.** A design or PR that changes a per-node, per-frame or per-message path states its expected CPU, memory and IPC-byte cost first. Added complexity, `unsafe` (including `get_unchecked` and SIMD, which fall under G5) or `#[inline(always)]` for speed must show the gain on a corpus workload or an instruction-count benchmark of a real call path; bounds checks are removed by structure (iterators, slice-then-index). Platform choices such as the global allocator are made in the perf lab on RES-3 and P-1 and recorded as decisions | Octane's overfitting; "workloads using real-world inputs are best" ([perf book](https://nnethercote.github.io/perf-book/benchmarking.html)); TigerBeetle's back-of-the-envelope sketches | review; lint `inline_always`; G5 | None |
| **CS-PF-8** | **Generics in hot loops, `dyn` at boundaries; size is reported.** Generic functions keep their non-generic body in an inner function. CI reports `cargo llvm-lines` and binary size per PR; installer growth above 256 KB† needs `Binary-Size: <reason>` in the PR body (RES-4) | Monomorphization cost ([tonic PR 1032](https://github.com/hyperium/tonic/pull/1032): about 23% less LLVM IR, a small build-time gain) | CI (T1 report and threshold); CI `pr-metadata` | `Binary-Size:` in the PR body |

### 6.11 Agents and the gate (CS-GA)

| ID | Rule | Why (source) | Enforced by | Exceptions |
|---|---|---|---|---|
| **CS-GA-1** | **Gate files have owners.** `clippy.toml`, the generated tier copies, `tools/lints.toml`, `tools/tiers.toml`, `deny.toml`, `unsafe-inventory.toml`, `nondeterminism-inventory.toml`, `tests/quarantine.toml`, `.config/nextest.toml`, `tools/ast-grep/`, `tools/coding-standard/` (including `tier-p-exceptions.toml`), the generators' expectation list, `.gitattributes`, `.github/workflows/` and `CODEOWNERS` itself are listed in `CODEOWNERS` under the security owner (unsafe-inventory entries also under their inventory owner), with "Require review from Code Owners" on `main`. A CI job `gate-config-changed` labels any PR that touches them and fails without that approval. Until a second maintainer exists, the approval is a recorded self-approval line in the PR, as the security scope's Section 9.2 allows for SEC-6 | Each lever is a one-line diff an agent told "make CI green" can write; SEC-27 runs checks from the base branch, but their configuration comes from the PR; GitHub: "an approval from any of the owners is sufficient" ([docs](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners)) | owner; CI `gate-config-changed` | None |
| **CS-GA-2** | **How agents apply this standard.** Before writing code an agent reads the one-page summary and the rules `tools/coding-standard/rules.toml` lists for the touched crates' tiers (`cargo xtask tier <crate>`), and runs `tools/checks.sh --range` before pushing. An agent **never** edits a gate file (CS-GA-1), adds a quarantine, inventory, allowlist or bypass entry, weakens or deletes a test, or relaxes a tier, unless the task explicitly says so and the PR says which file and why. A suppression or `invariant!` reason states a checkable fact (6.1); an agent that cannot state one stops and reports instead of suppressing. When a rule blocks the task, the agent reports the conflict rather than working around it. The agent instructions are checked in at `tools/coding-standard/agent-rules.md`, generated from the rule index | Agents optimize for the check they are given; a false reason is worse than a missing one because it is trusted | CI `gate-config-changed`; CS-GA-3 listing; CS-T13 sampling; review | The task explicitly assigns the gate change |
| **CS-GA-3** | **The PR is the record.** Trailers live in the PR body (`Spec:`, `Test:`, `Perf:`, `Binary-Size:`, `Fixes:`), which the repository's squash setting "pull request title and description" keeps in history ([GitHub](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/configuring-commit-squashing-for-pull-requests)). A CI step lists in the PR summary every **new** `#[expect]`, `invariant!`, `cs-allow`, quarantine entry, bypass, `Perf:` and `Binary-Size:` in the diff; the AI review verifies each reason against the code and treats a false reason as high severity. `.github/pull_request_template.md` is generated from the rule index: tiers of touched crates, new exceptions (auto-listed), unsafe changed, trailers, generated code regenerated, gate files touched, and the review-only rules for the reviewer to tick (CS-TY-5, CS-PF-6) | The project squash-merges and the PR title becomes the subject (`CONTRIBUTING.md`), so per-commit trailers are lost | CI `pr-metadata` (parses the PR body; fails on missing or malformed trailers) and `exceptions-listed` | None |
| **CS-GA-4** | **A machine-readable rule index feeds every agent and the AI review.** `tools/coding-standard/rules.toml` (proposed; not created by this document) holds, per rule: ID, one-line rule, tiers, path globs, mechanism, and whether a tool checks it. `xtask` generates from it the per-crate header ("this crate is tier P; rules …"), `agent-rules.md`, the PR template and the AI review's context: the one-page summary, the rules applying to the touched crates (review-only rules emphasized) and the new-exceptions list. The review sends the hand-written diff in chunks of up to 4000 lines each, every chunk with that context, instead of cutting at 4000 lines, and excludes `src/generated/` (CS-ST-17 (f)) | The review prompt today knows nothing of the standard, and a generator change would fill its 4000-line budget | CI `rules-index-fresh` (every CS ID in this document appears in the index and vice versa); `tools/checks.sh` review | None |

### 6.12 Rules owned by other documents

This standard supplies mechanisms for these and does not restate them. 0.1 IDs that only restated them are retired (Appendix C).

| Rule | Owner | Mechanism here |
|---|---|---|
| No raw pointers through fast API calls until the sandbox crash is understood (was CS-V8-7) | Stage 1 5.3 | gen: golden-file test plus a check over the full pinned IDL set |
| `cppgc` discipline: generated `Member` setters, no allocation or traced writes from style and layout workers (was CS-V8-8) | S-4c, FFI-R3 | gen; debug-V8 CI; CS-V8-9 adds finalization |
| Fuzz the Rust↔V8 boundary (Fuzzilli via REPRL) | FFI-R4, SEC-11 (c) | CS-TE-11 |
| Dependencies: cargo-deny's checks, cargo-vet, 7-day cooldown, recorded decision per new crate, vendored release builds (was CS-ST-8) | SEC-14, Stage 1 7.1 | A.4; CS-AS-13 inventory; CS-GA-1 |
| No code from another browser; spec anchors; `Spec:` trailer (was CS-ST-16) | M-1, D-S10, G9, ANC-1, Stage 1 7.2 | `provenance` check; `hg-anchor`; CI `pr-metadata` (the `Spec:` check is new: `commit-msg` does not check it today) |
| Reproducible PGO from the simulated corpus (was CS-PF-10) | Stage 1 7.1 | Release pipeline. 0.1's "PGO and LTO are worth 5–20%" is withdrawn: Android's figures measure a PGO-built rustc compiling faster, not faster programs |
| Attack-surface inventory and its approval | IPC-R11, SEC-6 | gen; CS-GA-1 |

### 6.13 Integrations: where this standard is the mechanism for other targets

| Target (security scope draft 0.2 unless noted) | What it requires | Mechanism here |
|---|---|---|
| IPC-R2, SEC-3 (a) | Identity from bound endpoints; `CallerIdentity` constructible only in `hg-ipc` | CS-TY-5, gen |
| IPC-R3, IPC-R4 | Closed type universe; valid by construction | CS-TY-3, CS-TY-4, CS-ER-6 |
| IPC-R5, SEC-7 (a), (b) | No panic on any byte string; handler lints denied with a reviewed allowlist | CS-ER-3 (tier P table, `invariant!`, generated `tier-p-exceptions.toml`); Kani in `hg-ipc-codegen` |
| IPC-R5 panic strategy, NET-R3, SEC-28 | Per-process strategy; network service unwinds per connection | **Conflict**: CS-D1 (Section 6.3) |
| IPC-R7 | Copy then validate what the receiver parses | gen |
| IPC-R8, SEC-3 (d) | Generation-tagged, never-reused IDs | CS-TY-1 |
| IPC-R13 | Sync allowlist; nested sync forbidden | CS-AS-7 |
| IPC-R14, IPC-R16, SEC-7 (c) | Sequence fuzzers; named non-`.hgidl` payloads with fuzz targets | CS-TE-11, CS-ER-3 |
| IPC-R17 | No process trusts another | CS-ER-3, CS-ER-6 (no "less privileged" test) |
| FFI-R1, SEC-11 (a) | `External` only through the typed wrapper | CS-V8-3 (clippy.toml, canary) |
| FFI-R2, SEC-11 (b) | No buffer borrow across script or await | CS-TY-10 |
| FFI-R3, FFI-R4, SEC-11 (c) | S-4c; boundary fuzzing | Section 6.12; CS-V8-9 |
| SEC-6, SEC-27 | Attack-surface diff approval; checks from the base branch | CS-GA-1 |
| SEC-14 | Dependency review | CS-ST-7, A.4 |
| SEC-28 | Network parsers degrade; capped pre-auth allocation | CS-ER-3, CS-ER-9, CS-TE-11 |
| SEC-34, SH-R5, SEC-35 | Secrets at rest; filtered crash dumps | CS-ER-14 (`Secret<T>`, literal panic messages) |
| DET-1, DST-1 (HGBA-3) | Bit-identical replay ("ship blocker for simulation mode"); DST yield | CS-AS-1 to CS-AS-4, CS-AS-12, CS-AS-13, CS-ER-2, CS-ER-10, CS-TE-5 |
| VER-1 (HGBA-3) | Kani-checked renderer-to-broker message types | CS-ER-3, CS-ER-9 |
| EQ-1, ORC-1 (HGBA-3) | Gated generation; oracles before code | CS-TE-3, CS-TE-15, CS-TE-17 |
| INC-3, RES-4 | Memory and installer size | CS-PF-1, CS-PF-3, CS-PF-8 |
| DET-2 | Substrate cost ≤ 1% | CS-AS-4 keeps contracts synchronous |

### 6.14 User experience, privacy and consent

The standard's users are contributors and agents (Sections 6.1, 6.11). Privacy is CS-ER-13 (no internal detail reaches script) and CS-ER-14 (no page data in diagnostics, `Sensitive<T>`, `Secret<T>`). Consent does not apply.

### 6.15 Alternatives considered

| Alternative | Recommendation | Reversal trigger |
|---|---|---|
| Enable clippy `pedantic` and `nursery` as groups (neqo) | **No**: enumerated lints, each with a rule ID, so an agent can tell why a lint fires | The table misses a bug class found twice in review |
| A custom rustc lint (dylint-style) for `RefCell` across re-entrant calls | **No** for Stage 1: token cells make it a borrow error (CS-D3) | CS-D3's fallback |
| Exempt tier O from all `disallowed_*` with `allow`, as tier T is | **No**: a per-crate `clippy.toml` keeps `catch_unwind`, unbounded channels, async mutexes and `v8::External::new` banned in the substrate (verified) | — |
| Move non-G3 bans to `ast-grep` so tier O can share tier T's exemption | **No**: clippy resolves paths through re-exports; a text rule does not | Per-crate `clippy.toml` lookup changes in a toolchain bump |
| `-D warnings` in `RUSTFLAGS` | **No**: pass it to clippy only, so `cargo build` caches stay valid | — |
| Percent-coverage gate; wall-clock perf gate per PR | **No** (CS-TE-15, CS-PF-4) | — |
| An adopted tracing crate for diagnostics | **No**: Izcal builds its own facade over the substrate, following CS-ER-14 (the standing build-our-own answer; not an owner decision) | — |

## 7. M-1 compliance

- **Dependencies used unmodified, as tools or libraries:** rustc, clippy, rustdoc, rustfmt, Cargo, cargo-deny, cargo-vet, cargo-nextest, cargo-fuzz and libFuzzer, OSS-Fuzz, proptest, bolero, Kani, Miri, loom, `gungraun` with Valgrind, `divan` (local only), insta, cargo-mutants, cargo-llvm-cov, cargo-hack, cargo-semver-checks, cargo-machete, ast-grep, GitHub `CODEOWNERS`, and `thiserror` (L4 and up). None is part of another browser.
- **Built new by Izcal:** the tier model, `tools/tiers.toml` and the declared-tier check; the lint-table and tier-`clippy.toml` generator; `invariant!`; `hg-graph-lint` extensions (A.6); the `ast-grep` rule set; `hg-lint-canary`; `TokenCell`/`ScriptToken`; `GcPayload`; typed lock levels; `hg_types::HashMap` and the per-process key; `Sensitive<T>`, `Secret<T>` and the diagnostics facade with its `log` adapter; the range-unwrap binding in the fork; the rule index and its generated agent rules, PR template and review context; the quarantine, test-verify, test-fails-on-base, `pr-metadata`, `exceptions-listed` and `gate-config-changed` jobs; the generator checks.
- **What each part is anchored to:** lint and profile rules to the Rust and Cargo references and clippy's documentation; CS-ER-13 to WebIDL, Fetch and HTML; CS-AS-6 (a) to HTML's resource-limit clause; CS-V8-1 and CS-V8-9 to V8's public `v8-sandbox.h`, `v8-cppgc.h` and `cppgc/prefinalizer.h`; CS-ER-6 to IPC-R4, IPC-R5 and IPC-R17.
- **Other projects as evidence only.** Chromium's, Firefox's, Servo's and WebKit's documents, postmortems and build files establish facts about their practice. No code, lint implementation or style-guide text is copied.

## 8. Targets

† marks thresholds this document sets from first principles; none is measured on Izcal yet. Numbers set by other documents carry no †. Prefix `CS-T` does not collide with any prefix in `reports/` (checked 18 October 2026).

| ID | Target | Metric and threshold | How measured | If it fails |
|---|---|---|---|---|
| **CS-T1** | **Main is lint-clean** | **0** rustc and clippy warnings with `--all-targets --all-features` on Linux, macOS and Windows; **0** rustdoc warnings in the `doc` job; **0** `clippy.toml` configuration warnings; on every merge | Required T0 `clippy`, `clippy-config` and `doc` jobs | The merge is blocked; a red `main` is reverted |
| **CS-T2** | **Tier P exceptions are rare** | **100%** of `#[expect]` carry a reason; in tier P, `invariant!` uses plus panic-lint expectations ≤ **1 per 1,000 lines†** per crate, published per milestone | Lint; `tier-p-exceptions.toml` | Above the bound, new exceptions in that crate need the tier P owner's approval until it is back under |
| **CS-T3** | **Rules are mechanical, in CI** | By the end of Phase 2, ≥ **70%†** of the 65 rules have a mechanism running in CI. This draft names a tool mechanism, at least in part, for all 65; mainly review: CS-TY-5, CS-PF-6; report-only until calibrated: CS-AS-7; partly mechanical or heuristic: CS-ER-7, CS-TY-1, CS-AS-12, CS-TE-17, CS-GA-2. Today's running count is 0 | Rule index (`rules.toml`, "checked by") joined with the CI job list and the canary, at each milestone | Convert review-only rules or record why they cannot be converted |
| **CS-T4** | **The gate is not flaky** | Flaky results ≤ **0.1%†** of T0/T1 test runs over a rolling 30 days†, wall-clock terminations counted separately as infrastructure; **0** expired quarantine entries | nextest results; `quarantine-expiry` | New-test merges in the affected crate pause until back under |
| **CS-T5** | **Same seed, same trace** | **100%** of nightly rerun pairs (≥ 100† seeds a night) produce byte-identical DST traces | `hg-dst` meta-test (S2's method) | A determinism bug, filed as a DET-1 ship blocker for simulation mode |
| **CS-T6** | **The perf gate is quiet** | 10 reruns of the instruction-count suite on one commit vary ≤ **0.1%†** per benchmark | Weekly noise job | The benchmark leaves the gate until fixed |
| **CS-T7** | **The perf gate catches what it should** | ≤ **25%†** of lab-confirmed regressions in CPU-bound kernels missed by the per-PR gate, over a rolling 4 quarters, applied once at least 8† such regressions exist | Bisection results joined with gate results | Add benchmarks for the missed paths |
| **CS-T8** | **Fixes carry real regression tests** | **100%** of PRs that close a `bug` issue or carry `Fixes:` have a `Test:` line; ≥ **90%†** have a test that fails on the merge base | `pr-metadata`; `test-fails-on-base` | Repeated misses make `test-fails-on-base` blocking for that area |
| **CS-T9** | **The standard fits T0** | T0 p90 wall time ≤ **20 min** (Stage 1 7.2) over a rolling 2 weeks†, with every CS job within its A.5 budget and each job's share published | CI timings per job, measured from the first Phase 1b month | The job over budget moves to T1 or becomes path-filtered; the budget table is updated in the same PR |
| **CS-T10** | **Unsafe code stays UB-free and bounded** | **0** Miri or loom findings on inventoried modules open > 14 days†; unsafe blocks per inventoried crate published per milestone, and growth needs a decision-log line | T1 `miri` and loom; graph count | Ship blocker for the crate |
| **CS-T11** | **Errors do not leak** | For every API that can fail on network access, CORS failure, DNS failure, TLS failure and connection refused give **byte-identical** exceptions, **100%** | Generated privacy suite (CS-ER-13) in T1 | Ship blocker for the API |
| **CS-T12** | **Clean frames do not allocate** | **0** allocations through the Rust global allocator (renderer script and paint paths, and the GPU service's compositor, WebRender included) in a frame with no DOM, style or input change, from M4; V8's and `cppgc`'s heaps are not counted; a skipped frame meets the target | CS-PF-3 tests | Ship blocker for the regressing PR |
| **CS-T13** | **Exception reasons are true** | A quarterly sample of 20† new tier P `invariant!`, `#[expect]` and `cs-allow` sites, re-reviewed by someone other than the author (until a second maintainer exists: a separate agent session that did not write them, recorded); ≥ **95%†** of reasons true | Sample log in the decision log | The AI review's reason check becomes blocking |
| **CS-T14** | **Generated code stays manageable** | Generated output ≤ **25%†** of tracked repository bytes, and `codegen-fresh` conflicts on ≤ **5%†** of PRs per month, from M2 | Repository statistics; CI | Triggers CS-D4's reversal |

## 9. Milestones, adoption and decisions

### 9.1 Adoption plan

| Phase | When | What applies |
|---|---|---|
| **Spikes: exempt** | Now and after | `m0/` (including the `m0/rusty_v8` checkout), `cppgc-spike/` and `v8-cppgc-ext/` are in the workspace's `exclude` and skipped by the CS checks. They are never imported by an `hg-*` crate; code that moves into Izcal is rewritten under this standard. The fork keeps upstream's conventions for upstream code; Izcal's code in it follows CS-FFI-1 |
| **Phase 1a: the workspace commit, ≤ 3 days†** | With the first M1 crate | Virtual manifest and profiles (A.1); `[workspace.lints]` base; root `clippy.toml` (A.3); Linux `clippy`, `clippy-config` and `doc` jobs; nextest with `retries = 0`; `deny.toml` advisories, licenses and sources; `CODEOWNERS` for the gate files (CS-GA-1). Every new crate starts tier R by default |
| **Phase 1b: tiers, ≤ 4 weeks†, before any tier P crate has non-skeleton code** | Early M1 | `tools/tiers.toml` and the declared-tier check; generated tier tables and tier O `clippy.toml`; `invariant!`; `[bans.build]` with the reviewed allowlist (A.4); `cs-c-unwind`, `cs-pub-trait-future`, `cs-async`, `cs-wrap-in-p`, `cs-std-os`; `pr-metadata`, `exceptions-listed`, `gate-config-changed`; macOS and Windows clippy; `rules.toml` and the review context. Code written between 1a and 1b is re-linted when 1b lands, which is cheap while it is small |
| **Phase 2: during M1** | As `hg-ipc-codegen`, `hg-substrate-*` and the first tests land | CS-ER-6 and Kani in the IPC generator; endpoint capacity and overflow policy (CS-AS-6 (b)); remaining `ast-grep` rules; CS-AS-13 inventory with pre-filled scans; `hg-lint-canary`; Miri, loom and the pinned nightly; `test-verify`, quarantine, `test-fails-on-base`; fuzz-target registry; typed lock levels; the diagnostics facade and `log` adapter; per-job T0 budgets measured |
| **Phase 3: M2–M4** | Bindings (M2), style (M3), layout (M4) | CS-V8 rules, the fork's range unwrap and CS-ER-13 (M2, with SEC-11); `TokenCell` prototype and CS-D3's exit criteria (M2); CS-V8-9 with the sweep stress suite (M2); generated-code repository rules (M2); size budgets as hot types appear; perf gate with the first kernels (M3, M4); allocation budgets; CS-D2 measurement (M4) |
| **Phase 4: M5 onward** | Network and navigation | CS-ER-7 and CS-ER-9 in the network service and updater; CS-D1's network-service outcome applied; CS-AS-7's budget becomes failing once calibrated; `cargo-semver-checks` when the embedding API exists (Stage 2) |

### 9.2 Staffing

The Infrastructure track (Stage 1 8.3) owns the tooling: about **3 days†** for Phase 1a, **3–4 engineer-weeks†** for Phase 1b (including the review of the 65-crate build-script allowlist), and **6–10 engineer-weeks†** across Phases 2–3. Generator rules fall to the Substrate and IPC and the JS, bindings and DOM tracks as part of building the generators; the range-unwrap binding and CS-FFI-1 to the `rusty_v8` owner. No new track is needed.

### 9.3 Decisions for the owner

| ID | Decision | Options | Recommendation | Reversal trigger |
|---|---|---|---|---|
| **CS-D1** | **Panic strategy** (conflicts with security scope 0.2 IPC-R5 and NET-R3) | (a) abort in every shipping and DST binary; (b) abort everywhere except the network service, which unwinds with a `catch_unwind` per connection (security scope 0.2); (c) unwind generally (Servo's model) | **(a)**. The network service holds every site's cookies, caches and sockets, so continuing after a panic risks corrupting state every site shares (a poisoned lock or half-updated cache), the bug-continuation Midori rejected; tier P lints, `invariant!` and SEC-28's fuzzing make network panics unreachable rather than contained, and the broker restarts a crashed service. **If (b) is chosen:** `hg-net-main` builds with its own profile (`[profile.release-net] inherits = "release"`, `panic = "unwind"`, verified to build), in its own invocation, so its dependencies compile twice in release; `catch_unwind` is allowed in exactly one `hg-net` module through a generated `#[expect]`; tier P lints are unchanged; V8 is not linked there, so `extern "C"` aborts do not arise | Network-service fuzzing after M5 keeps finding panics in dependencies upstream cannot fix within SEC-28's 14 days: isolate those dependencies in a separate process (keeps (a)), or choose (b) |
| **CS-D2** | **Overflow checks in tier R release builds** (tier P is on by rule) | (a) on; (b) off, on in the `checked` profile | **(a) if the measured cost is ≤ 1%†** on Speedometer 3 and P-1 at M4, else (b). The measurement includes `hg-types`, which is tier P and so overflow-checked in every process, its checks inlined into renderer hot paths. CS-D2 and DET-2's 1% substrate budget draw on the same P-1 headroom; CS-D2 gives way first | The cost crosses 1%† at any re-measurement (each stage boundary) |
| **CS-D3** | **Re-entrancy safety in DOM crates** | (a) `TokenCell`/`ScriptToken`, compile-time; (b) `RefCell` plus a custom lint and GC-stress fuzzing (Servo's approach) | **(a)**, prototyped at M2 on `Node`, `Element` and mutation observers, with **soundness exit criteria**: (1) the token is derivable only from `&mut HandleScope` (or `PinScope`), or from a generated callback prologue that consumes the scope V8 passes in, so an inner callback's token provably means the outer one is lent out; whether `ScriptToken` is a separate type or the scope itself is decided here, since two tokens can drift; (2) every rusty_v8 entry point that can run script (`Function::call`, `Script::run`, property get and set, and the rest found by auditing the API) is reachable only through token-taking wrappers in `hg-script`, enforced by `disallowed-methods` and the canary; (3) traced fields (`Member` links) are never inside `TokenCell`, which the generator enforces, because the concurrent marker reads them without the token; (4) a written multi-node mutation pattern (scoped multi-borrow, clone-out or per-node split), exercised on `insertBefore` with a `MutationObserver` and a custom-element reaction; (5) Miri and loom tests of the cell API | Any criterion fails, or the prototype costs > 2%† on the M2 bindings benchmarks: fall back to (b) |
| **CS-D4** | **Where generated code lives** | (a) checked in, with a freshness check and CS-ST-17's repository rules; (b) `build.rs` into `OUT_DIR` | **(a)** | CS-T14 fails for a quarter: move generation to an `xtask` step that writes outside `src/`, still without `build.rs` |
| **CS-D5** | **Per-PR perf gate platform** | (a) `gungraun` on GitHub-hosted Linux; (b) hardware counters on a dedicated Linux machine; (c) no per-PR gate | **(a)**: deterministic on virtual runners; SEC-27 forbids self-hosted runners on PRs | `gungraun` cannot run a needed kernel, or the gate exceeds its CS-T9 budget: move those benchmarks to (b) in T1, from `main` only |
| **CS-D6** | **Separate-session tests for AI-generated algorithms** (CS-TE-17) | (a) mandatory for generated code implementing web-visible behavior; (b) advisory | **(a)**, with the mechanical form in CS-TE-17 | For two quarters†, separate-session tests catch no defect that the implementation session's own tests missed (from `test-fails-on-base` and mutation reports): make it advisory |
| **CS-D7** | **Tier of the utility process** | (a) tier R; (b) tier P | **(a)**: the security scope's utility threat model keeps "one (site, container) per process at a time when outputs could leak across sites" | That rule relaxes to multi-site utility processes: move `hg-utility-main` to tier P |
| **CS-D8** | **The async model** | (a) closures on sequences by default; `async` only in allowlisted crates; JS Promises never bridged to Rust futures; (b) `async` throughout engine crates on a substrate executor | **(a)** (CS-AS-4): it matches Stage 1's synchronous substrate, keeps every waker owned by the substrate, and removes futurelock and cancel-safety from engine code | A protocol dependency the network service needs cannot run sans-IO (Q-CS-2): add that crate to the allowlist, not (b) |
| **CS-D9** | **How tier lint levels are applied** | (a) declared tiers, generated full `[lints]` tables and a tier O `clippy.toml`; (b) `[lints] workspace = true` everywhere plus a generated, marker-delimited `#![deny(...)]` block at each tier P crate root; (c) tiers computed from linkage | **(a)**: one inheritance rule per tier, config outside source files (so CS-GA-1 can own it), and no silent tier moves (verified to work, A.7) | The generator's maintenance cost exceeds (b)'s at the M2 review |
| **CS-D10** | **Phase 1 split** | (a) Phase 1a (≤ 3 days†) then 1b (≤ 4 weeks†); (b) everything before the first M1 commit (0.1) | **(a)**: keeps the standard off M1's critical path (Stage 1 8.2) while the code is small enough to re-lint | Phase 1b slips past the first non-skeleton tier P code: that code waits for 1b |

### 9.4 Amendments this standard proposes to other documents (not applied)

| Document | Amendment |
|---|---|
| Stage 1 2.2, G3 | G3 binds test targets too; extend the list per CS-AS-1 (including `std::env` and `std::os`); tier T tools are outside its scope |
| Stage 1 3.1, Section 8 | Record CS-D8: tasks are run-to-completion closures; `async` only in allowlisted crates; JS Promises are never bridged to Rust futures. Answer Q-CS-1 in the M1 exit text |
| Stage 1 5.3, HGBA-2 4.8 | Add the fork's range-unwrap binding (CS-V8-1) to the fork's planned divergence (S-4e), and CS-V8-9 to the node design's payload and prefinalizer text |
| Stage 1 7.1 | Add the `checked` profile; shipping and DST builds abort; `dev` unwinds |
| Stage 1 7.2 | T0 gains the CS jobs in A.5 with their budgets and path filters; clippy runs on three OSes |
| Security scope 0.2 IPC-R5 (panic-strategy clause), NET-R3, SEC-28 | If CS-D1 (a) is accepted: "abort in every shipping process; the network service meets SEC-28 by making panics unreachable from network input, and a crash restarts the service"; SEC-28's last clause becomes "no panic reachable from network input". If (b) is accepted, this standard adopts CS-D1 (b)'s mechanism instead |
| Security scope 0.2 IPC-R5 and SEC-7 (b) | Name the allowlist file: the generated `tools/coding-standard/tier-p-exceptions.toml` (CS-ER-3) |
| DevTools spec G12 | Note that `hg-substrate-sim` follows tier R and G3 because it ships in `hg-replay-main` |
| `CONTRIBUTING.md` | Add a "Coding standard" section linking this document; trailers in the PR body; the squash setting "pull request title and description"; `// cs-allow(...)` markers; the agent rules (CS-GA-2) |

0.1's amendment replacing IPC-R5's "lint errors on message-derived values" is withdrawn: security scope 0.2 already made that change.

## 10. Risks

| Risk | Mitigation |
|---|---|
| Tier P lints make broker and network code slow to write | `checked_*` plus `?` and `invariant!` are cheap; CS-T2 watches the count |
| Agents satisfy lints with plausible but false reasons | CS-GA-2, CS-GA-3 listing, AI review of reasons, CS-T13 sampling |
| Nursery lints change or misfire across toolchain bumps | Pinned toolchain and a scheduled bump PR with the lint diff (CS-ST-9) |
| The token-cell design does not fit the DOM or is unsound | CS-D3's exit criteria, with a fallback |
| `disallowed-*` paths stop matching after a rename or are misspelled | `clippy-config` fails on configuration warnings; `hg-lint-canary` fails when an expected lint stops firing |
| Determinism leaks through dependencies | CS-AS-13 pre-filled inventory plus the CS-T5 meta-test |
| The build-script allowlist grows unreviewed | Each entry is a SEC-14 (d) decision under `CODEOWNERS`; bypasses are pinned to build-script hashes |
| Instruction counts miss memory-bound regressions | The lab remains the authority (CS-PF-5); CS-T7 |
| T0 budget overrun | Per-job budgets, path filters, T1 placement (A.5), CS-T9 |
| The CS-D1 conflict with the security scope stays open | Both specs list it; it is decided before network-service code exists (M5) |

## 11. Open questions and flagged claims

### 11.1 Open questions

- **Q-CS-1 (for the Stage 1 owner). Which binaries link `hg-substrate-sim`?** Stage 1's M1 exit says "Browser, renderer and network processes start under both substrates", and DevTools G12 ships it in `hg-replay-main`. **Recommendation:** process logic lives in libraries whose entry function takes a `Substrate`; the shipping `hg-*-main` binaries link only `hg-substrate-os`; one simulation binary (run by `hg-dst`) and `hg-replay-main` link `hg-substrate-sim`; the graph lint enforces that list. Reason: a seeded `Entropy::crypto()` (Stage 1 3.1) should not exist in a binary that touches the network. Until answered, `hg-substrate-sim` is tier R with G3 applying.
- **Q-CS-2. HTTP/2.** The `h2` crate builds on tokio's I/O traits. Izcal writes its own HTTP/2 codec from RFC 9113 as a sans-IO state machine (the standing build-our-own answer), or allowlists a protocol driver crate under CS-AS-4. To be settled with the network service design before M5.
- **Q-CS-3. Proptest seeding.** Which configuration gives a fixed, printed seed per run (CS-TE-10) was not checked. Confirm at M1, or wrap the runner.
- **Q-CS-4. Which thread runs `cppgc` destructors** under `kIncrementalAndConcurrent` sweeping. The public headers do not say; `IsSweepingOnOwningThread` exists so code can ask whether it "has been called from a destructor". Settle at M2 by test, and decide whether Izcal configures `sweeping_support = kIncremental` (the fork exposes it) to keep payload `Drop` on the owning thread, at a measured cost.
- **Q-CS-5. Valgrind on V8-linked code.** Moot for the gate (CS-PF-4 keeps V8 out of benchmark crates), open for lab profiling.
- **Q-CS-6. T0 job costs.** The A.5 budgets are first-principles numbers until Phase 1b measures them.

### 11.2 Flagged claims

| Claim | Why flagged |
|---|---|
| Servo recovers from pipeline panics by reloading in place | From PR #30290's description, not confirmed merged |
| ArrayBuffer allocation failure throws `RangeError` | Recalled from ECMAScript, not re-fetched |
| Overflow checks in `hg-types` are inlined into renderer hot paths | Follows from where Rust inserts the checks for non-generic code; not measured |
| Fuchsia `lock_order` gives compile-time deadlock freedom | Module documentation only |
| Stylo never touches `STYLE_THREAD_POOL` when the embedder passes a pool | Read in the stylo 0.22.0 source; not tested at run time |
| The Quantum DOM scheduler's removal *because of* other projects (Fission, Spectre) | Inference; the sources show only that it was unused and removed in Firefox 66 |
| The 65-crate allowlist represents the M1 graph | Measured on a 12-crate L0 probe with `rustls`'s default `aws-lc-rs` provider; rerun on the real M1 graph in Phase 1b |
| Every † value in this document | Set from first principles; revisit after M2 when T0 has history |

---

## Appendix A. Enforcement

### A.1 Workspace `Cargo.toml`: lints (tier R base) and profiles

Exact proposed contents, verified on rustc and clippy 1.99.1 (A.7).

```toml
[workspace]
resolver = "3"
members = ["crates/*", "xtask"]
exclude = ["m0", "cppgc-spike", "v8-cppgc-ext"]   # spikes and the m0/rusty_v8 fork checkout (Section 9.1)

[workspace.package]
edition = "2024"
rust-version = "1.99"            # equals rust-toolchain.toml (CS-ST-9)
license = "MIT"
publish = false

# Tier R lint set, used by `[lints] workspace = true`. Tiers P, O and T get a full
# generated table (A.2), because Cargo rejects inheritance plus local keys.
# Edition 2024 already rejects unsafe attributes and extern blocks without `unsafe`.
[workspace.lints.rust]
unsafe_code = "deny"                       # G5; inventoried modules: #[expect(unsafe_code, ...)] (CS-TY-13)
unsafe_op_in_unsafe_fn = "deny"            # CS-TY-13 (edition 2024 default is warn)
unused_must_use = "deny"                   # was CS-ER-11
async_fn_in_trait = "deny"                 # CS-AS-4, partial: the hook catches `-> impl Future`
ffi_unwind_calls = "deny"                  # CS-ER-17; fires only in unwinding (test and dev) builds
unreachable_pub = "warn"                   # was CS-ST-4
unnameable_types = "warn"                  # was CS-ST-4
missing_debug_implementations = "warn"     # CS-ER-14
elided_lifetimes_in_paths = "warn"         # CS-TY-10: scope lifetimes stay visible
variant_size_differences = "warn"          # was CS-PF-2
unexpected_cfgs = { level = "warn", check-cfg = ["cfg(loom)", "cfg(kani)", "cfg(fuzzing)"] }

[workspace.lints.clippy]
allow_attributes = "deny"                  # 6.1 (was CS-ER-4)
allow_attributes_without_reason = "deny"   # 6.1
unwrap_used = "warn"                       # CS-ER-1
todo = "warn"
unimplemented = "warn"
let_underscore_must_use = "deny"           # was CS-ER-11
unused_result_ok = "warn"
map_err_ignore = "warn"                    # CS-ER-10
error_impl_error = "warn"
fallible_impl_from = "warn"                # CS-TY-3
undocumented_unsafe_blocks = "deny"        # CS-TY-13
multiple_unsafe_ops_per_block = "warn"
unnecessary_safety_comment = "warn"
unnecessary_safety_doc = "warn"
ptr_as_ptr = "warn"
ref_as_ptr = "warn"
transmute_undefined_repr = "warn"
non_send_fields_in_send_ty = "warn"        # CS-TY-9
arc_with_non_send_sync = "deny"
fn_params_excessive_bools = "warn"         # CS-TY-4
struct_excessive_bools = "warn"
disallowed_methods = "deny"                # CS-AS-1 and the A.3 lists
disallowed_types = "deny"
disallowed_macros = "deny"
await_holding_lock = "deny"                # CS-AS-8
await_holding_refcell_ref = "deny"
await_holding_invalid_type = "deny"
let_underscore_future = "deny"             # CS-AS-4
unused_async = "warn"                      # CS-D8
large_futures = "warn"
iter_over_hash_type = "warn"               # CS-AS-2
significant_drop_tightening = "warn"       # CS-AS-9
print_stdout = "warn"                      # CS-ER-14
print_stderr = "warn"
dbg_macro = "warn"
mod_module_files = "warn"                  # CS-ST-1
too_many_lines = "warn"                    # was CS-ST-11
negative_feature_names = "warn"            # CS-ST-6
redundant_feature_names = "warn"
large_types_passed_by_value = "warn"       # was CS-PF-2 (the perf group is on by default)
large_stack_arrays = "warn"
large_stack_frames = "warn"
trivially_copy_pass_by_ref = "warn"
inline_always = "warn"
unnecessary_box_returns = "warn"
redundant_clone = "warn"
needless_collect = "warn"

[workspace.lints.rustdoc]                  # run only by `cargo doc` (A.5 `doc` job)
broken_intra_doc_links = "deny"
private_intra_doc_links = "warn"
missing_crate_level_docs = "warn"          # CS-ST-13

# CS-ER-2. Test builds always unwind (Cargo forces it); `dev` matches them so
# dependencies build once (A.7). Every profile that builds a shipping or DST binary aborts.
[profile.dev]
panic = "unwind"

[profile.release]
panic = "abort"
lto = "fat"
codegen-units = 1

# DST, fuzzing and T0 assertions (CS-ER-1): release speed, debug checks.
[profile.checked]
inherits = "release"
debug-assertions = true
overflow-checks = true

# CS-TY-12: tier P and O crates check overflow in release. Generated from tools/tiers.toml.
[profile.release.package.hg-types]
overflow-checks = true
```

A comment marked "was" names a retired 0.1 ID (Appendix C), so existing reasons that cite it stay traceable. Removed from 0.1's table: `unsafe_attr_outside_unsafe` and `missing_unsafe_on_extern` (hard errors in edition 2024, so the lints enforce nothing), `mem_forget`, `exit` (covered by the `std::process::exit` ban), `wildcard_imports` and `wildcard_dependencies` (cargo-deny's `wildcards`), which no rule cited; `missing_docs`, `missing_errors_doc` and `missing_panics_doc` moved to the tier P table (CS-ST-13).

### A.2 Per-tier lint tables

`tools/lints.toml` holds the base table (A.1) and the deltas below. `cargo xtask lints` writes, into each tier P, O and T manifest, a full `[lints.rust]`, `[lints.clippy]` and `[lints.rustdoc]` table equal to the base plus its delta, and into `crates/hg-substrate-os/` a generated `clippy.toml`; T0's `lints-fresh` fails if any differs.

- **Tier P and O additions** (all `deny`): `clippy::expect_used`, `panic`, `unreachable`, `indexing_slicing`, `string_slice`, `get_unwrap`, `arithmetic_side_effects`, `unwrap_in_result`, `panic_in_result_fn`, `as_conversions`, `cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`, `cast_lossless`, `checked_conversions`, `wildcard_enum_match_arm`; raised to `deny`: `unwrap_used`, `todo`, `unimplemented`, `map_err_ignore`; at `warn`: rustc `missing_docs`, clippy `missing_errors_doc`, `missing_panics_doc`.
- **Tier O's `clippy.toml`:** the root file minus every array entry whose reason starts with `G3`, `CS-AS-3`, `CS-AS-4 select` or `CS-AS-6 bounded`. Everything else, including `catch_unwind`, `set_hook`, unbounded channels, async mutexes and the `v8` entries, stays.
- **Tier T changes:** `clippy::unwrap_used`, `todo`, `print_stdout`, `print_stderr`, `dbg_macro`, `disallowed_methods`, `disallowed_types`, `disallowed_macros` = `allow`. `hg-lint-canary` keeps the base table, so its expectations are checked.
- **Expectations the graph lint permits at crate or module level** (and no others): `#[expect(unsafe_code, …)]` on modules listed in `unsafe-inventory.toml`; the expectations on the generators' list (CS-ST-17 (c)); the `hg_types` hash aliases; the one `#[expect(clippy::panic)]` on `hg_types::invariant_failed`.

The sanctioned invariant path (verified: no lint fires at use sites in a tier P `Result` function, while `assert!` there is flagged by `panic_in_result_fn`):

```rust
/// Aborts with a literal message. The only panic site tier P code reaches.
///
/// # Panics
/// Always; the process aborts (CS-ER-2).
#[cold]
#[inline(never)]
#[track_caller]
#[expect(clippy::panic, reason = "CS-ER-1: the one sanctioned invariant-failure path")]
pub fn invariant_failed(msg: &'static str) -> ! {
    panic!("invariant: {msg}")
}

/// `invariant!(cond, "literal")`: a broken internal invariant, never an input error.
#[macro_export]
macro_rules! invariant {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            $crate::invariant_failed($msg)
        }
    };
}
```

### A.3 `clippy.toml` (workspace root)

```toml
msrv = "1.99"
avoid-breaking-exported-api = false      # internal crates have no semver (CS-ST-14)
# Size and length thresholds stay at clippy's defaults (futures 16384 B, errors 128 B, 100 lines).

# test code may panic (Section 6.2)
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
allow-print-in-tests = true

# CS-TY-12: float-based unit newtypes may use operators in tier P
arithmetic-side-effects-allowed = ["hg_types::CssPx", "hg_types::DevicePx"]

await-holding-invalid-types = [          # CS-AS-8; paths fixed when the types exist
  { path = "hg_dom::gc::NoGcScopeGuard", reason = "CS-AS-8: GC scope guard (S-4c)" },
  { path = "hg_substrate::SequenceBoundRef", reason = "CS-AS-8" },
  { path = "hg_dom::cell::TokenRef", reason = "CS-TY-10" },
  { path = "hg_dom::cell::TokenRefMut", reason = "CS-TY-10" },
]

# Entries whose reason starts "G3", "CS-AS-3" or "CS-AS-4 select" are dropped from hg-substrate-os's generated copy,
# and so are bounded-channel entries ("CS-AS-6 bounded"); every other entry stays (A.2).
disallowed-methods = [
  # G3 / CS-AS-1: ambient effects (replacement: the substrate)
  { path = "std::time::Instant::now", reason = "G3", replacement = "hg_substrate::Clock::monotonic" },
  { path = "std::time::SystemTime::now", reason = "G3", replacement = "hg_substrate::Clock::wall" },
  { path = "std::thread::spawn", reason = "G3", replacement = "hg_substrate::Executor::post" },
  { path = "std::thread::scope", reason = "G3", replacement = "hg_substrate::Executor::fork_join" },
  { path = "std::thread::sleep", reason = "G3" },
  { path = "std::thread::park_timeout", reason = "G3" },
  { path = "std::thread::available_parallelism", reason = "G3" },
  { path = "std::process::exit", reason = "G3: exit belongs to the substrate" },
  { path = "std::net::ToSocketAddrs::to_socket_addrs", reason = "G3: DNS belongs to the network service" },
  { path = "std::fs::read", reason = "G3" },
  { path = "std::fs::read_to_string", reason = "G3" },
  { path = "std::fs::write", reason = "G3" },
  { path = "std::fs::read_dir", reason = "G3" },
  { path = "std::fs::metadata", reason = "G3" },
  { path = "std::fs::symlink_metadata", reason = "G3" },
  { path = "std::fs::exists", reason = "G3" },
  { path = "std::fs::create_dir", reason = "G3" },
  { path = "std::fs::create_dir_all", reason = "G3" },
  { path = "std::fs::remove_file", reason = "G3" },
  { path = "std::fs::remove_dir", reason = "G3" },
  { path = "std::fs::remove_dir_all", reason = "G3" },
  { path = "std::fs::rename", reason = "G3" },
  { path = "std::fs::copy", reason = "G3" },
  { path = "std::fs::hard_link", reason = "G3" },
  { path = "std::fs::soft_link", reason = "G3" },
  { path = "std::fs::read_link", reason = "G3" },
  { path = "std::fs::set_permissions", reason = "G3" },
  { path = "std::fs::canonicalize", reason = "G3" },
  { path = "std::env::var", reason = "G3" },
  { path = "std::env::var_os", reason = "G3" },
  { path = "std::env::vars", reason = "G3" },
  { path = "std::env::vars_os", reason = "G3" },
  { path = "std::env::set_var", reason = "G3" },
  { path = "std::env::remove_var", reason = "G3" },
  { path = "std::env::args", reason = "G3" },
  { path = "std::env::args_os", reason = "G3" },
  { path = "std::env::current_dir", reason = "G3" },
  { path = "std::env::set_current_dir", reason = "G3" },
  { path = "std::env::current_exe", reason = "G3" },
  { path = "std::env::temp_dir", reason = "G3" },
  { path = "std::env::home_dir", reason = "G3" },
  { path = "rand::thread_rng", reason = "G3", replacement = "hg_substrate::Entropy::stream" },
  { path = "rand::rng", reason = "G3", replacement = "hg_substrate::Entropy::stream" },
  { path = "rand::random", reason = "G3", replacement = "hg_substrate::Entropy::stream" },
  { path = "getrandom::fill", reason = "G3", replacement = "hg_substrate::Entropy::crypto" },
  { path = "getrandom::getrandom", reason = "G3 (getrandom 0.2)", replacement = "hg_substrate::Entropy::crypto" },
  { path = "num_cpus::get", reason = "G3" },
  # CS-AS-3: the substrate owns every thread
  { path = "rayon::spawn", reason = "CS-AS-3" },
  { path = "rayon::spawn_fifo", reason = "CS-AS-3" },
  { path = "rayon::join", reason = "CS-AS-3: global pool", replacement = "hg_substrate::PhasePool" },
  { path = "rayon::scope", reason = "CS-AS-3: global pool", replacement = "hg_substrate::PhasePool" },
  { path = "rayon::ThreadPoolBuilder::build_global", reason = "CS-AS-3" },
  { path = "rayon::ThreadPoolBuilder::build", reason = "CS-AS-3" },
  # CS-AS-4: no third-party executor
  { path = "futures::executor::block_on", reason = "CS-AS-4" },
  { path = "futures::executor::block_on_stream", reason = "CS-AS-4" },
  # CS-AS-6: queues are substrate primitives; unbounded ones exist nowhere
  { path = "std::sync::mpsc::channel", reason = "CS-AS-6 unbounded" },
  { path = "std::sync::mpsc::sync_channel", reason = "CS-AS-6 bounded: blocking send", replacement = "hg_substrate::Executor::post" },
  { path = "crossbeam_channel::unbounded", reason = "CS-AS-6 unbounded" },
  { path = "crossbeam_channel::bounded", reason = "CS-AS-6 bounded", replacement = "hg_substrate::Executor::post" },
  { path = "flume::unbounded", reason = "CS-AS-6 unbounded" },
  { path = "flume::bounded", reason = "CS-AS-6 bounded", replacement = "hg_substrate::Executor::post" },
  { path = "futures::channel::mpsc::unbounded", reason = "CS-AS-6 unbounded" },
  { path = "futures::channel::mpsc::channel", reason = "CS-AS-6 bounded", replacement = "hg_substrate::Executor::post" },
  { path = "tokio::sync::mpsc::unbounded_channel", reason = "CS-AS-6 unbounded" },
  { path = "tokio::sync::mpsc::channel", reason = "CS-AS-6 bounded", replacement = "hg_substrate::Executor::post" },
  # CS-ER-2, CS-ER-14, CS-ER-10
  { path = "std::panic::catch_unwind", reason = "CS-ER-2" },
  { path = "std::panic::resume_unwind", reason = "CS-ER-2" },
  { path = "std::panic::set_hook", reason = "CS-ER-14: crash handler only" },
  { path = "std::panic::take_hook", reason = "CS-ER-14: crash handler only" },
  { path = "std::backtrace::Backtrace::capture", reason = "CS-ER-10: crash handler only" },
  { path = "std::backtrace::Backtrace::force_capture", reason = "CS-ER-10: crash handler only" },
  { path = "log::set_logger", reason = "CS-ER-14: the substrate installs the log adapter" },
  { path = "log::set_boxed_logger", reason = "CS-ER-14: the substrate installs the log adapter" },
  # CS-V8-3 / FFI-R1: no raw pointers in V8 objects (silent until hg-lint-canary links v8)
  { path = "v8::External::new", reason = "CS-V8-3/FFI-R1: typed wrapper only" },
  { path = "v8::Object::set_aligned_pointer_in_internal_field", reason = "CS-V8-3" },
  { path = "v8::Object::set_internal_field", reason = "CS-V8-3" },
]

disallowed-types = [
  { path = "std::thread::Builder", reason = "G3" },
  { path = "std::process::Command", reason = "G3" },
  { path = "std::time::Instant", reason = "G3", replacement = "hg_types::Instant" },
  { path = "std::time::SystemTime", reason = "G3", replacement = "hg_types::SystemTime" },
  { path = "std::fs::File", reason = "G3" },
  { path = "std::fs::OpenOptions", reason = "G3" },
  { path = "std::fs::DirBuilder", reason = "G3" },
  { path = "std::net::TcpStream", reason = "G3" },
  { path = "std::net::TcpListener", reason = "G3" },
  { path = "std::net::UdpSocket", reason = "G3" },
  { path = "std::collections::HashMap", reason = "CS-AS-2", replacement = "hg_types::HashMap" },
  { path = "std::collections::HashSet", reason = "CS-AS-2", replacement = "hg_types::HashSet" },
  { path = "std::hash::RandomState", reason = "CS-AS-2" },
  { path = "hashbrown::HashMap", reason = "CS-AS-2", replacement = "hg_types::HashMap" },
  { path = "hashbrown::HashSet", reason = "CS-AS-2", replacement = "hg_types::HashSet" },
  { path = "indexmap::IndexMap", reason = "CS-AS-2", replacement = "hg_types::IndexMap" },
  { path = "indexmap::IndexSet", reason = "CS-AS-2", replacement = "hg_types::IndexSet" },
  { path = "ahash::RandomState", reason = "CS-AS-2" },
  { path = "ahash::AHashMap", reason = "CS-AS-2", replacement = "hg_types::HashMap" },
  { path = "futures::executor::LocalPool", reason = "CS-AS-4" },
  { path = "tokio::sync::Mutex", reason = "CS-AS-8" },
  { path = "tokio::sync::RwLock", reason = "CS-AS-8" },
  { path = "async_lock::Mutex", reason = "CS-AS-8" },
  { path = "futures::lock::Mutex", reason = "CS-AS-8" },
]

disallowed-macros = [
  { path = "tokio::select", reason = "CS-AS-4 select: futurelock" },
  { path = "futures::select", reason = "CS-AS-4 select" },
  { path = "futures::select_biased", reason = "CS-AS-4 select" },
]
```

Measured behavior (A.7): `std::fs::File` as a type ban also covers `File::open` and `File::create`; the hash aliases are flagged only at their definition; paths into crates the linted crate does not depend on (here every `v8::` and `hg_*` path) give no diagnostic, which is why `hg-lint-canary` exists; a misspelled reachable path or a feature-gated item that is off (`tokio::select` without tokio's `macros` feature) gives a configuration warning, which `clippy-config` turns into a failure, so an entry that cannot resolve in a tier's crates is dropped from that tier's generated copy.

### A.4 `deny.toml` (replaces `tools/deny.toml`; `[advisories]` is today's)

Verified with cargo-deny 0.20.2 on a probe workspace whose one crate depends on the real L0 set: `v8` 152.2.0, `stylo` 0.22.0, `selectors` 0.38, `taffy` 0.9, `parley` 0.12, `html5ever` 0.39, `webrender` 0.70, `wgpu` 30, `rayon` 1, `rustls` 0.23, `thiserror` 2, `accesskit` 0.25, with `proptest`, `insta`, `loom`, `bolero`, `gungraun` and `divan` as dev-dependencies (457 packages). `cargo deny check` passed: advisories, bans, licenses and sources. The license list is `CONTRIBUTING.md`'s ("MIT, Apache-2.0, BSD, ISC, Zlib, Unicode or MPL-2.0") written as SPDX identifiers; `Apache-2.0 WITH LLVM-exception` is this document's addition.

```toml
[graph]
targets = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-pc-windows-msvc"]
all-features = true

[advisories]
version = 2
unmaintained = "none"
yanked = "warn"

[licenses]
version = 2
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause",
         "BSD-3-Clause", "ISC", "Zlib", "Unicode-3.0", "MPL-2.0"]
confidence-threshold = 0.93
unused-allowed-license = "allow"
private = { ignore = true }

[bans]
multiple-versions = "warn"
wildcards = "deny"
allow-wildcard-paths = true
deny = [
  { crate = "async-std", reason = "CS-AS-4: discontinued" },
  { crate = "async-trait", reason = "CS-AS-4: public contracts are synchronous" },
  { crate = "tokio", wrappers = ["hg-substrate-os", "hg-lint-canary"], reason = "CS-AS-4: one scheduler" },
  { crate = "smol", wrappers = ["hg-substrate-os"], reason = "CS-AS-4" },
]

[bans.build]
allow-build-scripts = ["anyhow", "ash", "aws-lc-rs", "aws-lc-sys", "bincode-next", "bindgen",
"clang-sys", "crc32fast", "crossbeam-deque", "crossbeam-epoch", "crossbeam-utils", "encoding_rs",
"freetype-sys", "generator", "getrandom", "gleam", "glslopt", "glutin_wgl_sys", "gungraun",
"gungraun-common", "gungraun-macros", "icu_calendar_data", "icu_locale_fallback_data",
"icu_normalizer_data", "icu_properties_data", "icu_segmenter_data", "khronos_api", "khronos-egl",
"libc", "libm", "libz-sys", "naga", "num-traits", "objc2", "parking_lot_core", "paste",
"prettyplease", "proc-macro2", "quote", "rayon-core", "rustix", "rustls", "rustversion",
"selectors", "serde", "serde_core", "serde_json", "slotmap", "stylo", "stylo_atoms",
"stylo_static_prefs", "thiserror", "v8", "wayland-sys", "web_atoms", "webrender", "wgpu",
"wgpu-core", "wgpu-hal", "winapi", "windows_x86_64_gnu", "windows_x86_64_msvc",
"yeslogic-fontconfig-sys", "zerocopy", "zmij"]   # measured starting list; each entry a SEC-14 (d) decision
executables = "deny"
interpreted = "deny"
include-dependencies = false          # scan the crates that run at build time, not their whole subtrees
include-workspace = true
include-archives = false              # Windows import libraries (windows_x86_64_*) are linked, not run
bypass = [
  { crate = "v8@152.2.0", build-script = "47d307ac1df6338dd418d57e286f96b3cf9fcd324579b28d754c56302dab7623" },
  { crate = "stylo@0.22.0", build-script = "625c8b0c05491ff67873a7eb63f6347702830683f63b3d554ad1db3d89ad4427" },
  { crate = "aws-lc-sys@0.46.0", build-script = "8144aba9f6165a3f0dcb76547cf788d78fbcd4cd12a525f315fc6d4756ea19fb" },
  { crate = "aws-lc-rs@1.18.2", build-script = "791f97e6abdb685bcc722342d49f2646b3146b209f68753669c178716f6cee01" },
  { crate = "freetype-sys@0.23.0", build-script = "ef01f6ea9e4cb2db8a9cde77e242574ef5693877fb59deb5ec6d77dc840aa3d8" },
  { crate = "glslopt@0.1.14", build-script = "92e29699487c21389fc823c50b410cfbfc74cc774265d24958489909ee48a076" },
  { crate = "zerocopy@0.8.63", allow-globs = ["cargo.sh", "ci/*.sh"] },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = ["https://github.com/Rorqualx/rusty_v8-izcal"]
```

Notes. Without the bypasses, the scan reported 409 executable scripts once build scripts were allowed, 213 of them in `v8` (the `build/`, `third_party/` and `tools/` trees) and 70 in `aws-lc-sys`; a `build-script` checksum bypass skips a crate's scan only while its `build.rs` is unchanged. The `v8` entry becomes the fork's version and hash when the fork is consumed from git. `wrappers` applies to direct dependents only. Rules about Izcal's own crates (`anyhow` only in tier T, `rand` only via the substrate) live in `hg-graph-lint`, which knows the tiers. The deny check is cached by `Cargo.lock` hash.

### A.5 Hook and CI steps

**`tools/checks.sh` checks** (same script for hooks and CI; `--staged` runs the fast ones, `--range` all):

| Check | Mode | Fails when | Rule |
|---|---|---|---|
| `clippy` | `--range`; `--staged` with `IZCAL_CLIPPY=1` | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` reports anything | CS-T1 |
| `clippy-config` | with `clippy` | any clippy diagnostic is located in a `clippy.toml` (`--> …clippy.toml:<line>`), such as "does not refer to a reachable function" (verified to catch a misspelling that exits 0) | CS-T1 |
| `doc` | `--range` | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items` fails (verified to fail on a broken link that clippy passes) | CS-ST-13, CS-T1 |
| `deny` | both | `cargo deny check` fails | CS-ST-7, SEC-14 |
| `graph` | both | `hg-graph-lint` reports a G1–G8 or CS violation (A.6) | Many |
| `lints-fresh`, `exceptions-fresh`, `rules-index-fresh` | both | a generated table, tier `clippy.toml`, `tier-p-exceptions.toml` or the rule index is stale | CS-ST-10, CS-ER-3, CS-GA-4 |
| `codegen-fresh` | `--range` | `cargo xtask codegen --check` changes a generated file | CS-ST-17 |
| `cs-patterns` | both | an `ast-grep` rule in `tools/ast-grep/` matches an added hand-written line without a `cs-allow` marker, or a marker's line no longer matches | A.6 |
| `file-length` | both | a hand-written `.rs` file exceeds 2000† lines without a line-1 marker with a reason | was CS-ST-11 |
| `toolchain` | both | `rust-toolchain.toml`, the nightly pin and `rust-version` disagree | CS-ST-9 |

When `clippy` runs, `security`'s `awk` SAFETY heuristic remains for C++ and shell files and for code outside the workspace; for workspace Rust, `undocumented_unsafe_blocks` replaces it. `commit-msg` keeps checking subjects; trailers move to the PR body (CS-GA-3).

**CI jobs by tier, with T0 budgets** (adds to Stage 1 7.2; budgets are wall-clock seconds† per job on GitHub-hosted runners, jobs in parallel, measured in Phase 1b):

| Tier | Job | Runs when | Budget† |
|---|---|---|---|
| T0 | `clippy` + `clippy-config` (Linux; macOS and Windows in parallel) | `.rs`, manifest, lock or `clippy.toml` changed | 300 s |
| T0 | `doc` | `.rs` or manifest changed | 150 s |
| T0 | `deny` (cached by `Cargo.lock` hash) | manifest or lock changed | 30 s |
| T0 | `graph`, freshness checks, `cs-patterns`, `toolchain` | always | 60 s |
| T0 | `codegen-fresh` | generator, IDL or generated files changed | 90 s |
| T0 | `pr-metadata`, `exceptions-listed`, `gate-config-changed` | always (PR) | 15 s |
| T0 | `test-verify` (medium tests, 20† seeds) | tests added or changed | 120 s |
| T0 | `test-fails-on-base` (affected crates only) | PR closes a `bug` issue or carries `Fixes:` | 240 s |
| T0 | `quarantine-expiry` | always | 5 s |
| T0 | fuzz targets: `cargo check` and regression-corpus replay | `parser`-tagged crate changed | 120 s |
| T0 | allocation-budget tests | measured crates changed | 60 s |
| T0 | `perf-gate` (base and head in one job) | benchmarked crate changed | 300 s |
| T0 | `hg-lint-canary` | `clippy.toml`, `tools/lints.toml`, a tier file or `Cargo.lock` changed; else nightly in T1 | 180 s |
| T1 | `miri`, loom (pinned nightly); one build per `hg-*-main`; clippy per binary with default features; fuzz-target builds; `cargo hack --each-feature`; `cargo mutants --in-diff` (advisory); diff coverage; `cargo machete` report (was CS-ST-15); `cargo vet`; CS-ER-13 privacy suite; `cargo build --timings`; size and `llvm-lines`; fork ASan/UBSan (CS-FFI-1); `hg-lint-canary` nightly | per merge or nightly | — |
| T2 | Same-seed rerun meta-test (CS-T5); fuzzing under an RSS limit; perf-gate noise job, weekly (CS-T6) | nightly or weekly | — |
| Release | `cargo-semver-checks` on published crates; PGO from the simulated corpus | release | — |

The abort (`release`, `checked`) and unwind (`dev`, test) builds of a crate do not share artifacts; DST's `checked` build is a separate cache and counts against T0's DST budget, which Stage 1 already holds.

### A.6 Custom checks that need a project tool

| Tool | Check | Rule |
|---|---|---|
| `hg-graph-lint` | Declared tier per crate ≥ computed tier; tier T crates unreachable from shipping binaries; `hg-substrate-sim` linked only by the binaries Q-CS-1 settles | 6.2, CS-ST-10 |
| | Profiles used by shipping and DST binaries set `panic = "abort"` | CS-ER-2 |
| | G3 for dev-dependencies; `rand`, `getrandom`, `num_cpus`, `tokio`, `smol` reachable from `hg-*` only via `hg-substrate-os` and the canary | CS-AS-1 |
| | Non-inventoried crates carry `#![forbid(unsafe_code)]`; inventoried module paths match `unsafe-inventory.toml`; unsafe block counts | G5, CS-TY-13, CS-T10 |
| | Crate- and module-level expectations only where A.2 permits; generated code's expectations only from the list | CS-ST-17 |
| | `anyhow` only in tier T; proc macros only from the allowlist and not in L1–L3; layer order | CS-ER-10, CS-ST-2 |
| | Flat layout, `publish = false`, `version = "0.0.0"`, `hg-` prefix, one `tests/` root | CS-ST-1, CS-ST-14, CS-TE-2 |
| | `nondeterminism-inventory.toml` entries for the simulated subgraph, scan-fresh | CS-AS-13 |
| | Crates tagged `parser` have a `fuzz/` target; inventoried crates are in the `miri` job | CS-TE-11, CS-TE-13 |
| `ast-grep` (`tools/ast-grep/`) | `cs-unwrap-on-try`, `cs-panic-format-args`, `cs-sensitive-debug`, `cs-non-exhaustive`, `cs-refcell-in-dom`, `cs-style-thread-pool`, `cs-std-os`, `cs-cfg-os`, `cs-atomic-ordering-comment`, `cs-float-par-reduce`, `cs-primitive-id-param`, `cs-unchecked-local-cast`, and the four below | CS-ER-7, CS-ER-14, CS-TY-4, CS-TY-10, CS-AS-3, CS-AS-1, CS-ST-18, CS-AS-9, CS-AS-12, CS-TY-1, CS-V8-5 |
| `hg-ipc-codegen` | Endpoint capacity and overflow policy per direction; bound or free; closed error enums; maximum lengths; `Result` handlers; dispatcher terminates bad senders; Kani harness per message (no panic, round trip, bounded allocation) | CS-AS-6, CS-ER-6, CS-ER-9, IPC-R2, IPC-R4 |
| `hg-webidl-codegen` | Tag ranges in 1–0x5fff assigned and asserted; range unwrap plus second check; error-to-exception table with static messages; `ScriptToken` in signatures that can run script; `GcPayload` and generated prefinalizers; no raw pointers in fast calls | CS-V8-1, CS-V8-9, CS-ER-13, CS-TY-10 |
| `hg-lint-canary` (tier T, base table, not a default member) | Depends on every crate named in `clippy.toml` (including `tokio` with `macros`, and `v8`); one use of each banned path under `#[expect(clippy::disallowed_…, reason = "canary")]`; an unfulfilled expectation means a path stopped matching | A.3 |
| `xtask` | `lints`, `exceptions`, `codegen --check`, `tier <crate>`, rule-index generation (headers, `agent-rules.md`, PR template, review context), quarantine expiry, test-verify selection, test-fails-on-base | CS-ST-10, CS-ER-3, CS-ST-17, CS-GA-2 to CS-GA-4, CS-TE-3, CS-TE-5, CS-TE-6 |
| `hg-substrate` | Debug long-task detector with sync-IPC attribution; thread-count assertion in simulation; the diagnostics facade and `log` adapter | CS-AS-7, CS-AS-3, CS-ER-14 |

The four new `ast-grep` rules, verified with ast-grep 0.51.0 (each fired on the violating lines of a sample file and on nothing else):

```yaml
id: cs-c-unwind              # CS-ER-17
language: rust
severity: error
message: "CS-ER-17: no hand-written extern \"C-unwind\""
rule:
  kind: extern_modifier
  regex: C-unwind
---
id: cs-pub-trait-future      # CS-AS-4: what async_fn_in_trait misses
language: rust
severity: error
message: "CS-AS-4: a public trait method returns a future"
rule:
  any:
    - kind: function_signature_item
    - kind: function_item
  has:
    field: return_type
    regex: "Future"
  inside:
    kind: trait_item
    has:
      kind: visibility_modifier
    stopBy: end
---
id: cs-async                 # CS-AS-4 / CS-D8; ignores generated from the allowlist in tools/tiers.toml
language: rust
severity: error
message: "CS-AS-4 / CS-D8: async code only in allowlisted crates"
rule:
  any:
    - kind: async_block
    - kind: await_expression
    - kind: function_modifiers
      regex: "^async"
ignores: ["crates/hg-substrate-os/**", "crates/hg-substrate-sim/**"]
---
id: cs-wrap-in-p             # CS-TY-12; applied to tier P paths only
language: rust
severity: error
message: "CS-TY-12: wrapping or saturating arithmetic in tier P needs // cs-allow(CS-TY-12): modular by design"
rule:
  any:
    - kind: field_identifier
      regex: "^(wrapping|saturating|overflowing)_"
      inside:
        kind: field_expression
        inside: { kind: call_expression }
    - kind: type_identifier
      regex: "^(Wrapping|Saturating)$"
```

### A.7 Verification record (18 October 2026, rustc and clippy 1.99.1, cargo-deny 0.20.2, ast-grep 0.51.0)

Draft 0.1's record stands where unchanged: every lint name and group in A.1 and A.2 resolved; every `clippy.toml` and `deny.toml` key parsed; every reachable disallowed path fired (52 entries in the fact-check). Draft 0.2 re-ran its changed blocks in a scratch workspace with probe crates for each tier:

- **A.1** parses; the edition-2024 defaults are as stated (`unsafe_op_in_unsafe_fn` and `unexpected_cfgs` warn by default; the two removed lints are migration-only). With `dev` aborting, `cargo test` after `cargo build` recompiled every dependency (5 units on a two-dependency probe); with `dev` unwinding, only the crate (1). A `[profile.release-net]` with `inherits = "release"` and `panic = "unwind"` builds (CS-D1 (b)).
- **A.2**: a tier P crate's `Result` function using `invariant!` raised nothing; `assert!` there raised `panic_in_result_fn` and `missing_panics_doc`; `CssPx + CssPx` passed with `arithmetic-side-effects-allowed`, an integer newtype's `+` and a plain `u32 + u32` were flagged, `wrapping_add` was not; `#[expect(unsafe_code, reason = …)]` on a module was fulfilled under `unsafe_code = "deny"`; `#[cfg_attr(target_os = "macos", expect(...))]` was fulfilled on macOS. The tier O probe with the generated `clippy.toml` allowed `std::fs::read`, `std::thread::spawn` and `sync_channel`, and flagged `std::sync::mpsc::channel`, `catch_unwind` and `tokio::sync::Mutex`, with diagnostics citing the crate's own `clippy.toml`. The tier T probe (filesystem, `unwrap`, `println!`, `Instant::now`) passed.
- **A.3**: all 38 entries added in 0.2 fired in a tier R probe (`std::fs`, `std::env`, `park_timeout`, `block_on`, `block_on_stream`, `LocalPool`, every channel constructor, `take_hook`, `log::set_logger`, `set_boxed_logger`, `DirBuilder`, `hashbrown`, `indexmap`, `ahash` types); none produced a "does not refer" warning. A misspelled `std::fs::reed_link` gave only a warning and exit 0 under `-D warnings`; the `clippy-config` pattern caught it.
- **A.4**: on the real L0 probe graph (457 packages), 0.1's `[bans.build]` failed with 68 `build-script-not-allowed` errors (65 crates); with the allowlist, 409 executable-script findings followed until the scan was scoped and the bypasses added; the final block passed all four checks. Stylo 0.22.0's `build.rs` runs Python (mako).
- **Rustdoc:** a broken intra-doc link passed `cargo clippy -D warnings` and failed the `doc` command.
- **A.6**: each `ast-grep` rule matched exactly its violations in a sample (two `extern "C-unwind"` items, not `extern "C"`; two future-returning `pub` trait methods, not a plain method or a private trait; an `async fn`, its `.await` and an `async` block; `Wrapping`, `wrapping_add` and `saturating_sub`, not `checked_add`).
- **V8 facts** read in v8 152.2.0's headers and the fork: tag constants and the range requirement (`v8-sandbox.h`), `CppHeap`'s default sweeping (`v8-cppgc.h`), prefinalizer properties (`cppgc/prefinalizer.h`), the exact-tag `v8__Object__Unwrap` and `RustObj::~RustObj` (`m0/rusty_v8/src/binding.cc`).
- **Failed or not verifiable here:** the `v8::` entries in A.3 and the paths in `await-holding-invalid-types` are silent until the canary links `v8` and the `hg_*` types exist; `allow-wildcard-paths` was not exercised with a wildcard path dependency.

### A.8 Traceability to the research notes

| Note rule | CS rule |
|---|---|
| ER-1–ER-18 (errors note 13.2) | CS-ER-1 (ER-1, ER-5, ER-18), CS-ER-2 (ER-2, ER-16), CS-ER-3, CS-ER-6 (ER-6, ER-12), CS-ER-7 (ER-7, ER-8), CS-ER-9, CS-ER-10 (ER-10, ER-15), A.1 (ER-4, ER-11), CS-ER-13, CS-ER-14, CS-ER-17 |
| TY-1–TY-15 | CS-TY-1 (TY-1, TY-2), CS-TY-3 (TY-3, TY-8), CS-TY-4 (TY-4, TY-6), CS-TY-5 (TY-5, TY-7), CS-TY-9, CS-TY-10 (TY-10, TY-11; TY-10's `try_borrow_mut` restored), CS-PF-1 (TY-12), CS-TY-12 (TY-13), CS-TY-13 (TY-14), CS-ST-13 (TY-15) |
| V8-1–V8-6 | CS-V8-1 (V8-1, V8-2), CS-V8-3, CS-TY-10 (V8-4, V8-6), CS-V8-5 |
| A1–A16 | CS-AS-1, -2, -3; CS-AS-4 (A4, A5, A10, A14); CS-AS-6, -7, -8; CS-AS-9 (A9, A11); CS-TY-9 (A12); CS-AS-12; CS-AS-13; A.1 (A15, A16) |
| S1–S15 | CS-ST-1 (S1, S12), CS-ST-2 (S2, S3), A.1 (S4, S11), CS-ST-14 (S5, S14), CS-ST-6, CS-ST-7, 6.12 (S8), CS-ST-9, CS-ST-10, CS-ST-13, A.5 (S15) |
| T-1–T-18 | CS-AS-1 (T-1), CS-TE-2 (T-2, T-9), CS-TE-3, -4, CS-TE-5 (T-5, T-7), CS-TE-6, -8, CS-TE-10 (T-10, T-18), CS-TE-11, -12, CS-TE-13 (T-13, T-14; Kani per pure unsafe block restored as SHOULD), CS-TE-15 (T-15, T-16), CS-TE-17 |
| PF-1–PF-12 | CS-PF-1, A.1 (PF-2), CS-PF-3, -4, -5, CS-PF-6 (PF-6, PF-7, PF-9, PF-11, PF-12), CS-PF-8, 6.12 (PF-10) |

Choices that differ from the notes, recorded as the fact-check asked: tier R `unwrap_used`, `unused_result_ok`, `fallible_impl_from` and `mod_module_files` are `warn`, not `deny` (CI behaves the same under `-D warnings`; a local `cargo check` does not run clippy anyway); `missing_docs` is limited per CS-ST-13; PF-2's "deny the perf group" became "the perf group stays on"; A12's `future_not_send` was dropped, because CS-D8 leaves no `Send` futures outside the substrate; S2's "no serde derive or `syn` in L1–L3" was widened to "no proc-macro dependencies in L1–L3".

## Appendix B. Review disposition

**Fact-check corrections** (`factcheck.md`, "Required corrections" 1–12 and the tables): all applied.

| Item | Disposition |
|---|---|
| 1–2 Tier O/T crate-level expectation breaks CI and exempts non-G3 bans | Applied: tier T `allow` in generated tables; tier O per-crate generated `clippy.toml` keeps non-G3 bans (verified) |
| 3 CS-V8-1 range check, pre-order labeling, 15-bit tags | Applied: new fork binding; Izcal's own scheme; embedder range 1–0x5fff |
| 4 CS-ER-17 lint | Applied: `cs-c-unwind` hook; `ffi_unwind_calls` described as unwind-build only |
| 5 CS-AS-4 lint | Applied: `cs-pub-trait-future` hook |
| 6 Rustdoc lints never run | Applied: `doc` job in T0, counted in CS-T1 and A.5 budgets |
| 7 Misspelled `clippy.toml` paths exit 0; modules warn | Applied: `clippy-config` check; Section 3 reworded |
| 8 Build-script allowlist understated; `[graph] targets` | Applied: measured 65-crate list, scoped scan, hash-pinned bypasses, targets |
| 9 SEC-14 (d) citation | Applied: cited only for "a recorded human decision" |
| 10 Tier S vs Stage 1 | Applied differently: Q-CS-1 for the Stage 1 owner with a recommendation; DevTools G12 also found to ship `hg-substrate-sim`; tier S dissolved (sim tier R, `hg-dst` tier T) |
| 11 Wrong external facts | Applied: Firefox triage (now 1 business day†), Android PGO figures (claim withdrawn), Quantum DOM, SE-0466, arrayref (`proc-macro1`), Cloudflare 5 December (Lua, FL1), security scope 3.3 (quinn-proto, rustls), CS-TE-5 wording, afidt-box, tidy suppression, TigerBeetle average and Gallant example |
| 12 Lint-level statements | Applied: `cargo check` runs rustc lints only; edition-2024 defaults; migration lints removed |
| Partly right items in tables 2.1–2.3 | Applied: Cloudflare quote; Mojo scoped to untrusted senders; Deno resizable buffers; FIDL flexible types; Firefox crash annotations confirmed and unflagged; Chromium abort details; Firefox repository link; s2n-quic `.clippy.toml`; Chromium `<random>`; test-verify year; WebKit "back out or fix"; Servo quote scope; Stylo thread count; `gungraun` Linux and FreeBSD only (unflagged); coverage "low to moderate"; Midori OOM; RFD 609 date via the issue; tonic effect size; bolero link; Miri native-lib caveat; rustc UI wording; Cargo's own test suite; `static_assert_size` advice; proptest constraints; SQLite `make test`; loom README; GhostCell unflagged. The mimalloc claim left with CS-PF-11's retirement |
| Internal references (3.1) | Applied: G9 by `hg-anchor`; `hg-tree-traits` and `hg-semantic-repair` tiered; "most of the code" softened; IPC-R5 as the bad-message source; Mojo cited directly for CS-TY-1; IPC-R3/R4 for CS-TY-4; separate session as Izcal's addition; `Spec:` not enforced today; HGBA-2 P1 wording; principles named by document; B2 and RES-3 sources; DET-1 "for simulation mode"; CS-D6 trigger no longer leans on EQ-1; VER-1 scope; license list as an interpretation; S-4c paraphrase; `m0/rusty_v8` in `exclude` |
| Coverage gaps (1.4) | Applied: `std::fs`, `std::env`, `park_timeout`, `DirBuilder` in A.3; `std::os` by hook; `hashbrown`, `indexmap`, `ahash`; `futures::executor` |
| Silent changes (3.3); untraced lints; † markings; 11.2 list; header; CS-T3 | Applied: A.8 records the changes; untraced lints removed or traced; † removed from numbers set elsewhere and added to this document's own; 11.2 says "every † value"; header lists every cited ID; CS-T3 is a Phase 2 CI target |
| Canary vs `wrappers` and graph allowlist | Applied: canary in `wrappers` and the graph allowlist; path-filtered in T0 because it builds `v8` |

**Design critique** (`critique.md`):

| Finding | Disposition | Where / why |
|---|---|---|
| H1 Async model undecided | Applied | CS-D8, CS-AS-4 (closures by default, allowlisted `async`, no bridging of JS Promises); AS-8, AS-10, AS-14 scoped |
| H2 Bounded-queue rule unworkable | Applied | CS-AS-6 (a)–(c): memory accounting under HTML's resource-limit clause; non-blocking tier P senders declared in IDL; substrate-only channels; `sync_channel` suggestion removed; 1024† only for (c) |
| H3 Token-cell soundness | Applied | CS-D3 exit criteria (1)–(5), including the two-token question |
| H4 GC finalization | Applied | CS-V8-9; Q-CS-4 for the sweeping thread |
| H5 T0 budget not credible | Applied | Per-job budgets and path filters (A.5); per-binary and fuzz builds to T1; `test-fails-on-base` on fix PRs, affected crates; deny cached; CS-T9 as p90 wall time |
| H6 Phase 1 on M1's critical path | Applied | Phase 1a (≤ 3 days†) and 1b (≤ 4 weeks†), CS-D10 |
| H7 Tier P arithmetic and fake errors | Applied | `invariant!` (A.2, verified); `cs-wrap-in-p`; `arithmetic-side-effects-allowed`; `panic_in_result_fn` resolved; invariants never become `BadMessage` (CS-ER-3) |
| H8 Agents can edit the gate | Applied | CS-GA-1 to CS-GA-4: `CODEOWNERS`, agent rules, suppressions auto-listed, `rules.toml` proposed, review context and chunked diff |
| M1 Abort vs tests | Applied | CS-ER-2; `dev` unwinds (measured) |
| M2 Simulator panic costs; `hg-dst` tier | Applied | `hg-dst` tier T, one seed per child process, crash-handler hook, signature counting (CS-ER-2) |
| M3 Computed tiers flip | Applied | Declared tiers (CS-ST-10, CS-D9); lists marked expected; `hg-shell` noted; `hg-types` cost in CS-D2 |
| M4 deny.toml on the real graph | Applied | Measured on `v8` 152.2.0 and the L0 set; scoped executables check; bypasses (A.4) |
| M5 Generated code repository rules | Applied | CS-ST-17 (b), (f); CS-T14; CS-D4 trigger objective |
| M6 Perf gate determinism | Applied | CS-PF-4 (seeded hashing, single thread, same job, no `v8`) |
| M7 Trailers and squash merges | Applied | CS-GA-3, CS-TE-3 (PR body, squash setting, issue-based trigger) |
| M8 Logging and telemetry privacy | Applied differently | Folded into CS-ER-14 (no new ID); the proposed CS-D9 "own facade vs `tracing`" is not an owner decision: Izcal builds its own facade |
| M9 C++ and FFI declarations | Applied | CS-FFI-1 |
| M10 Platform code and `cfg` | Applied | CS-ST-18; `cfg_attr` expect pattern (verified); clippy on three OSes; per-binary clippy in T1 |
| M11 Hasher keying | Applied | CS-AS-2: one key per process in a `OnceLock` |
| M12 Inventory for every package | Applied | CS-AS-13 scoped to the simulated subgraph, pre-filled by scan |
| M13 Too many rules; wrong summary | Applied | 100 → 65 rules; summary rebuilt around the critique's list |
| M14 Nightly pinning and cadence | Applied | CS-ST-9 |
| M15 Inventoried crates cannot use `unsafe` | Applied | Module-level `#[expect(unsafe_code)]` (verified); `CODEOWNERS` by inventory owner; SAFETY shape; unsafe count in CS-T10; Kani per pure unsafe block as SHOULD (CS-TE-13) |
| M16 Generator-emitted expectations | Applied | CS-ST-17 (c)–(e) |
| M17 Targets that cannot fail | Applied | CS-T2, T3, T7, T12 changed; CS-T13 and CS-T14 added |
| M18 Decisions | Applied | CS-D2 trigger and DET-2 order; CS-D4 objective; CS-D6 mechanical; CS-D7 cites the utility threat model; CS-D8 async, CS-D9 tier application (critique's D10 and D11 merged), CS-D10 phase split; critique's D9 rejected (build our own) |
| L1 Sync IPC vs no blocking | Applied | CS-AS-7 |
| L2 External enums | Applied | CS-TY-4 adapter pattern |
| L3 Type bans vs dependency APIs | Applied | CS-AS-1 adapter modules |
| L4 `cs-allow` never expires | Applied | 6.1 and `cs-patterns` |
| L5 Timeouts as flakes | Applied | CS-TE-2 (simulated time), CS-TE-5, CS-T4 |
| L6 `test-verify` on small tests | Applied | CS-TE-6 |
| L7 Naming conventions | Applied | CS-ST-14 |
| L8 PR template | Applied | CS-GA-3 |
| L9 Secrets | Applied | CS-ER-14 `Secret<T>`, SEC-34 |
| L10 Documentation lints | Applied | CS-ST-13 scoped |
| L11 Dropped lessons | Applied in part | TY-10 `try_borrow_mut` and T-13 Kani restored; V8-1's "non-wrappable tag range" rejected: v8 152.2.0's `v8-sandbox.h` defines no such range (only the embedder range and V8-internal tags from 0x6000); A12 `future_not_send` rejected as moot under CS-D8 |

The critique's strengths are kept: a mechanism per rule, the tier model tied to the process map (now declared), generator-owned seams, abort with the simulator worked through, the conflicts table, measured tool gaps, the determinism discipline, the performance approach, checked-in generated code, spec-first tests for AI code, and amendments proposed rather than applied.

## Appendix C. ID changes from 0.1

Surviving IDs keep their numbers and subjects. A retired ID is never reused; in an existing `reason = "…"` it resolves to the rule below.

| 0.1 ID | 0.2 |
|---|---|
| CS-ER-4 | 6.1 Exceptions and A.1 (`allow_attributes`) |
| CS-ER-5, CS-ER-16 | CS-ER-1 |
| CS-ER-8 | CS-ER-7 |
| CS-ER-11 | A.1 (`unused_must_use`, `let_underscore_must_use`, `unused_result_ok`) |
| CS-ER-12 | CS-ER-6 |
| CS-ER-15 | CS-ER-10 |
| CS-ER-18 | CS-ER-2 |
| CS-TY-2 | CS-TY-1 |
| CS-TY-6 | CS-TY-4 |
| CS-TY-7 | CS-TY-5 |
| CS-TY-8 | CS-TY-3 |
| CS-TY-11 | CS-TY-10 |
| CS-V8-2 | CS-V8-1 |
| CS-V8-4, CS-V8-6 | CS-TY-10 |
| CS-V8-7 | Section 6.12 (Stage 1 5.3) |
| CS-V8-8 | Section 6.12 (S-4c, FFI-R3) |
| CS-AS-5, CS-AS-10 | CS-AS-4 |
| CS-AS-11 | CS-AS-9 |
| CS-AS-14 | A.1 (`large_futures`) |
| CS-AS-15 | A.1 (`let_underscore_future`) and CS-ER-14 (printing) |
| CS-ST-3 | CS-ST-2 |
| CS-ST-4 | A.1 (`unreachable_pub`, `unnameable_types`) |
| CS-ST-5 | CS-ST-14 |
| CS-ST-8 | Section 6.12 (SEC-14) |
| CS-ST-11 | A.1 (`too_many_lines`) and A.5 `file-length` |
| CS-ST-12 | CS-ST-1 |
| CS-ST-15 | A.5 (`cargo machete` report) |
| CS-ST-16 | Section 6.12 (M-1, D-S10, G9) |
| CS-TE-1 | CS-AS-1 |
| CS-TE-7 | CS-TE-5 |
| CS-TE-9 | CS-TE-2 |
| CS-TE-14 | CS-TE-13 |
| CS-TE-16 | CS-TE-15 |
| CS-TE-18 | CS-ER-1 and A.1 `checked` profile |
| CS-PF-2 | A.1 (perf lints) |
| CS-PF-7, CS-PF-9, CS-PF-11, CS-PF-12 | CS-PF-6 |
| CS-PF-10 | Section 6.12 (Stage 1 7.1) |
| — | New: CS-V8-9, CS-FFI-1, CS-ST-17, CS-ST-18, CS-GA-1 to CS-GA-4; CS-T13, CS-T14; CS-D8 to CS-D10; Q-CS-1 to Q-CS-6 |
| Tier S | Dissolved: `hg-substrate-sim` tier R (pending Q-CS-1), `hg-dst` tier T |

Rule counts, 0.1 → 0.2: errors 18 → 10; types 13 → 8; V8 and FFI 8 → 5; async 15 → 10; structure 16 → 10; testing 18 → 12; performance 12 → 6; agents and the gate 0 → 4. Total 100 → 65.
