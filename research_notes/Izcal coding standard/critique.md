# Izcal coding standard: design critique (method phase 8)

Reviewed 18 October 2026, against `reports/Izcal coding standard.md` draft 0.1. Reviewer: independent design critic. I did not write the spec or the research notes. Sources and lint names belong to the separate fact-check; I checked a few lint semantics only where the design depends on them (`cargo clippy --explain`, clippy 1.99.1), and looked at crate sources in the local Cargo registry where noted.

Context read: the spec in full; `.claude/skills/first-principles-spec/SKILL.md` and `references/spec-template.md`; Stage 1 design Sections 1–8 (G1–G9, substrate, IPC, renderer pipeline, CI tiers); security scope draft 0.1 (IPC-R3–R5, SEC-3/4/7/11/12/14/27/28); `CONTRIBUTING.md`; `tools/checks.sh` (AI review); all three research notes.

**Counts:** 8 high, 18 medium, 11 low.

**M-1 / D-S10 / build-our-own:** no violations found. Every borrowed idea (CanGc, GhostCell, `gin`'s second type check, Mojo's rules, S2's same-seed rerun) is cited as evidence and rebuilt for Izcal. The tools it adds (proptest, insta, gungraun, loom, cargo-mutants and the rest) are unmodified dependencies and not part of any browser. The spec never treats build-vs-reuse as an owner question.

---

## High

### H1. The async model is not decided, but six rules assume one

- **Location:** CS-AS-4, CS-AS-8, CS-AS-10, CS-AS-14, CS-AS-15; Section 6.7; top-16 item 11.
- **Problem:** Stage 1 Section 3.1 makes the substrate synchronous traits plus `Executor::post(seq, Task, delay)`. The async note's lesson table says "tasks are run-to-completion closures (A4)", but the spec never says so. It also never says whether engine crates may write `async fn` and `.await`, or which poller would run them. Meanwhile CS-AS-8 regulates guards across `.await`, CS-AS-10 requires a `# Cancel safety` section on every `pub async fn`, CS-AS-14 caps future sizes, and CS-AS-4 allows crate-private async traits. An engineer or agent cannot tell whether writing `async fn fetch_thing()` in `hg-net` is normal, discouraged or banned. Nor can they tell what owns its waker. The rules also miss the real Izcal hazard: a JS Promise is a V8 microtask, not a Rust future, so "async" in bindings means something else again.
- **Proposed change:** add decision **CS-D8, the async model**, with this recommendation. (a) `Task` is a `FnOnce` closure that runs to completion on its sequence; completions arrive as posted tasks or `SequenceBound` sinks. (b) `async fn` and `.await` are banned in L3–L6 engine crates through the `hg-graph-lint` crate list plus a hook. They are allowed only in an allowlisted set (candidates: `hg-substrate-*`, and an `hg-net` protocol driver if the HTTP/2 question in 11.1 needs one). There, futures are `!Send`, sequence-local and polled only by a substrate-provided poller. (c) CS-AS-8's `.await` clauses, CS-AS-10's doc rule and CS-AS-14 apply only to allowlisted crates. CS-AS-8's "posted-task boundary" clause becomes the general rule. (d) State that JS Promises and microtasks are V8's and are never bridged to Rust futures. Reversal trigger: a protocol dependency the network service needs cannot run sans-IO.

### H2. "Every queue is bounded" has no workable overflow policy for web-visible task queues or for broker sends

- **Location:** CS-AS-6 (default 1024 tasks per sequence), CS-AS-7, A.3 (`std::sync::mpsc::channel` → `replacement = "std::sync::mpsc::sync_channel"`).
- **Problem:** Three issues.
  1. The HTML task queues, the timer list and `postMessage` are unbounded by specification. A page that schedules 100,000 `setTimeout`s or floods `postMessage` is valid web content. None of the three policies works on the script sequence: backpressure cannot block JS; "reject with an error" has no spec-defined error; "drop-oldest" breaks the spec silently.
  2. A broker or service that sends to a renderer must never block on a full endpoint. Otherwise one slow or hostile renderer stalls a shared process, which is the cross-site DoS that tier P exists to prevent. The rule does not say which policy shared processes use.
  3. The suggested replacement, `sync_channel`, has a *blocking* `send`, which breaks CS-AS-7 ("no task blocks"). Raw std channels are also invisible to the simulator's scheduler.
- **Proposed change:** split CS-AS-6:
  - **(a) Web-visible queues** (HTML task sources, timers, message ports) are bounded by **memory accounting**, not task count. Exceeding the renderer's budget is a renderer OOM: a tier R crash of the attacker's own site, the outcome content can already cause.
  - **(b) Cross-process endpoints:** a sender in a tier P process never blocks. On overflow toward a less-privileged receiver it drops with a counter, or treats the receiver as unresponsive and asks the broker to kill it. The IDL declares which per endpoint direction, and `hg-ipc-codegen` rejects an endpoint without it.
  - **(c) Intra-process queues** use only substrate primitives (`post`, `SequenceBound`). Ban std, crossbeam, flume and futures channels outside `hg-substrate-*` entirely, with the substrate as the replacement, and remove the `sync_channel` suggestion.
  - **(d)** Drop the 1024† number, or make it the default only for (c).

### H3. CS-D3's token cell lacks a soundness argument for three cases that decide whether it works

- **Location:** CS-TY-10, CS-D3, A.3 `await-holding-invalid-types`, Section 11.2 (GhostCell flagged).
- **Problem:** The idea is good (H-level praise below), but the spec presents it as decided pending a performance prototype. The prototype's acceptance criteria cover only expressiveness and a 2%† cost. The hard questions are soundness:
  1. **Minting at re-entry.** JS → Rust callback → JS → Rust callback: the inner callback enters through `extern "C"` with no Rust path to the outer token. The generated shim must conjure a fresh `&mut ScriptToken`. That is sound only if *every* way to run script or GC takes `&mut ScriptToken`, so the outer token is provably lent out at that moment. Nothing in the spec states or enforces this invariant. One hand-written `v8::Function::call` without the token breaks it.
  2. **GC tracing.** Production `cppgc` marks concurrently (S-4c). A traced field inside a `TokenCell` is read by the marker without the token, which is a data race. The spec never says that traced state (`Member` links) must live outside token cells, with generated barriered setters.
  3. **One mutable borrow at a time.** GhostCell-style cells allow one `borrow_mut` across *all* cells at once. DOM insert, remove and replace mutate parent, child and sibling together. The spec's fallback trigger names this case, but the design choices (scoped multi-borrow APIs, clone-out, or per-node split) are left unstated.
  4. **A second token.** rusty_v8 already makes "may run script" an exclusive borrow of `&mut HandleScope`/`PinScope`. A separate `ScriptToken` duplicates that, and the two can drift.
- **Proposed change:** rewrite CS-D3's option (a) as a design with an explicit soundness argument, and add these as M2 prototype exit criteria:
  - the token is derivable only from `&mut HandleScope`, or from a generated callback prologue that consumes the scope V8 passes in;
  - a graph or hook check that every rusty_v8 entry point that can run script is reachable only through token-taking wrappers (`disallowed-methods` on the raw `Function::call`, `Script::run` and similar outside `hg-script`'s wrapper module);
  - traced fields are never inside `TokenCell`, which the generator enforces;
  - a written multi-node mutation pattern, exercised on `insertBefore` with a `MutationObserver` and a custom-element reaction;
  - Miri or loom-style tests of the cell API itself.

  Keep the 2%† trigger as a second criterion.

### H4. No rule covers code that runs during GC finalization

- **Location:** gap next to CS-V8-8 and CS-TY-10. Stage 1 5.2 puts "Rust-owned payload … dropped when the node is collected" and a prefinalizer in the node design.
- **Problem:** Rust payload inside a `cppgc` object is dropped during sweep. If a payload's `Drop` (or anything it owns) dereferences another GC object, a `Member`, a node ID lookup that touches the heap, or a `TokenCell`, it can touch an object that is already dead or being swept: use-after-free in safe-looking code. Allocating in the `cppgc` heap from a prefinalizer, or running script from `Drop`, is also forbidden by the heap. This is the GC-era version of the Deno lifetime bugs the spec cites, and neither CS-V8-* nor S-4c covers it.
- **Proposed change:** add **CS-V8-9, finalization discipline**:
  - Payload types stored in GC objects implement a sealed `GcPayload` marker. The generator or a derive rejects payloads that contain `Member`, GC handles, `TokenCell`, `SequenceBound` or anything with a non-trivial `Drop` that is not on an allowlist.
  - Prefinalizers are generated and may only release data-plane and semantic-plane slots by ID, which are plain integer operations.
  - `Drop` never allocates in the GC heap, never posts script work and never takes a lock above the lowest level.
  - Enforcement: gen, plus a debug-V8 stress test that sweeps with poisoned memory (the existing S-4c suite extended).

### H5. CS-T9's 6-minute share of T0 is not credible, and T0 is already full

- **Location:** CS-T9; A.5 T0 row; Stage 1 7.2 T0 (≤20 min already holds fmt, clippy, graph lint, anchor lint, unit and property tests, Kani for changed messages, bindings tests on debug V8, WPT smoke of about 2k subtests, 200 DST seeds, and the fork's `test_cppgc`).
- **Problem:** Several jobs added to T0 are each a separate full build:
  - per-binary builds, one `cargo build` per `hg-*-main`, about 8, with different feature unification so their caches are not shared;
  - fuzz-target builds, which need nightly plus sanitizer and coverage instrumentation, so every dependency is rebuilt;
  - `test-fails-on-base`, which builds the merge base;
  - the `perf-gate` under Valgrind, roughly 20–100× slowdown, measured for base and head;
  - `clippy --all-targets --all-features`, a full check build;
  - `cargo deny` with `[bans.build] include-dependencies = true`, which scans the 202 MB `v8` crate source (measured locally: `v8-152.2.0`);
  - `test-verify`, allocation tests and the canary.

  CS-T9's own list leaves out per-binary builds, fuzz builds, `test-fails-on-base` and the canary. Adding them on top of Stage 1's T0 content will exceed 20 minutes on GitHub-hosted runners. The failure action ("move the slowest CS job to T1") then fires in week one, leaving the gate undefined.
- **Proposed change:**
  - Budget each job (seconds†) and measure them in the first Phase 1 month.
  - Make expensive jobs **path-conditional**: fuzz builds only when a `parser`-tagged crate changes; perf gate only when a benchmarked crate changes; per-binary builds only when a manifest or feature changes.
  - Move per-binary feature builds and fuzz-target builds to T1, keeping a T0 compile of fuzz targets as `cargo check` only.
  - Run `test-fails-on-base` only on `fix/*` PRs, building only the affected crates' test binaries.
  - Cache the cargo-deny build scan by `Cargo.lock` hash.
  - Restate CS-T9 as "T0 p90 wall time ≤ 20 min over a rolling 2 weeks, with every CS job's share published".

### H6. Phase 1 puts 3–5 engineer-weeks of tooling ahead of the first M1 substrate commit

- **Location:** Section 9.1 Phase 1 ("the commit that creates the workspace": all lint tables and the generator, `hg-graph-lint` with tiers, full `deny.toml`, every CS-ER, CS-TY, CS-AS and CS-ST rule); 9.2 staffing.
- **Problem:** M1 sits on the critical path (M0 → M1 → M2 …, Stage 1 8.2). The Infrastructure track (2–3 engineers) also owns CI tiers, the DST farm, oracles, the WebDriver server needed at M2, and the perf lab. `hg-graph-lint` does not exist yet, and neither does `cargo xtask lints`. Making the first workspace commit wait for all of it delays the substrate and IPC tracks, or it pushes people to work outside the workspace, which is worse.
- **Proposed change:** split Phase 1.
  - **Phase 1a, day one, ≤3 days†:** virtual manifest, profiles (`panic = "abort"`), `[workspace.lints]` base table, `clippy.toml` with the G3 lists, clippy in CI, nextest `retries = 0`, `deny.toml` advisories plus sources.
  - **Phase 1b, ≤4 weeks†, before any tier P crate has non-skeleton code:** tier file, generated tier tables, graph-lint tiers, `[bans.build]`.
  - State the cost of the gap: code written between 1a and 1b is re-linted when 1b lands, which is cheap while the code is small.

  This keeps B1's "from day one" spirit without gating M1.

### H7. Tier P's lint set rewards silent wraparound and pushes invariants into fake errors

- **Location:** CS-ER-3, CS-ER-16, CS-TY-1, CS-TY-12, CS-T2, A.2 tier P additions.
- **Problem:** Each of the following follows from the lint set as written.
  1. **`arithmetic_side_effects` is satisfied by `wrapping_*` and `saturating_*`.** Clippy's own text: "it is desirable to explicitly call checked, wrapping or saturating arithmetic methods", and `Wrapping` and `Saturating` types are ignored. Under deny plus the CS-T2 budget, the cheapest fix in the broker is `wrapping_add`: a silent wrong length or offset in a privileged process. That is worse than the abort it replaces.
  2. **Custom unit types are flagged.** The lint "checks any kind of arithmetic operation of any type" and ignores only floats and listed types. CS-TY-1's unit newtypes (`CssPx`, `DevicePx`, `ByteLen`) in tier P crates (`hg-types` is tier P) are flagged on every `+` unless they are listed in `arithmetic-side-effects-allowed`, which A.3 does not set.
  3. **`panic_in_result_fn` flags `assert!`** ("panic! or assertions in a function whose return type is Result"). So CS-ER-16's "cheap asserts stay on in release" needs an `#[expect]` in every `Result`-returning tier P function, and the two rules contradict each other.
  4. With expectations rationed (CS-T2), the path of least resistance for a real internal invariant is `return Err(Error::Internal)`. That error then propagates up a generated handler as `BadMessage`. The broker's own bug gets blamed on the renderer, which is killed, and the bug disappears from crash telemetry.
- **Proposed change:**
  - Add one sanctioned invariant path, `hg_types::invariant!(cond, "literal")` (and `invariant_unreachable!("literal")`). It aborts with a literal message and carries the only `#[expect]` for the panic lints, inside the macro definition. CS-T2 counts `invariant!` uses per kLOC instead of scattered expectations. Reviewers and the AI review can grep every one.
  - Add an ast-grep rule `cs-wrap-in-p`: `wrapping_*`, `saturating_*`, `Wrapping` and `Saturating` in tier P need `// cs-allow(CS-TY-12): modular by design` (hashes, sequence numbers, checksums).
  - Set `arithmetic-side-effects-allowed` to Izcal's float-based unit types in `clippy.toml`.
  - Add a rule that internal-invariant failures in tier P never map to `BadMessage`. `hg-ipc-codegen`'s `Result` type separates `BadMessage` (sender's fault) from bugs, which go to `invariant!`.
  - Add to CS-ER-3: "prefer `checked_*` plus `?`; never wrap to satisfy the lint".

### H8. Nothing stops an agent from making the gate pass by editing the gate

- **Location:** gap. Sections 6.1 and 6.13 ("the standard grows beyond what agents can follow"), CS-ER-4, A.5. Also `tools/checks.sh` review prompt and `IZCAL_REVIEW_MAX_LINES=4000`.
- **Problem:** The standard's levers are files a PR can change. An agent told "make CI green" can change each of them, and each is a one-line diff a tired reviewer approves:
  - `clippy.toml`, `tools/lints.toml`, `tools/tiers.toml` (move a crate from P to R), `deny.toml` (add a build-script allowlist entry);
  - `unsafe-inventory.toml`, `nondeterminism-inventory.toml`, `tests/quarantine.toml` (quarantine the failing test), `.config/nextest.toml`, the ast-grep rules;
  - `#[expect(..., reason = "invariant: …")]` with a plausible but false reason, and `// cs-allow` markers.

  SEC-27 already requires checks to run from the base branch, but these configuration files come from the PR. The AI review prompt knows nothing of this standard. Its 4000-line cut would also fill with checked-in generated code (H-adjacent; see M5).
- **Proposed change:** add Section 6.14, **"Rules for agents and for changing the gate"**.
  - (a) The gate-configuration files above are in `CODEOWNERS` under the security owner. A CI job `gate-config-changed` flags any PR touching them and requires that owner's approval: once two maintainers exist, a human other than the author; until then, a recorded decision line in the PR.
  - (b) **Agents never edit gate configuration, add quarantine entries or add `unsafe-inventory` entries** unless the task explicitly says so. The agent prompt text is checked in at `tools/agent-rules.md`.
  - (c) Every *new* `#[expect]` and `cs-allow` in a diff is listed in the PR summary by a CI step. The AI review is told to verify each reason against the code and to treat a false reason as high severity.
  - (d) A machine-readable rule index, `tools/cs-rules.toml` (ID, one-line rule, tiers, path globs, mechanism). From it `xtask` writes a short per-crate `AGENTS.md`-style header ("this crate is tier P; rules …"). The hook's review prompt gets the rules that apply to the touched crates, with emphasis on review-only rules.
  - (e) The review excludes `src/generated/` from the diff it sends and reviews the generator change instead.

---

## Medium

### M1. "Abort in every profile" contradicts itself for tests, and costs a second dependency build

- **Location:** CS-ER-2, Section 6.3 row 1 ("tests must never depend on unwinding either"), CS-TE-2 medium tests, CS-TE-10 proptest.
- **Problem:**
  - Cargo builds tests with unwind regardless of the profile, so every `cargo test` and nextest run unwinds.
  - CS-ER-2 says `#[should_panic]` is fine, and that depends on unwinding. Proptest's failure detection and shrinking also use unwinding.
  - Medium simulator tests in `tests/it/sim/` therefore have a different panic model from the `hg-dst` farm, which aborts.
  - With `panic = "abort"` in `dev`, `cargo build` and `cargo test` compile every dependency twice: once with abort, once with unwind. Stylo, wgpu, WebRender and the V8 bindings are big, so this matters for local iteration and CI caches.
- **Proposed change:** reword the rule. **Production and simulation binaries abort. Test builds unwind (Cargo forces it), and test frameworks may rely on that. Production code must not.** Ban `catch_unwind` in non-test code, which is what the lint does. Measure the double-build cost in Phase 1a. If it is large, set `dev` to unwind and keep abort in `release`, `checked` and every shipped or DST binary, since nextest's process-per-test already contains a panic. Then the graph check becomes "every *shipping and DST* profile aborts".

### M2. The simulator panic model is right, but its costs and `hg-dst`'s tier are wrong

- **Location:** CS-ER-18, Section 6.2 tier S, A.3 `set_hook` ban.
- **Problem:** Abort is the correct choice: V8's process-wide platform and entropy source mean a panic cannot be contained per simulated process. Two consequences are unstated.
  1. Tier R allows invariant panics, so corpus-driven DST seeds that hit a web-reachable renderer panic end the whole seed and its batch. DST-1's yield accounting needs to separate "seed ended by a known renderer panic" from new bugs.
  2. `hg-dst` is tier S, with G3 and the disallowed lists applying, but it spawns child OS processes, writes bug reports and needs `set_hook` to print the seed before abort.
- **Proposed change:**
  - Split tier S into `hg-substrate-sim` (tier S, G3 applies) and the `hg-dst` runner (tier T, OS allowed, never linked into the simulated program).
  - Run one seed per child process, or resume the batch after the failing seed, so one panic does not lose the batch.
  - Let the sim binary's crash handler, the one `set_hook` site, print seed and schedule.
  - Have DST-1 count panic-terminated seeds by signature, deduplicated.

### M3. Computed tiers flip silently, and the tier table already contradicts the rule

- **Location:** Section 6.2 ("strictest tier of any `hg-*-main` binary that links it"); tier R list; CS-D2.
- **Problem:**
  1. A PR that adds one dependency edge (for example `hg-browser-main` → `hg-semantic` for a shell feature) silently moves the target crate to tier P. That produces hundreds of lint errors in code the author did not touch.
  2. The tier R list includes `hg-shell` and possibly `hg-devtools` and `hg-media`, but the broker owns the "Shell UI" (Stage 1 Section 1), so `hg-shell` is likely linked into `hg-browser-main`, which makes it tier P under the rule.
  3. `hg-types` (geometry, IDs, atoms) is tier P because the broker links it, so its arithmetic carries `overflow-checks = true` in release. Overflow checks are inserted in the defining crate's MIR, so they are inlined into renderer hot paths too. This confounds CS-D2's measurement, and the tier P lint set lands on geometry code (H7.2).
- **Proposed change:**
  - **Declare** each crate's tier in `tools/tiers.toml`, and have `hg-graph-lint` *fail* when the declared tier is laxer than the computed one, with the message "crate X is now linked by Y; declare tier P in its own PR".
  - Remove the per-tier crate lists from 6.2, or label them "expected; tiers.toml is normative".
  - Split hot geometry and units out of `hg-types` into a renderer-side crate. Or keep it and state that `hg-types` is overflow-checked everywhere, and include that cost in CS-D2.

### M4. The proposed `deny.toml` would fail on Izcal's real dependency graph

- **Location:** A.4 `[bans.build]`; CS-ST-7 ("`build.rs` only for … the `v8` fork and platform FFI crates"); A.7 ("passed on a scratch workspace").
- **Problem:** The scratch workspace did not contain the real dependencies.
  - Measured locally on `v8-152.2.0`: the crate ships `build.rs`, **319 `.py` files and 231 executable files** (202 MB). With `include-dependencies = true`, `executables = "deny"` and `interpreted = "deny"`, it fails unless it has `bypass` entries with globs or checksums.
  - Also in the local registry with `build.rs`: `selectors-0.38.0`, `libc`, `ring`, `zerocopy`, `wgpu-0.20`.
  - Stylo's build is likely to run Python (mako) for property generation. That needs checking against 0.22.

  "No build scripts by default" is the right policy, but the allowlist will hold dozens of entries, not "the v8 fork and platform FFI crates".
- **Proposed change:** generate the Phase 1b allowlist from the real `cargo metadata` and record each entry with a reason and the reviewer. Add `bypass` entries (with `allow-globs` and a checksum) for `v8`, and for `stylo` if its build runs Python. Reword CS-ST-7 to "build scripts only for allowlisted crates; each addition is a SEC-14 decision". Rerun A.7's verification against the M1 dependency set.

### M5. Checked-in generated code needs repository rules, or it will swamp review, merges and the AI hook

- **Location:** CS-ST-7, CS-D4, `tools/checks.sh` (`IZCAL_REVIEW_MAX_LINES=4000`, `size` check at 5 MB).
- **Problem:** `hg-bindings` is "the largest single body of Stage 1 code" (Stage 1 5.3), generated from the full WPT `interfaces/` set. Generator changes will produce diffs of tens of thousands of lines.
  - Parallel agent worktrees (CONTRIBUTING requires one per session) will conflict on generated files constantly.
  - The AI review's 4000-line cut fills with generated code.
  - Clippy, rustfmt and `mdlinks`-style checks run over it.
  - Single generated files may hit the 5 MB `size` check.

  CS-D4's reversal trigger ("reviewers report them as a burden") is not measurable.
- **Proposed change:**
  - Mark `**/src/generated/**` as `linguist-generated` and `merge=izcal-regen` in `.gitattributes`, and document conflict resolution as "take either side, run `cargo xtask codegen`".
  - Exclude it from the AI review and from the `cs-patterns` hook; `codegen-fresh` is the check.
  - Emit one file per interface, so files stay small and conflicts stay local.
  - Measure generated lines and repository growth at M2.
  - Make CS-D4's trigger objective: generated output above N MB† or above X% of clone size, or `codegen-fresh` merge conflicts on more than Y%† of PRs over a month.

### M6. The instruction-count gate needs four details to be deterministic

- **Location:** CS-PF-4, CS-D5, CS-T6.
- **Problem:** Instruction counts are deterministic only when:
  - (a) hashing is seeded, but CS-AS-2 makes production seeds random;
  - (b) kernels run single-threaded, since Rayon work-stealing under Valgrind changes counts;
  - (c) base and head are measured in the same job on the same runner image, because toolchain and glibc `ifunc` choices drift between runs;
  - (d) the benchmark binaries do not link V8 (open question 11.1).

  On (d), G1 helps: style and layout kernels depend only on `hg-tree-traits`. The `Perf:` trailer is also self-approved while there is one maintainer.
- **Proposed change:** write (a)–(c) into CS-PF-4. Require gate benchmarks to live in crates that do not link `v8`, which G1 and G6 already make possible: say so as a design benefit. Have `Perf:` trailers listed in the PR summary like `#[expect]` (H8c).

### M7. Commit trailers do not survive squash merges, and `fix/*` can be avoided by naming

- **Location:** CS-TE-3, CS-PF-4, CS-PF-8, CS-T8; CONTRIBUTING "squash merge".
- **Problem:** The project squash-merges, and the PR title becomes the subject. Per-commit `Test:`, `Perf:`, `Binary-Size:` and `Spec:` trailers are lost or garbled unless the squash body carries them. CS-T8's "100% of `fix/*` PRs" is satisfied by naming a bug-fix branch `feat/…`.
- **Proposed change:** make the PR description the source of trailers. CI parses the PR body, and the repository setting "default squash message = PR title and description" keeps them in history. Trigger `test-fails-on-base` by a `Fixes:` or issue link (or a `bug` label) rather than the branch prefix. CS-T8 measures PRs that close bug issues.

### M8. Logging, tracing and telemetry privacy are missing

- **Location:** CS-AS-15 ("substrate trace sink"), open question 11.1, CS-ER-14, Section 6.12.
- **Problem:** The standard bans printing but gives no rules for what replaces it. Missing:
  - levels and categories;
  - what may be logged in which tier (URLs, origins, cookies, headers, page text);
  - crash keys;
  - the zero-cost-when-off requirement;
  - DST trace determinism (fields in scheduler order);
  - what happens to dependencies that log through the `log` crate. Stylo does, and `log::set_logger` is process-global state that someone must install in each process and in the simulator.

  CS-ER-14's ast-grep rule catches only literal format arguments in panic macros, not `Debug` of a `Url` in a trace line.
- **Proposed change:** add **CS-ER-19 / CS-AS-16, diagnostics**, with a decision **CS-D9** (own trace facade over `hg-substrate` vs the `tracing` crate as an unmodified dependency):
  - Page-derived values (`Url`, `Origin`, header values, DOM text) are wrapped in `hg_types::Sensitive<T>`, whose `Debug` and `Display` redact in release builds and in crash and telemetry paths.
  - The substrate installs the `log` adapter in every process, and in the simulator as an ordered trace event.
  - Telemetry fields come from a closed enum, never free text.
  - Tracing disabled at compile time costs nothing (an allocation test).

### M9. No standard covers Izcal-authored C++ and the hand-written FFI declarations

- **Location:** Section 9.1 ("the `rusty_v8` fork keeps upstream's conventions … only the existing hooks apply"); CS-ER-17; G6.
- **Problem:** The fork's Izcal-authored C++ (the predictable platform, which is load-bearing for DET-1, and the `cppgc` API additions) and its Rust `extern "C"` declarations are the riskiest FFI in the project. Rusty_v8's bindings are mostly hand-declared, so a signature or layout mismatch is silent undefined behavior. Keeping upstream conventions for upstream code is right, but Izcal's own additions need rules.
- **Proposed change:** add a short **CS-FFI section** for Izcal-authored fork code and any project C or C++:
  - clang-format with the fork's style; `-Werror`; no exceptions; no `abort` reachable from embedder input;
  - every `extern "C"` declaration has a layout or ABI test: `static_assert(sizeof/alignof)` on the C++ side matched by `const` asserts on the Rust side, generated where possible;
  - ASan and UBSan builds of the fork's tests in T1;
  - `// SAFETY:`-equivalent comments on C++ functions that take raw pointers from Rust;
  - review by the `rusty_v8` owner;
  - each Izcal commit in the fork stays upstreamable (CONTRIBUTING already requires that).

### M10. Platform-specific code: one clippy run cannot see the other two OSes, and `#[expect]` breaks across `cfg`

- **Location:** A.5 `clippy` check; CS-ER-4; CS-ST-6; gap on platform code organization.
- **Problem:**
  - Clippy on Linux does not lint `#[cfg(target_os = "macos")]` or `"windows"` code: Mach and Win32 IPC transports, sandbox code, GPU backends. G3 bans and tier P lints are unenforced there.
  - An `#[expect]` that fires on one OS or feature set but not another raises `unfulfilled_lint_expectations` under `-D warnings` on the other.
  - `--all-features` also lints a feature combination no binary ships (`ai` plus defaults), while missing per-binary combinations.
- **Proposed change:**
  - Run clippy on all three OSes in T0. Linux is required; macOS and Windows can run in parallel, with T1 as the fallback if runner cost is a problem.
  - Document `#[cfg_attr(target_os = "…", expect(...))]` as the pattern for platform-dependent expectations.
  - Add a structure rule: `cfg(target_os)` appears only in `hg-substrate-os`, `hg-ipc` transport modules, the sandbox and `hg-gpu` backends, one module per OS (`os/linux.rs`, `os/macos.rs`, `os/windows.rs`), enforced by a hook.
  - Run clippy per binary with default features in T1 as well as `--all-features`.

### M11. The deterministic-hasher mechanism is not specified, and the obvious implementation is nondeterministic

- **Location:** CS-AS-2.
- **Problem:** "A hasher keyed from `Entropy::stream(HashSeed)`" leaves open how `HashMap::default()` gets its key without an argument. Drawing a key per map from a stream makes keys depend on map-creation order, which differs across fork/join workers. That breaks DET-1. A key held per simulated process needs process-local global state.
- **Proposed change:** specify the design. Use one key per OS process, set once at startup by `hg-substrate-os` (random) or by the simulator (from the seed), and stored in a `OnceLock` in `hg-types`. Name this as the one sanctioned piece of global state in `hg-types`, with the reason. Never draw a key per map. Benchmarks use the simulator key (M6).

### M12. A nondeterminism inventory for every package is too much work and will be filled in carelessly

- **Location:** CS-AS-13 ("every third-party package in `cargo metadata`, dev-dependencies included").
- **Problem:** A browser with wgpu, WebRender, Stylo, AccessKit and the `windows`/`objc2` stacks pulls in hundreds of packages. Requiring an entry for each, dev-dependencies included, invites copy-paste entries ("none") that nobody verifies. The spec admits the meta-test finds leaks only after the fact.
- **Proposed change:** require entries only for packages linked into code that runs under `hg-substrate-sim`, a subgraph `hg-graph-lint` can compute. Other packages get the generated default "outside simulation". Pre-fill each entry from a scan of the package source for `std::thread`, `std::time`, `std::env`, `RandomState`, `rand` and `getrandom` (ast-grep over the registry sources), so the human confirms rather than invents. Re-scan on every version bump.

### M13. 100 rules is too many, and the one-page list shows the wrong 16

- **Location:** "The rules on one page"; Sections 6.4–6.10.
- **Problem:** About 14 rules only restate a lint's own documentation (CS-ER-4, CS-ER-11, CS-TY-8, CS-AS-14, CS-ST-4, CS-ST-9, CS-ST-10, CS-ST-11, CS-ST-12, CS-ST-15, CS-PF-2, CS-PF-7, and parts of CS-AS-15 and CS-ST-6). About 5 restate other specs (CS-V8-7, CS-V8-8, CS-ST-8, CS-ST-16, and CS-TE-1, which is G3 plus a CI flag). Lint-enforced rules announce themselves through the compiler, so they need no place in a summary. Meanwhile the one-page list gives space to the never-`#[allow]` rule (item 4, a lint), build scripts (item 13, rare, enforced by cargo-deny) and synchronous contracts (item 12, architecture), and leaves out "which tier am I in", unsafe, and "no page data in logs or errors".
- **Proposed change:**
  - Move the lint-restating rules into the A.1/A.2 tables as annotated rows ("lint X — why"), keeping their IDs as aliases so existing reasons stay valid.
  - Merge CS-ER-1 + CS-ER-5 + CS-ER-16 into one per-tier panic policy, and CS-TY-10 + CS-TY-11 + CS-V8-4 + CS-V8-6 into "no borrow across script or GC".
  - Mark rules that restate other specs "see X" in one line. That leaves about 60–65 rules.
  - Rebuild the one-page list around the rules **no tool checks** and the decisions a writer makes:
    1. know your tier (`cargo xtask tier <crate>`);
    2. errors vs `invariant!`;
    3. checked arithmetic, never wrapping, in tier P;
    4. no borrow across script or GC, and finalization discipline;
    5. only the substrate touches the OS or threads;
    6. queue and blocking policy;
    7. never hand-write a V8 or IPC seam;
    8. unsafe: inventory, SAFETY comment, Miri;
    9. no page data in panics, logs or exceptions;
    10. parse at the boundary;
    11. deterministic tests and a fails-before-fix regression test;
    12. spec-first tests for AI code;
    13. never edit the gate to pass it (H8);
    14. hot-path budgets.

### M14. Toolchain pinning ignores nightly tools, and no upgrade cadence is set

- **Location:** CS-ST-9; CS-TE-11/13 (cargo-fuzz and Miri need nightly; Kani ships its own toolchain); Section 10 ("nursery lints change").
- **Problem:** The pinned stable toolchain cannot run Miri or cargo-fuzz. Without a pinned nightly, those jobs drift and flake, which breaks CS-TE-5's spirit. There is no cadence for bumping the toolchain, Stylo, wgpu or WebRender, so bumps arrive as large surprise PRs that also churn nursery lints.
- **Proposed change:** add `rust-toolchain-nightly.toml` (a dated nightly), used only by the Miri, fuzz and loom jobs, bumped together with stable. Set a bot-driven stable bump every second release† (about 12 weeks), whose PR includes the lint diff and drops misfiring nursery lints with reasons. Set exact-pinned L0 dependency bumps as a monthly bot PR† per owner (Stage 1 7.7 table), with security bumps immediate.

### M15. Inventoried crates have no permitted way to use `unsafe`, and unsafe review lacks process detail

- **Location:** A.1 `unsafe_code = "deny"` in `[workspace.lints]`; A.2 "Crate-level expectations the graph lint permits (and no others)"; CS-TY-13; G5.
- **Problem:**
  - Tier R inventoried crates (`hg-dom`, `hg-bindings`, `hg-script`) inherit `unsafe_code = "deny"` through `[lints] workspace = true`. A.2 permits no crate- or module-level expectation for `unsafe_code`, so as written they cannot compile.
  - The process is thin. Who reviews an unsafe diff (G5 names an owner but nothing routes the review)? What does a SAFETY comment have to contain? Is unsafe growth tracked? The research note's T-13 called for Kani harnesses per unsafe block, which was dropped.
- **Proposed change:**
  - Permit `#[expect(unsafe_code, reason = "G5: <inventory entry>")]` on **modules** (not crates) listed in `unsafe-inventory.toml`, and have the graph lint check that the module paths match.
  - Route `CODEOWNERS` by inventory owner.
  - Define a SAFETY-comment shape: precondition, why it holds here, what would break it.
  - Add a target: unsafe blocks per inventoried crate are published per milestone and must not grow without a decision-log line.
  - Restore "Kani per unsafe block where the block is pure Rust", or say why it was dropped.

### M16. Generated code has no standard of its own, and generator-emitted expectations can hollow out tier P

- **Location:** Section 6.1 ("the generator emits whatever `#[expect]` attributes it needs"); A.2; tier T for the generators.
- **Problem:** `hg-ipc-codegen` and `hg-webidl-codegen` are the mechanism for CS-ER-6, CS-ER-12, CS-V8-1, CS-V8-2 and CS-V8-5: the most security-critical code in Stage 1. Yet they are tier T (panics allowed, docs optional), and their output may suppress any lint. A generator that emits `#[expect(clippy::indexing_slicing)]` on every decoder makes tier P meaningless for the IPC parsers.
- **Proposed change:** add **CS-ST-17, generated code**:
  - (a) Generated tier P code must pass the tier P table with only a fixed, listed set of expectations (for example `too_many_lines`, `missing_docs`). The list lives in `tools/lints.toml`, under the same CODEOWNERS as H8a.
  - (b) Generator changes carry a golden-output diff on a fixed IDL sample in the PR, and need the security owner when they touch IPC or V8 seams.
  - (c) Generators are tier T for panics but follow tier P review rules.
  - (d) Generated output is deterministic: same input, byte-identical output across OSes, with `codegen-fresh` run on all three.

### M17. Several targets either cannot fail or have failure actions with no consequence

- **Location:** Section 8.
- **Problem and change, per target:**
  - **CS-T2:** "a cleanup item enters the next milestone" has no consequence. Change to: above the bound, new expectations in that crate need the tier P owner's approval until it is back under. Count `invariant!` (H7).
  - **CS-T3** measures the document, not the code, and the draft already passes it (90 of 100). Change to: "≥70%† of rules have a mechanism **running in CI** by the end of Phase 2", measured by the canary and job list. Also note that some listed mechanisms are heuristics (`cs-primitive-id-param`, `cs-float-par-reduce`, which cannot see types).
  - **CS-T7:** per-quarter samples of lab regressions in CPU-bound kernels will be single digits. Use a rolling 4 quarters with a minimum count before the rule applies.
  - **CS-T12:** say what is counted. The Rust global allocator in the renderer's script and paint path and the compositor's Rust code count; V8, `cppgc` and WebRender's own C++ allocations do not. Also say whether a frame with no change should run at all.
  - **Add CS-T13, expectation honesty:** a quarterly sample of 20† new tier P `#[expect]`/`invariant!`/`cs-allow` sites re-reviewed by someone other than the author; ≥95%† of reasons must be true. Below that, the AI review rule for reasons becomes blocking.
  - **Add CS-T14:** generated-code size and `codegen-fresh` conflict rate (M5).

### M18. Decisions: two reversal triggers are not triggers, and several real decisions are missing

- **Location:** Section 9.3.
- **Problem and change:**
  - **CS-D2:** "re-measured at each stage boundary" is a schedule, not a trigger. Make it "the measured cost moves across 1%† at any re-measurement". Note that CS-D2's 1% and DET-2's 1% substrate budget both draw on the same P-1 headroom; state which one gives way first.
  - **CS-D4:** the trigger is subjective (M5).
  - **CS-D6:** option (a) needs a mechanism, not "the PR states which session". Make it mechanical: the spec-derived tests land in an earlier commit or PR, marked expected-fail. CI shows they fail at that commit and pass after the implementation. The test commit's `Claude-Session` differs from the implementation's.
  - **CS-D7:** recommendation (a) holds, because SEC-12 keeps utility processes to "one (site, container) per process at a time". Cite that, and make the trigger "SEC-12 relaxes to multi-site utility processes".
  - **Missing decisions:**
    - CS-D8, the async model (H1);
    - CS-D9, the diagnostics facade (M8);
    - CS-D10, how tier lint levels are applied. The alternative to generated full `[lints]` tables is a generated, marker-delimited `#![deny(...)]` block at each tier P crate root on top of `[lints] workspace = true`, which Cargo allows and which keeps one inheritance path. Recommend whichever `xtask` makes simpler, but record the choice;
    - CS-D11, whether tier membership is declared or computed (M3);
    - CS-D12, the Phase 1 split (H6).

---

## Low

- **L1. Sync IPC vs "no task blocks."** CS-AS-7 bans waiting on another sequence, while Stage 1 4.1 allowlists sync IPC (`document.cookie`, `localStorage`, sync XHR). Name the allowlist as the sanctioned exception in CS-AS-7, and require the substrate's long-task detector to attribute sync-IPC time separately.
- **L2. `wildcard_enum_match_arm` applies to all enums.** CS-TY-6 says "Izcal-owned", but the lint also fires on std and dependency enums: wgpu errors in `hg-gpu`, `io::ErrorKind`, Stylo and neqo types. State the expected pattern for external enums (`#[expect]` with "external enum" as the reason, or a wrapper `match` in one adapter module).
- **L3. Type bans vs dependency APIs.** `disallowed-types` on `std::collections::HashMap` and `std::time::Instant` fires when Izcal code *names* a type a dependency returns (an adblock or AccessKit API returning a std `HashMap`, a WebRender API taking `Instant`). Add an adapter-module pattern with one `#[expect]` per adapter.
- **L4. `cs-allow` markers never expire.** Unlike `#[expect]`, a `// cs-allow(...)` comment stays after the code it excused is gone. Have `cs-patterns` fail on a marker whose line no longer matches its rule.
- **L5. Timeouts as a flake source.** nextest's wall-clock `slow-timeout` on loaded CI runners produces failures that are not determinism bugs, which undermines CS-TE-5's "a flake is a determinism bug". Medium tests should bound work in simulated time or in scheduler steps. Count wall-clock terminations as infrastructure failures in CS-T4.
- **L6. `test-verify` on small tests.** Rerunning a pure, deterministic small test 10× in nextest's process-per-test model finds almost nothing. Keep the 20 extra seeds for medium tests, and spend the small-test budget on order shuffling inside one process only where tests share statics (which G3 should make rare).
- **L7. Naming and API conventions are thin.** CS-ST-14 covers casing and conversions only. Add: unit suffixes where a newtype is not used (`_px`, `_bytes`, `_ms`); `try_` for fallible twins; `Id` suffix for ID newtypes; per-operation error types named `<Op>Error`; `new` never fails (`try_new` or `parse` do); boolean parameters forbidden in public signatures (already the TY-4 lint, so make it the stated convention).
- **L8. No PR template or review checklist.** Add `.github/pull_request_template.md` from the rule index: tier of touched crates; new `#[expect]`/`invariant!`/`cs-allow` (auto-listed); unsafe changed; `Spec:`, `Test:`, `Perf:`; generated code regenerated; gate configuration touched (H8). Reviewers then check the review-only rules (CS-TY-5, CS-TY-7, CS-TY-11, CS-PF-6, CS-PF-12) explicitly.
- **L9. Secrets handling.** Cookies at rest, credentials (SEC-23), TLS secrets and tokens need a type rule: `Secret<T>` with no `Debug` or `Display`, zeroize-on-drop, constant-time comparison, never in IPC error enums or traces. Cross-reference the security scope.
- **L10. Documentation lints on internal crates.** `missing_docs` plus `missing_panics_doc` plus `missing_errors_doc` at warn, with `-D warnings`, on every internal `pub` item, combined with tier R's `expect("invariant")` style, will produce boilerplate (`/// # Panics\n/// If the invariant fails.`). Open question 11.1 already suspects this. Start with these lints on L1–L2 contract crates, tier P and published crates. Widen only if the M2 review shows a need.
- **L11. Lessons dropped from the research notes.** Errors note TY-10: tier P code that uses `RefCell` uses `try_borrow_mut` or avoids `RefCell`; the spec mentions `RefCell` panics only as a lint gap. V8-1: non-wrappable `cppgc` objects use the separate embedder non-wrappable tag range. Testing note T-13: Kani per unsafe block (see M15). Async note A12: `clippy::future_not_send` where futures cross threads (moot if H1 bans most async). Restore them or record why each was dropped in A.8.

---

## What the spec does well

- **Every rule names its mechanism**, and "review only" is said out loud. This is the single most important property for a codebase written largely by agents, and it is rare in coding standards.
- **The tier model** expresses, once, the fact that a panic costs different amounts in different processes. Tying it to the process map instead of to crate names is the right first principle; M3 only asks that the result be declared rather than silently computed.
- **The generators own the dangerous seams.** Generated bad-message handling with no forgettable `report_bad_message`, generated tag ranges with a second Rust-side check, a generated error-to-exception table with static messages, and closed IPC error enums remove whole bug classes structurally rather than by vigilance. CS-ER-6 in particular is a clean answer to crbug 1285384's lesson.
- **Abort everywhere, with the simulator consequence worked through.** The reasoning (V8 `extern "C"` callbacks, unknown state after a panic, exact seed replay) is correct for this architecture, and turning SEC-28's "contain the panic" into "make it unreachable" is the right amendment.
- **The conflicts table (6.3)** resolves real contradictions between sources (RFD 400 vs 609, SEC-14 vs the note's cooldown, `unreachable_pub` vs `redundant_pub_crate`, G3 vs tools) instead of hiding them.
- **Measured tool gaps (Section 3, A.7)**, such as silent `disallowed-methods` paths into unreachable crates and integration-test helper exemptions, led to concrete countermeasures (`hg-lint-canary`, the `#[cfg(test)] mod` layout). That is P6 in practice.
- **The determinism discipline** extends G3 to hash seeds, async locks, global pools and Stylo's thread pool, adds a dependency inventory and a same-seed meta-test, and gives the gate a no-retry, quarantine-with-expiry policy. That is coherent and matches B1.
- **The performance approach**: instruction counts per PR, the lab as authority, backout first, size budgets as `const` asserts and explicit override trailers. All are cheap, deterministic and agreed in advance.
- **Checked-in generated code without `build.rs`** is the right call for supply-chain and review reasons (the M5 issues are repository plumbing, not a reason to reverse it).
- **Spec-first tests for AI-generated code (CS-TE-17/CS-D6)** attack the actual failure mode of LLM-written oracles, and are made checkable by `test-fails-on-base` and mutation reports.
- **Discipline about provenance and scope:** M-1 is clean, amendments to other documents are proposed rather than applied, every † threshold is flagged, and A.8 makes the research-note lineage traceable.
