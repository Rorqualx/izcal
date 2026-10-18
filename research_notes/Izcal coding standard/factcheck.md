# Fact-check: Izcal coding standard (draft 0.1)

*Independent fact-check of `reports/Izcal coding standard.md` (draft 0.1, 822 lines, 100 CS rules), 18 October 2026. Method: the appendix configs (A.1 workspace lints and profiles, a generated A.2 tier P table, A.3 `clippy.toml`, A.4 `deny.toml`) were copied verbatim into a throwaway workspace and run on rustc/clippy 1.99.1 (2026-10-13) and cargo-deny 0.20.2, with test crates that exercise every banned path and every measured claim. External sources were fetched directly; internal references were checked against the documents on disk. The spec and repo files were not edited.*

## Summary

| Area | Checked | Confirmed | Wrong | Partly right / overstated | Unsupported | Unreachable |
|---|---|---|---|---|---|---|
| Toolchain: lint names, groups, levels (A.1, A.2, A.7) | 94 lints | 94 names and groups | 0 | 4 (levels or edition defaults described wrongly) | — | — |
| Toolchain: config keys (`clippy.toml`, `deny.toml`, Cargo) | 41 keys | 41 parse | 0 | — | — | — |
| Toolchain: behavior claims (measured in the spec, mechanisms) | 24 | 15 | 3 | 6 | — | — |
| External sourced claims | 106 | 78 | 6 (incl. #31 P/W) | 21 (incl. #11 P/U) | 1 | 0 |
| Internal references (other Izcal docs, repo files) | about 120 | about 99 | 7 | 10 (paraphrase, missing source doc, conflict) | 4 | — |
| Internal consistency | 14 checks | 9 | 2 | 3 | — | — |

**Verdict.** The toolchain work is sound: every lint name, group and config key in the appendix is correct for 1.99.1, the configs parse, and every disallowed path fires. The measured claims in Section 3 reproduce. The real problems are **mechanisms that do not do what the rule says** (five rules whose named lint is silent or covers something else) and **a crate-level expectation that will break CI**. The external evidence is mostly right: 78 of 106 claims are confirmed, 21 are overstated or need a caveat, 6 are wrong and 1 is unsupported. Several are attributed to the wrong source.

## Required corrections, by impact

1. **The single crate-level G3 expectation will fail CI (Section 6.2 tier O/T, A.2).** `#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "G3 …")]` emits `unfulfilled_lint_expectations` for each lint the crate never triggers (measured: a crate that calls a banned method but uses no banned type gets the warning). Under `-D warnings` that is a CI failure in every tier O/T crate that does not use both a banned method and a banned type. The spec forbids `#[allow]` (CS-ER-4), so this needs a stated exception: either `#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = …)]`, permitted by `hg-graph-lint` only in tier O/T, or a generated `[lints.clippy] disallowed_* = "allow"` in those tiers' tables. The second is cleaner, since the tables are already generated.
2. **The blanket G3 exemption also exempts the non-G3 bans (CS-ER-2, CS-ER-15, CS-AS-6, CS-AS-8, CS-AS-10, CS-V8-3).** The `disallowed-methods`/`-types`/`-macros` lists mix G3 entries with `catch_unwind`, `resume_unwind`, `set_hook`, `Backtrace::capture`, unbounded channels, async mutexes and `v8::External::new`. A crate-wide allow in tier O (`hg-substrate-os`) and tier T switches all of them off. That contradicts CS-ER-2 ("MUST NOT be used outside tier T"), because tier O gets `catch_unwind` too, and CS-AS-6, because the substrate could create unbounded channels. Fix: clippy cannot scope a lint by entry, so either accept this and say so per rule, or move the non-G3 entries into a check (`ast-grep` or graph) that does not share the exemption.
3. **CS-V8-1's range check is not available through rusty_v8.** `v8::Object::unwrap::<TAG, T>` in rusty_v8 152.2.0 and in Izcal's own fork (`m0/rusty_v8/src/binding.cc:4836`) builds `CppHeapPointerTagRange(tag, tag)`, so it checks one exact tag. The inheritance-range check CS-V8-1 depends on needs a new `unwrap` binding in the fork that takes `(first, last)`. That adds to the fork's divergence (S-4e) and should be named in Stage 1 5.3 / HGBA-2 4.8 work. Also, v8-sandbox.h does not prescribe depth-first pre-order or `[id(I), id(last descendant)]`. In its own example the parent's id is not at the start of its range. Pre-order numbering satisfies V8's rule, but it is Izcal's scheme and should be labeled that way. Tags are 15-bit (`kLastTag = 0x7fff`), and rusty_v8 asserts `TAG < 0x7fff`.
4. **CS-ER-17's lint does not check what the rule says.** `ffi_unwind_calls` fires only on **calls to** `extern "C-unwind"` functions, and only when the crate is compiled with `panic=unwind`. Measured: it is silent in the `dev` profile (abort) and fires only in the `lib test` build. It never fires on a hand-written `extern "C-unwind" fn` definition. So "No hand-written `extern "C-unwind"`" needs an `ast-grep` rule. `ffi_unwind_calls` can stay, but as a test-build-only check.
5. **CS-AS-4's lint does not enforce "public contracts are synchronous".** `async_fn_in_trait` (deny) fires on `async fn` in a publicly reachable trait (measured). But its own suggested fix, `fn f(&self) -> impl Future<Output = T> + Send`, passes cleanly (measured). The rule needs an `ast-grep` or graph check for `-> impl Future` and `Pin<Box<dyn Future>>` in `pub` trait methods, or the mechanism column should say "partial".
6. **The rustdoc lints never run in the CI the spec defines (CS-T1, CS-ST-13).** `[workspace.lints.rustdoc]` applies only under `cargo doc`/`rustdoc`. Measured: a broken intra-doc link fails `cargo doc` but passes `cargo clippy -- -D warnings`. A.5 has no `cargo doc` job, so CS-T1's "0 rustdoc warnings" and `broken_intra_doc_links = "deny"` are unenforced. Add `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --document-private-items` to T0, and count it in CS-T9.
7. **A misspelled reachable path in `clippy.toml` does not fail CI.** Section 3 and A.7 are right that it is "reported", but it is a configuration warning, not a lint. Measured: `cargo clippy -- -D warnings` exits 0 with "`std::fs::reed` does not refer to a reachable function". A module path (`std::fs`) is also reported ("expected a function, found a module"), not ignored. So Section 3's "not modules" should read "module entries are rejected with a warning". The canary (A.6) is therefore needed for typos in reachable paths as well, not only for non-dependencies. Alternatively, `tools/checks.sh` can fail on that warning text.
8. **CS-ST-7's build-script allowlist is far larger than "the `v8` fork and platform FFI crates".** With A.4's `[bans.build]` and only ten common dependencies (rand, rayon, tokio, futures, crossbeam, flume …), cargo-deny reported 13 `build-script-not-allowed` errors: `libc`, `proc-macro2`, `quote`, `rayon-core`, `crossbeam-utils`/`-epoch`/`-deque`, `getrandom` 0.3, `zerocopy`, `rustversion`, `wasm-bindgen`(-shared) and `wit-bindgen`. `thiserror` 2.x and `serde` also ship `build.rs`, and CS-ER-10 allows `thiserror`. Rayon, needed for fork/join and Stylo, cannot be used without allowlisting `rayon-core` and crossbeam. The rule text and the Phase 1 effort (9.2) should expect a reviewed list of tens of crates. Also add `[graph] targets = [...]`, so wasm-only crates (`wasm-bindgen`, `wit-bindgen`) stop showing up through `all-features` with every target included.
9. **CS-ST-7 cites SEC-14 (d) for something SEC-14 (d) does not say.** On disk, SEC-14 (d) is "0 new crates without a recorded human decision". Nothing in the security scope gives the security owner sign-off on build-script allowlist additions. Either propose it as a SEC-14 amendment in 9.4, or cite (d) only for "a recorded human decision".
10. **Tier S contradicts Stage 1 (Section 6.2).** "`hg-substrate-sim`… never linked into a shipping binary (graph check)" conflicts with Stage 1's M1 exit ("Browser, renderer and network processes start under both substrates", Stage 1 L396) and with renderer-in-simulation fuzzing (L360). If the shipping `hg-*-main` binaries can start under the simulated substrate, then either `hg-substrate-sim` is linked into them, or simulation needs separate binaries. Either way the tier rule and the graph check need restating against Stage 1. Ask the Stage 1 owner which is meant.
11. **Wrong external facts that drive text:**
    - **CS-PF-5:** Firefox's perf-sheriffing page has no "3-day triage" rule. The policy is daily review, a 1-business-day developer response, help after 5 business days, and inactivity after 1 week. CS-PF-5's "Alerts are triaged within 3 days" needs a new basis or a †.
    - **CS-PF-10:** the Android post's 5–20% figures measure **rustc built with PGO/LTO/BOLT compiling faster**, not faster Rust programs. They do not support "PGO and LTO are worth 5–20%" for Izcal's binary.
    - **Section 2, "Quantum DOM":** labeling was not required. Unlabeled tasks only lost preemption. Bug 1485216 removed the unused CooperativeThreadPool/Scheduler. The part that "showed up in performance profiles" was LabeledEventQueue, which *was* in use. "A scheme correct only if all metadata is right" overstates the source.
    - **Section 4, "SE-0466":** this is an opt-in, per-module MainActor default. Swift did not "move back toward main-actor defaults" language-wide; the proposal rejected changing the default.
    - **Section 2, "arrayref":** the malicious crate was `proc-macro1`, which the compromised `arrayref` 0.3.10 (also `internment` 0.8.7 and `append-only-vec` 0.1.9) was made to depend on. The post does not say "resolving and building was enough". Name `proc-macro1` and drop or soften that sentence.
    - **CS-ER-8:** the 5 Dec 2025 Cloudflare outage was a Lua nil-index error in FL1. The blog says the Rust FL2 code did **not** have the bug. It is valid evidence for "config is input", but must not read as a second Rust failure.
    - **CS-ER-3:** security scope 3.3 names quinn-proto panics and a rustls pre-auth allocation. It never mentions h2, and rustls-webpki appears elsewhere with no panic.
    - **CS-TE-5:** Haben et al. say flaky tests *revealed* more than a third of regression faults (Section 2 has this right). CS-TE-5's "Retries hid over one third of regression faults" is not in the paper.
    - **CS-AS-4:** "the 2026 goal only settles a design" quotes the in-place-initialization goal, not afidt-box. afidt-box plans nightly implementation work.
    - **CS-ST-11:** rustc tidy's file-length suppression (`// ignore-tidy-file-filelength`) needs no reason. It is not "a reasoned suppression".
    - **Section 4:** TigerBeetle's rule is an *average* of two assertions per function, and Gallant's noise example is not `expect("no panic")`.
12. **Lint-level statements that are wrong for this workspace:**
    - **Section 6.1:** "`deny` additionally stops a local `cargo check`" is true only for rustc lints. `cargo check` does not run clippy lints at all.
    - **A.7:** "the rest allow" uses the edition-2015 defaults from `rustc -W help`. In edition 2024, `unsafe_op_in_unsafe_fn` warns by default, and `unexpected_cfgs` warns too (A.7 lists only three warn-by-default lints).
    - **A.1 and CS-TY-13:** `unsafe_attr_outside_unsafe` and `missing_unsafe_on_extern` are edition-2021 migration lints. In edition 2024 an unsafe attribute without `unsafe(...)` and an `extern` block without `unsafe` are hard errors (measured: no lint attribution). The two lines are harmless but enforce nothing. Say so, or drop them.

## 1. Toolchain facts (local, rustc/clippy 1.99.1, cargo-deny 0.20.2)

Scratch workspace: `/private/tmp/claude-501/-Users-joederas-Documents-Cline-code-browser/c3bb02be-ac0c-419a-b09b-4f21425fd5ce/scratchpad/cs-factcheck/ws/`. Its root `Cargo.toml`, `clippy.toml` and `deny.toml` are A.1, A.3 and A.4 verbatim. Test crates:
- `tr`: tier R, using `workspace = true` and every banned path.
- `tp` and `tt`: generated tier P tables, also covering test exemptions, the hash alias and `async_fn_in_trait`.
- `tu`: crate-level expectation and FFI lints.

### 1.1 Lint names, groups, default levels

The groups and default levels come from `clippy-driver rustc -W help`, parsed per group.

| Item | Spec says | Measured | Verdict |
|---|---|---|---|
| 31 restriction lints listed in A.7 | restriction | all restriction, default allow | Confirmed |
| 20 pedantic lints in A.7 | pedantic | all pedantic, allow | Confirmed |
| `significant_drop_tightening`, `non_send_fields_in_send_ty`, `fallible_impl_from`, `redundant_clone`, `needless_collect`, `large_stack_frames`, `transmute_undefined_repr` | nursery | nursery, allow | Confirmed |
| `await_holding_lock`/`_refcell_ref`/`_invalid_type`, `let_underscore_future`, `arc_with_non_send_sync` | suspicious | suspicious, warn | Confirmed |
| `result_large_err`, `large_enum_variant`, `expect_fun_call` | perf | perf, warn | Confirmed |
| `disallowed_methods`/`_types`/`_macros`, `missing_safety_doc` | style | style, warn | Confirmed |
| `vec_box` | complexity | complexity, warn | Confirmed |
| `negative_feature_names`, `redundant_feature_names`, `wildcard_dependencies` | cargo | cargo, allow | Confirmed |
| `redundant_pub_crate` (6.3) | "off" | nursery, allow, so off by default | Confirmed |
| `exhaustive_enums` (Section 2 link) | — | restriction | Exists |
| `string_to_string` | removed | not in 1.99.1 | Confirmed |
| rustc `unused_must_use`, `async_fn_in_trait`, `improper_ctypes` | warn by default | warn | Confirmed |
| rustc `let_underscore_lock`, `arithmetic_overflow`, `unconditional_panic` | deny by default | deny | Confirmed |
| A.7 "the rest allow" | allow | `unexpected_cfgs` warns; in edition 2024 `unsafe_op_in_unsafe_fn` warns | **Partly wrong** (correction 12) |
| `unsafe_attr_outside_unsafe`, `missing_unsafe_on_extern` as CS-TY-13 mechanisms | lint | in edition 2024 the compiler errors first; the lints are migration-only | **Redundant** (correction 12) |
| rustdoc `broken_intra_doc_links`, `private_intra_doc_links` (warn), `missing_crate_level_docs` (allow) | exist | exist with those defaults | Confirmed |
| `clippy.toml` defaults: `future-size-threshold` 16384, `large-error-threshold` 128, `too-many-lines-threshold` 100, `enum-variant-size-threshold` 200 | as stated | `cargo clippy --explain` gives the same | Confirmed; the four keys restate defaults |
| `allow-unwrap-in-consts = true` | set | default is already `true` | Redundant (harmless) |
| CS-ER-14: `expect_fun_call` as a mechanism | lint | perf lint suggesting `unwrap_or_else(\|\| panic!(…))` for `expect(&format!(…))`; it does not look at `panic!`/`assert!` format arguments or at what is formatted | **Weak mechanism.** The `ast-grep` rule is the real check; say so |

### 1.2 Configuration keys

| Item | Verdict |
|---|---|
| A.1 as a whole: `resolver = "3"`, `[workspace.package] publish = false`, `rust-version`, `[workspace.lints.*]`, `unexpected_cfgs` with `check-cfg`, `[profile.checked] inherits = "release"`, `[profile.release.package.hg-navigation] overflow-checks` | Parses. The package-profile entry for a non-existent crate warns "did not match any packages" on every command until the crate exists |
| Package override of `panic` rejected | Confirmed: "`panic` may not be specified in a `package` profile" |
| `[lints] workspace = true` plus `[lints.clippy]` | Confirmed rejected: "cannot override `workspace.lints` in `lints`…" |
| All 22 A.3 keys, including `await-holding-invalid-types` entries with `path`/`reason` and `replacement` fields | Accepted (an unknown key would be a hard error) |
| A.4 keys: `[licenses] version`, `allow`, `confidence-threshold`, `unused-allowed-license`, `private = { ignore = true }`; `[bans] multiple-versions`, `wildcards`, `allow-wildcard-paths`, `deny` with `wrappers`/`reason`, `skip`; `[bans.build]` `allow-build-scripts`, `executables`, `interpreted`, `include-dependencies`, `include-workspace`, `include-archives`, `bypass`; `[sources]` four keys | All deserialize on 0.20.2. `check bans licenses sources` runs; the licenses check passed on this graph |
| `unused-wrapper` is a diagnostic, not a key | Confirmed: seen as `warning[unused-wrapper]` |
| `wrappers` applies to direct parents only | Confirmed: `warning[unmatched-wrapper]: direct parent 'tr' … was not marked as a wrapper` plus `error[banned]` |

### 1.3 Behavior claims

| Claim (where) | Measured | Verdict |
|---|---|---|
| Every A.3 `disallowed-methods`/`-types`/`-macros` entry for a reachable crate fires (std, rand 0.9, getrandom 0.2/0.3, num_cpus, rayon, tokio, crossbeam, futures, flume, async-lock) | All 52 reachable entries fire. `std::hash::RandomState` is reported under its `hash_map::RandomState` path | Confirmed |
| `std::fs::File` type ban covers `File::open`/`File::create` (A.3) | 2 hits | Confirmed |
| Paths into non-dependencies are silent (Section 3, A.7) | `v8::*`, `hg_*`, `notadep::foo`, `rayon::spwan` in a crate without rayon: no diagnostic | Confirmed |
| A misspelled reachable path is reported (Section 3) | Reported as a warning, and **exit 0 under `-D warnings`** | **Partly right** (correction 7) |
| `disallowed-*` do not match modules (Section 3) | A module path warns "expected a function, found a module" | **Partly wrong** (correction 7) |
| The `hg_types::HashMap` alias is flagged only at its definition (CS-AS-2) | `#[expect(clippy::disallowed_types)]` on the alias is fulfilled; uses are not flagged; `iter_over_hash_type` still fires through the alias | Confirmed |
| `allow-unwrap-in-tests` misses root-level helpers of an integration-test crate, and covers `#[cfg(test)] mod` (Section 3, 6.2, CS-TE-9) | Root helper flagged; root `#[test]` exempt; `#[cfg(test)] mod sub;` helper exempt; plain `mod plain;` helper flagged; lib `#[cfg(test)] mod tests` helper exempt | Confirmed |
| `async_fn_in_trait` (deny) enforces CS-AS-4 | Fires on `pub trait { async fn }`, not on a private trait, **not on `-> impl Future`** | **Partly** (correction 5) |
| E0038 for `dyn` with `async fn`, E0658 for RTN (CS-AS-4) | Both reproduced | Confirmed |
| `ffi_unwind_calls` (deny) enforces CS-ER-17 | Silent under `panic=abort`; fires only in the unwind test build, on calls only | **Wrong mechanism** (correction 4) |
| A panic escaping `extern "C"` aborts (Rust 1.81; Section 2, CS-ER-2) | Under `-C panic=unwind`, `catch_unwind` around an `extern "C"` callback that panics still aborts: "panic in a function that cannot unwind … aborting", exit 134 | Confirmed |
| Cargo forces unwind in test builds (6.3) | The `lib test` target compiled with unwind (shown by `ffi_unwind_calls` firing there only) | Confirmed. The abort and unwind builds of a crate do not share artifacts, which is a T0 compile cost CS-T9 should count |
| Crate-level `#![expect(disallowed_methods, disallowed_types)]` for tier O/T (6.2, A.2) | `unfulfilled_lint_expectations` when one of the two is unused | **Breaks CI** (correction 1) |
| `hg-lint-canary` works by unfulfilled expectations (A.6) | Mechanism confirmed: unfulfilled expect is a warning and fails `-D warnings` | Confirmed, with a conflict: the canary must depend on `tokio` (and `v8`), so A.4's `wrappers = ["hg-substrate-os"]` and A.6's "tokio only via `hg-substrate-os`" both reject it. Add the canary to `wrappers` and to the graph allowlist. Depending on `v8` also runs its build script (the prebuilt archive download) in T0 |
| rustdoc lints in `[workspace.lints.rustdoc]` enforce CS-T1 | Fire only under `cargo doc` | **Unenforced in A.5** (correction 6) |
| Section 6.1 "`deny` stops a local `cargo check`" | Only rustc lints run under `cargo check` | **Partly wrong** (correction 12) |
| A.4 `[bans.build]` allowlist "`v8` plus the reviewed list" (CS-ST-7) | 13 build-script errors from a 10-crate graph | **Understated** (correction 8) |
| `allow-wildcard-paths = true` covers workspace path dependencies | Accepted; not exercised with a wildcard path dependency | Not measured |

### 1.4 Coverage gaps in A.3 (rule text vs list)

CS-AS-1 says "`std::fs` files and functions", "`std::net` sockets", "`std::env::{var, var_os, vars}`" and "the `std::time` clock types". The list covers 11 `std::fs` functions plus `File`/`OpenOptions`. Since modules cannot be banned (1.3), these are not caught:
- `std::fs`: `remove_dir`, `create_dir`, `hard_link`, `soft_link`, `read_link`, `symlink_metadata`, `set_permissions`, `exists`, `DirBuilder`.
- `std::os::unix::net::{UnixStream, UnixListener, UnixDatagram}`.
- `std::env`: `args`, `args_os`, `current_dir`, `set_current_dir`, `temp_dir`, `home_dir`, `set_var`/`remove_var` (`unsafe` in 2024).
- `std::thread`: `park_timeout`.

CS-AS-2 bans std's `HashMap`/`HashSet`/`RandomState`, but not `hashbrown::HashMap` (random foldhash seed by default), `ahash::RandomState` or `indexmap` with a `RandomState`. CS-AS-5 names `futures::executor` but A.3 bans nothing from it. None of these are factual errors. They are places where the rule text promises more than the list delivers, and the canary will not catch omissions.

## 2. External facts

Verdicts: C = confirmed, W = wrong, P = partly right or overstated, U = unsupported by the cited source. All sources were reachable. For arXiv PDFs the text was extracted locally.

### 2.1 Incidents, security, V8 and IPC

| # | Claim (spec location) | Verdict | Evidence | Correction |
|---|---|---|---|---|
| 1 | Cloudflare 18 Nov 2025: FL2 `unwrap()` on `Err` after the 200-feature limit; about 5 h 46 min (Section 2, 5, CS-ER-7) | C | "11:20 UTC" to "As of 17:06 all systems … functioning as normal"; "called Result::unwrap() on an Err value"; "limit is set to 200" | Customer impact began at 11:28 |
| 2 | Remediation quote, "harden Cloudflare-generated configuration files in the same way…" (CS-ER-8) | P | The exact text is "Hardening **ingestion of** Cloudflare-generated configuration files in the same way we would for user-generated input" | Fix the quotation |
| 3 | 5 Dec 2025 "nil-field outage" (CS-ER-8) | P | Lua `attempt to index field 'execute' (a nil value)` in FL1; "In … FL2 proxy, which is written in Rust, the error did not occur"; about 25 min | Say Lua/FL1 (correction 11) |
| 4 | Mojo: `ReportBadMessage` does not stop execution (Section 5, CS-ER-6) | C | "ReportBadMessage() does not stop execution of the current function. You must manually return immediately" | — |
| 5 | crbug 1285384 was a missing `return` (CS-ER-6) | C (as cited) | The doc links 1285384 under "reporting a bad message but erroneously continuing onwards" | The bug's own contents were not checked |
| 6 | "can trigger a `CHECK()` to crash the receiving process" (CS-ER-3) | C | Verbatim | — |
| 7 | "never `CHECK()` IPC content" (Section 4) | P | "never use CHECK() to validate the message content…", for **untrusted** senders | Quote it, and scope it to untrusted senders |
| 8 | `CHECK()` fine for trusted senders (CS-ER-16) | C | "When the sender is trusted, CHECK() is acceptable and often preferred." | — |
| 9 | Deno CVE-2024-27934: raw pointer in `v8::External` led to UAF and code execution, 2024-03 | C | GHSA-3j27-563v-28wf, published 2024-03-05 | — |
| 10 | Deno CVE-2023-28445: "buffers resized under Rust slices" (Section 5, CS-V8-6) | P | "Resizable ArrayBuffers passed to asynchronous native functions that are shrunk during the asynchronous operation" | "Rust slices" is an inference |
| 11 | arrayref attack, Rust blog 2026-08-20 (Section 2, CS-ST-7) | P/U | The malicious crate was `proc-macro1`, whose build script downloaded a payload; arrayref 0.3.10, internment 0.8.7 and append-only-vec 0.1.9 were made to depend on it; each was up about 86–107 min. "Resolving and building was enough" is not in the post | Correction 11 |
| 12 | Futurelock: RFD 609, omicron#9259, 2025-10; every API request hung; `select!` over `&mut` plus `tokio::sync::Mutex` (Section 2, CS-AS-10) | C | RFD 609 definition; FIFO tokio mutex handed the lock to the unpolled waiter; issue #9259 opened 2025-10-19 | RFD 609 is undated; cite the issue for the date. The RFD says an unfair mutex would not reliably prevent it either |
| 13 | "No lint exists" (Section 2) | C | RFD 609 lists a Clippy lint as an open question | — |
| 14 | RFD 400 recommends `select!` over `&mut`, and avoiding `tokio::sync::Mutex` (6.3, CS-AS-8) | C | "select! over a &mut reference to the future"; "this guide recommends avoiding tokio::sync::Mutex" | — |
| 15 | V8 requires all children of a class to be in the parent's tag range (CS-V8-1) | C | "all (transitive) child classes … have type ids in the same range" | — |
| 16 | Depth-first pre-order and `[id(I), id(last descendant)]` (CS-V8-1) | U | Not in v8-sandbox.h; its example has the parent's id at the end of the range | Correction 3 |
| 17 | gin performs "a second type check … before casting" (CS-V8-2) | C | "Perform a second type check to assert that we indeed have the expected type before casting." | — |
| 18 | rusty_v8 `unwrap`: pointer stays "on the stack" or moves into a cppgc pointer type (CS-V8-4) | C | v8 152.2.0 `object.rs`, verbatim (also read locally) | — |
| 19 | Chromium FFI doc: a `Result` core plus a thin wrapper (CS-ER-17) | P | "this may require splitting some functions into…" | It is a suggestion, not a policy |
| 20 | Chromium `unsafe.md` requires safety comments (CS-TY-13) | C | "requires that all unsafe code is accompanied by a safety comment" | — |
| 21 | FIDL: purpose-built error enums; stack traces "can unintentionally leak privileged information" (CS-ER-12) | C | Verbatim | — |
| 22 | FIDL `flexible` exists "for separately versioned peers" (CS-ER-12) | P | Version skew is given for flexible *methods*. Flexible *types* are for future members and unknown data | Reword |
| 23 | Firefox IPDL `[Tainted]` (CS-ER-6) | C | "Parameters are required to be validated before using them." | — |
| 24 | netstack3 types its lock order (CS-AS-9) | C | `impl_lock_after!`, `LockAfter` | — |
| 25 | Firefox puts Rust panic messages into crash annotations (CS-ER-14, flagged in 11.2) | C | Bug 1507465: "putting Rust panic reasons into the 'crash reason' annotation since bug 1275780 … web content leaking into the crash reason (bug 1502477)" | Cite bugs 1275780, 1502477 and 1507465, then unflag it in 11.2. Bug 1502477 directly supports CS-ER-14 |

### 2.2 Browser and project practice

| # | Claim | Verdict | Evidence | Correction |
|---|---|---|---|---|
| 26 | Chromium builds Rust with abort (Section 5, CS-ER-2) | P | `BUILD.gn`: `-Cpanic=abort` (official builds use `-Cpanic=immediate-abort`) plus `-Zpanic_abort_tests`, only when `!rust_prebuilt_stdlib`; host build tools keep unwind | "Abort in shipped Rust, including tests" is accurate. "All" is not |
| 27 | Firefox `Cargo.toml` aborts in dev and release | C | Also has `*-megazord` profiles with `panic = "unwind"` | Cite mozilla-firefox/firefox; gecko-dev is the old mirror |
| 28 | Firecracker aborts; cast lints; `missing_debug_implementations` (6.3) | C | All at warn | — |
| 29 | neqo: strict restriction lints plus pedantic and nursery groups (Section 5, 6.13) | C | Groups at warn with `priority = -1`, plus the `cargo` group | — |
| 30 | s2n-quic `clippy.toml` exempts tests; s2n-quic denies `unused_must_use` (Section 5, CS-ER-11) | P | The file is `.clippy.toml` (the cited URL is a 404); the deny is in `s2n-quic-transport/src/lib.rs` only | Fix the link and the scope |
| 31 | Quantum DOM: one process, labeling needed, never enabled, removed in 66, "its queue showed up in profiles" (Section 2, 5, CS-AS-3) | P/W | Bug title "CooperativeThreadPool, Scheduler and LabeledEventQueue are unused", mozilla66. LabeledEventQueue *was* used and is what showed up in profiles. Labeling was incremental, not required | Correction 11 |
| 32 | Chromium: "prefer sequences to physical threads"; locks discouraged | C | "strongly preferred"; "Usage of locks is discouraged in Chrome." | — |
| 33 | Chromium bans `<thread>`, `<chrono>`, `<random>` (Section 2, CS-AS-1) | P | `<chrono>` and the thread support library are banned; for `<random>` only the engines and generators are banned, and the distributions are allowed | Reword |
| 34 | TaskEnvironment mock time does not cover threads it does not own (CS-AS-3) | C | "Delayed tasks on unmanaged base::Thread's … are still real-time" | — |
| 35 | Firefox test-verify: 10 + 5 runs, then chaos mode (Section 2, CS-TE-6) | C | Verbatim. The "2016–2017" dates are not on the page; bug 1357513 is from 2017 | Drop or source the years |
| 36 | Test-disabling thresholds 8% and 30% (11.2) | C | Plus a "2 business days" escalation clock | Can be unflagged |
| 37 | WebKit zero tolerance; backout by default (CS-PF-5) | P | "zero-tolerance policy"; the patch is backed out **or** fixed immediately | "Back out or fix at once" |
| 38 | Firefox 3-day alert triage (CS-PF-5) | **W** | No such rule (correction 11) | — |
| 39 | Mozilla: 3–7% of alerts invalid (CS-PF-5) | C | 3% in April 2021; just under 7% over 6 months | — |
| 40 | Chromium binary-size bot: `Binary-Size:` footer (Section 5, CS-PF-8) | C | "Adding a 'Binary-Size: …' footer … will bypass the bot assertions" | — |
| 41 | Servo: `CanGc`; "a panic waiting to be triggered" (CS-TY-10) | C | Verbatim | — |
| 42 | Servo exhausted file handles through IPC overuse (CS-AS-6) | C | Verbatim, with issues #23910, #33672 and #23905 | — |
| 43 | Servo: "Your PR won't get accepted without a test" (CS-TE-4) | C | Verbatim | — |
| 44 | Servo experience report: "one of our largest areas of unsafe code"; lints for rooting | P | The quote covers foreign values in general (SpiderMonkey and custom allocators), not only GC | Minor |
| 45 | Stylo 0.22: lazy global `STYLE_THREAD_POOL`; the embedder passes its pool to `traverse_dom` (CS-AS-3) | C | `global_style_data.rs` `LazyLock<StyleThreadPool>`; `traverse_dom(…, pool: Option<&rayon::ThreadPool>)` | Threads spawn only when the computed count is > 1 (capped at 6) |
| 46 | Android enables overflow checks by default (Section 2, CS-TY-12) | C | "Overflow checking is on by default in Android for Rust" | — |
| 47 | Android: PGO and LTO worth 5–20% (CS-PF-10) | **W** | The figures are toolchain (rustc) build speed | Correction 11 |
| 48 | MiraclePtr protects manual-memory heaps, not safe Rust (CS-PF-11) | C | `raw_ptr<T>`/BackupRefPtr on PartitionAlloc | The mimalloc link supports "workload-specific gains" only by inference; quote a result from it |
| 49 | Midori: abandonment; overflow and OOM are bugs; an ignored return value is a compile error; more release than debug assertions | C | All verbatim in Duffy | OOM was opt-in recoverable in places |
| 50 | FDB abstracts "network, disk, time, and pseudo random number generator"; avoided dependencies | C | Verbatim | — |
| 51 | WebKit: fine-grained locks are fine when cheap | C | "rarely an excuse for not having one, or even multiple, fine-grained locks" | — |
| 52 | SE-0466: Swift "moved back toward main-actor defaults" (Section 4) | **W** (overstated) | Opt-in per module; the default stays `nonisolated` | Correction 11 |

### 2.3 Rust ecosystem, testing and performance research

| # | Claim | Verdict | Evidence | Correction |
|---|---|---|---|---|
| 53 | rustc-perf: instruction counts detect about 0.3% vs about 1% for wall time, 2019 (Section 2, CS-PF-4) | C | Internals post, 2019-04-14: "(e.g. 0.3%) can be reliably detected with instruction counts" | — |
| 54 | `gungraun` is the renamed iai-callgrind (Section 2) | C | The 0.17.0 changelog (2025-09-21) records the rename; the latest is 0.20.0 | — |
| 55 | `gungraun` on macOS (11.2 flagged, CS-D5) | C (flag justified) | The install docs list Linux and FreeBSD only; Docker is suggested for macOS | Unflag: "unsupported on macOS" |
| 56 | LUCI retries up to 5 times (Section 2) | C | §4: "LuCI automatically reruns the test up to 5 times" | — |
| 57 | Flaky tests "revealed over one third of regression faults" (Section 2) | C | Abstract, verbatim | — |
| 58 | "Retries hid over one third of regression faults" (CS-TE-5) | **W** | The paper says flaky tests *reveal* > 1/3 of faults. It does not say retries hid them | Reword CS-TE-5 to match Section 2. The argument (do not retry) still follows |
| 59 | LLM oracles encode actual behavior (Section 5, CS-TE-17) | C | arXiv 2410.21136 abstract | Its oracles also out-detect EvoSuite's; worth a clause |
| 60 | Octane-tuned optimizations hurt real pages | C | "often had a detrimental effect on real-world scenarios" | — |
| 61 | Coverage correlates "weakly" when suite size is controlled (Section 4, CS-TE-15) | P | "low to moderate correlation" | Use the paper's words |
| 62 | Mytkowicz: wrong in sign under innocuous setup changes | C | Link order and environment size flip an O3 speedup to a slowdown | — |
| 63 | Rust 1.81: `#[expect]` stable; panics escaping `extern "C"` abort | C | Release post, verbatim | — |
| 64 | `try_reserve` stable in 1.57 | C | Verbatim | — |
| 65 | `[workspace.lints]` since 1.74 (CS-ST-10) | C | Verbatim | — |
| 66 | MSRV-aware resolver, default in edition 2024 (CS-ST-9) | C | 1.84 post | — |
| 67 | `async fn` in traits is not `dyn`-compatible (CS-AS-4) | C | 2023-12-21 post; reproduced as E0038 locally | — |
| 68 | The 2026 afidt-box goal "only settles a design" (CS-AS-4) | **W** | That sentence belongs to the in-place-initialization goal. afidt-box plans nightly implementation work (`dyn_box!`, RFCs) | Reword: "nightly-only work in 2026; no stable date" |
| 69 | RTN blocked on the new trait solver (CS-AS-4) | C | "Full stabilization is blocked on the next-gen trait solver work" | — |
| 70 | async-std discontinued (CS-AS-5) | C | README | — |
| 71 | Tokio: std mutex "often preferred" (CS-AS-8) | C | Verbatim | — |
| 72 | Ryhl: 10–100 µs between awaits (6.3, CS-AS-7) | C | Verbatim | — |
| 73 | Rayon `reduce` order "not fully specified"; float `sum` "not fully deterministic" (CS-AS-12) | C | Verbatim | — |
| 74 | loom: bound 2–3 "enough to catch most bugs" (CS-TE-14) | C | docs.rs | — |
| 75 | loom treats some orderings approximately (CS-AS-11) | C | GitHub README: `SeqCst` is treated as `AcqRel`; some executions are missed | Cite the README, not docs.rs |
| 76 | Merged integration tests compile about 3× faster (CS-TE-9) | P | It was Cargo's own test suite (3× compile time, 5× artifacts) | Say whose suite |
| 77 | Edition 2024 merges doctests | C | Edition guide | — |
| 78 | Package overrides cannot set `panic`; tests ignore it (CS-ER-2, 6.3) | C | Cargo profiles reference; also reproduced locally. `-Zpanic-abort-tests` is still unstable | — |
| 79 | Features are additive and unioned; per-package unification unstable (CS-ST-6) | C | `-Z feature-unification`, default "selected" | — |
| 80 | SWE at Google: value lost near 1% flakiness; 0.15%; coverage as a ceiling; sizes enforced (CS-TE-1, -5, -15) | C | Verbatim | Enforcement "varies by language" |
| 81 | Google surfaces mutants in code review; cargo-mutants `--in-diff` (CS-TE-16) | C | Petrović and Ivanković 2018; mutants.rs | — |
| 82 | SQLite fuzzcheck replays historical cases "in every test run" (CS-TE-8) | P | On every `make test`, with a few thousand selected cases | Minor rewording |
| 83 | SQLite assertions only in debug builds (CS-TE-18) | C | `SQLITE_DEBUG` | — |
| 84 | proptest strategies "do not generate invalid values" (CS-TE-10) | P | "aware of simple constraints" | Narrow the claim |
| 85 | proptest failure persistence (CS-TE-8) | C | `proptest-regressions` replay | — |
| 86 | rustc tidy: 3000 lines per file "with a reasoned suppression" (CS-ST-11, 11.2) | P | `LINES = 3000` applies to `.rs` (the doc comment is stale). The directive is `// ignore-tidy-file-filelength`; **no reason is required** | Drop "reasoned"; unflag the `.rs` question |
| 87 | rustc warns not to copy `static_assert_size` (6.3) | C | "Don't copy this macro into your own code. Use a `#[test]` instead." | Note that rustc's advice is a `#[test]`; the standard deliberately chooses the opposite |
| 88 | Cross-crate inlining inferred since 2023 (CS-PF-7) | C | PR #116505, 1.75 | Leaf functions only; off with incremental builds and at opt-level 0 |
| 89 | Perf book: "workloads using real-world inputs are best" (CS-PF-6) | C | Verbatim | — |
| 90 | tonic #1032 shows monomorphization cost (CS-PF-8) | C | About 23% less LLVM IR; build time 21 s to 19.5 s | Small effect |
| 91 | Torvalds: "must simply not be used in the kernel" | C | Verbatim (about the panic-based alloc model) | — |
| 92 | RFC 560, RFC 2945 | C | — | — |
| 93 | Kernel: panicking "should be very rare"; prefer `expect`; `# Panics` (CS-ER-1, -4, CS-ST-13) | C | Verbatim | — |
| 94 | Gallant: "correct Rust programs don't panic" (CS-ER-1) | C | Verbatim | — |
| 95 | Gallant on `expect("no panic")` noise (Section 4) | P | The post's noise examples are `expect("a valid regex")` and similar; "no panic" is not in it | Fix the example |
| 96 | TigerBeetle "≥2 assertions per function" (Section 4) | P | "must average a minimum of two assertions per function", an average density | Reword |
| 97 | TigerBeetle: 70 lines, "put a limit on everything", assertions "a force multiplier", back-of-the-envelope sketches, Yuan 92% | C | Verbatim | — |
| 98 | Clippy defaults: 128 B, 16384 B, 100 lines; `exhaustive_enums` | C | Clippy config docs; also reproduced locally | — |
| 99 | bolero: one harness for fuzzers and Kani (CS-TE-10) | C | Cfg-gated `bolero-kani` | The link pins 0.8.0; the current release is 0.13.7 |
| 100 | Miri cannot run FFI (CS-TE-13) | P | True by default; an experimental, Unix-only `-Zmiri-native-lib` exists | Add the caveat |
| 101 | nextest process per test | C | — | — |
| 102 | rustc UI tests: annotations because auto-blessed snapshots let bad changes through (Section 5, CS-TE-12) | P | "This redundancy helps avoid mistakes since the .stderr files are usually auto-generated" | Use the guide's wording |
| 103 | LLVM requires tests for every fix (CS-TE-3) | C | Verbatim | — |
| 104 | matklad: flat layout for 10k–1M lines; generics at boundaries; checked-in generated code (2022) | C | Verbatim | — |
| 105 | Jewson; anyhow is for applications and captures backtraces under env vars (CS-ER-10, -15) | C | `RUST_LIB_BACKTRACE`/`RUST_BACKTRACE` | — |
| 106 | GhostCell, Yanovski et al., ICFP 2021 (11.2) | C | PACMPL 5 ICFP, Article 92 | Unflag |

## 3. Internal references

The security scope on disk is still **draft 0.1**, the same as the standard's header. No SEC or IPC-R ID cited by the standard has been renumbered or changed meaning on disk; re-check when 0.2 lands.

### 3.1 Wrong or unsupported

| Reference | Used in | Verdict | Evidence on disk |
|---|---|---|---|
| G1–G9 "enforced by `hg-graph-lint`" | Section 1 table | W | Stage 1 L59: "G1–G8 enforced by `hg-graph-lint` … G9 by `hg-anchor`" |
| `hg-substrate-sim` never linked into a shipping binary | 6.2 tier S | Conflicts | Stage 1 L396 (M1: processes "start under both substrates"), L360 (correction 10) |
| Tier table lists every crate | 6.2 | Incomplete | Stage 1 has `hg-tree-traits` (L2, G1/G2) and `hg-semantic-repair` (G8, L70); neither is assigned a tier |
| "Most of the code will be written with AI help (Stage 1 7.2)" | Intro | P | Stage 1 L334 describes AI-assisted work but never says "most" |
| "bad message kills the sender" attributed to Stage 1 Section 4 | Section 1 table | U | That is IPC-R5 (security scope L481) |
| SEC-14 (d) = security-owner sign-off on allowlist additions | CS-ST-7 | W | L712: (d) is "0 new crates without a recorded human decision" (correction 9) |
| Security scope 3.3: "2026 panics in quinn-proto, h2, rustls-webpki" | CS-ER-3 | W | L84: quinn-proto panics/OOM and a rustls 1 GiB pre-auth allocation; h2 is not mentioned |
| Security scope 6.6: "Mojo prefers typed structs to raw values" | CS-TY-1 | U | Not in the document. Mojo's own doc does say it ("use structured types"); cite that instead |
| Security scope 6.6: "Mojo type rules" | CS-TY-4 | W attribution | Those rules are IPC-R3/R4; Mojo is cited there only for R2 and R5 |
| ORC-1: tests written "in a session that does not see the implementation" | CS-TE-17, CS-D6 | U | ORC-1 (HGBA-3 L400) says oracles come before code; the separate session is a **new** requirement. Present it as Izcal's addition |
| `Spec:` trailer enforced by the `commit-msg` hook as an "existing rule" | CS-ST-16 | W today | `.githooks/commit-msg` checks the subject and adds `Hooks-Skipped` only; `hg-anchor` does not exist yet |
| HGBA-2 P1 "security and isolation" | Section 4 | P | HGBA-2 L59: "Security and privacy of the user" |
| P1/P2/P4/P6 with no document named | Section 4 | Ambiguous | HGBA-3 numbers its principles differently (its P4 is Robustness); write "HGBA-2 P1" etc. |
| B2 grouped with IPC-R rules (Section 7: "I-2, B2, VER-1") | Section 7 | Source missing | B2 is HGBA-3 (L141), and the header's HGBA-3 list omits it |
| RES-3, RES-* | CS-PF-5, CS-PF-11 | Source missing | Stage 1 §13.3 (L547–554) |
| DET-1 "ship blocker" | CS-T5, 6.3 | P | HGBA-3 L398: "ship blocker **for simulation mode**" |
| EQ-1 as CS-D6's reversal metric | CS-D6 | P | EQ-1 compares AI-generated vs hand-written defects per KLOC; it has no "(a)-tested vs other gated code" breakdown |
| VER-1 "Kani-checked deserializers" | 6.11 | P | L401 covers renderer-to-broker message types only |
| Licence list "is `CONTRIBUTING.md`'s" | A.4 | P | CONTRIBUTING L128: "MIT, Apache-2.0, BSD, ISC, Zlib, Unicode or MPL-2.0". `Apache-2.0 WITH LLVM-exception` is an addition, and the SPDX forms are an interpretation; say so |
| S-4c "heap thread affinity" | CS-TY-9 | Paraphrase | The rule is in HGBA-2 L338, not that phrase |
| "No Cargo workspace exists; only spike code" | Section 3 | P | True for Izcal code. But `m0/rusty_v8` is the fork submodule with its own `[workspace]`, and three spikes declare standalone workspaces. The `exclude` list in A.1 should also carry `m0/rusty_v8` explicitly (it is under `m0`, so it is already covered) |

### 3.2 Confirmed (summary)

- **Stage 1 (draft 0.11):**
  - G1–G9 content: G3's list, G4 `ai`, G5, G6 crates.
  - Section 1 process map, 2.2, 3.2 (one OS process, crashes injected by the scheduler), Section 4 (`.hgidl`, generated endpoints, Kani).
  - 5.2 (u32+u32 generational IDs, the stale-ID property test), 5.3 (fast-call pointer rule, `test_fast_calls_pointer`).
  - 7.1, 7.2 (T0 ≤ 20 min, clippy listed in T0, `Spec:`, pinned spec text), 7.4, 7.5 (OSS-Fuzz), 7.6.
  - 8.3 (23–30.5 engineers; the Infrastructure, Substrate and IPC, and JS/bindings/DOM tracks), RES-4, D-S4, D-S10, layers L1–L3.
  - All 42 crate names in the tier table exist in Stage 1.
- **HGBA-2 (rev 2.1.6):** M-1 and "M-1 part 4", P2, P6, 4.8, 5.4 (`PolicyDenied` indistinguishable from `Unsupported`), S-4c (including "release V8 misses it"), S-4e, I-1, I-2, P-1, ANC-1.
- **HGBA-3:** B1 ("retrofitting it is very hard"), B3, B5 (header only), DET-2, DST-1, ORC-1 (oracles before code), INC-3 (counting allocator), Section 6.
- **Security scope 0.1:** SEC-7, SEC-11 (a) and (b), SEC-12, SEC-14 (a)–(c), SEC-27, SEC-28 (exact quote, 14 days), IPC-R1–R5 (R15 is the last), 6.8 typed `External` wrapper.
- **Repo files:**
  - `tools/checks.sh`: check list, modes, AI `review`, SAFETY `awk` heuristic, zizmor, cargo-deny advisories only.
  - Clippy runs nowhere: no match in `tools/`, `.githooks/` or `.github/workflows/`.
  - `tools/deny.toml`: `[graph]`/`[advisories]` exactly as A.4 says.
  - Fork URL `https://github.com/Rorqualx/rusty_v8-izcal`.
  - CONTRIBUTING: the another-browser dependency rule.
  - No `CS-` prefix collision in `reports/`.
- **Research notes:** every A.8 mapping is accurate (all 85 note rules), including ER-16→CS-ER-18, ER-18→CS-ER-16, TY-12→CS-PF-1, TY-15→CS-ST-13 and A12→CS-TY-9.

### 3.3 Silent changes from the notes

The standard changed some note levels without saying so. CI behaves the same (`-D warnings`); a local `cargo check` differs.
- From deny to warn: `unwrap_used` in tier R (ER-5), `unused_result_ok` (ER-11), `fallible_impl_from` (TY-8), `mod_module_files` and `missing_docs` (S12, S13).
- PF-2's "deny the perf group" became "stays on".
- A12's `future_not_send` was dropped.
- S2's "no serde derive or `syn` in L1–L3" became "no proc-macro deps", which is wider.

These are legitimate choices, but 6.3 or A.8 should record them.

## 4. Internal consistency

| Check | Result |
|---|---|
| Rule count and IDs | 100 rule rows: ER 18, TY 13, V8 8, AS 15, ST 16, TE 18, PF 12. No duplicate IDs; every CS ID referenced anywhere is defined. 12 targets (CS-T1–T12) and 7 decisions (CS-D1–D7), all defined and all referenced IDs resolve |
| Intro "100 rules, CS-ER-1 to CS-PF-12" | Consistent |
| CS-T3 "names a mechanism for 90 of 100" | Consistent with its own list (8 review-only plus 2 partial). Two more should be counted as non-gating: CS-AS-7 ("reports only, until calibrated") and CS-ST-3 (a T1 timings *report*). Also, "running in T0 or T1" is mostly prospective: the gen, graph, hook and canary mechanisms do not exist yet, so today's measured value is about 0%. State CS-T3 as a Phase 1/2 target |
| Mechanisms that do not enforce their rule | CS-ER-17 (`ffi_unwind_calls`), CS-AS-4 (`async_fn_in_trait`), CS-ER-14 (`expect_fun_call`), CS-T1/CS-ST-13 (rustdoc lints with no doc job), CS-ER-2 (tier O exempt through the G3 expectation). See corrections 2 and 4–6 |
| A.1 lints not traced to any rule | `mem_forget`, `exit`, `wildcard_imports`, `unused_async`, `wildcard_dependencies` sit under group comments but no rule cites them. Add them to a rule, or drop them, so every lint has a rule ID (6.13's own principle) |
| † markings | 32 † uses. Four mark numbers this document did not set, which contradicts Section 8's definition (set "by this document from first principles"): "23–30.5 engineers†" (Stage 1 8.3), "7-day†" cooldown (SEC-14, twice), "SEC-28's 14 days†" and DET-2's "≤ 1%†" in 6.11. Several of the document's own thresholds lack †: CS-PF-5 "triaged within 3 days" (now also unsourced), CS-TE-7 "over 7 days", CS-T4 "rolling 30 days", CS-TE-14 "preemption bound 3" (loom-sourced, so fine) |
| 11.2 † list vs body | The list omits 70%, 256 cases, 10k, 100 seeds, 90%, 50%, 2% (CS-D3), 2 business days and the engineer-weeks. Say "every † value", or complete the list |
| Section references | All "Section N.N" and "Appendix A.N" references resolve; 167 links, all absolute `https`, none relative |
| Header "Relates to" | The body also cites SEC-12, SEC-27, HGBA-2 S-4e, I-1, I-2, 5.4, P-1, ANC-1, HGBA-3 B2, Stage 1 13.3 RES-3, D-S4 and others not listed. SEC-3 and SEC-4 are listed but used nowhere in the body |
| 11.2 flagged claims now settled | Firefox panic annotations: confirmed (bugs 1275780, 1507465). Firefox thresholds: confirmed. Quantum DOM: the causation remains inference, and the "never enabled" wording needs the 2.2 #31 fix. Stylo pool: confirmed from source. arrayref: confirmed from the Rust blog, with corrections. gin second check: confirmed by direct fetch |
| CS-D5 vs SEC-27 | Consistent. A.5 puts `perf-gate` on GitHub-hosted runners |
| A.2 tier S/T: "Everything else stays as in the base, including the disallowed lists" vs 6.2 "Exempt … by crate-level expectation" | Consistent in intent; the mechanism breaks (correction 1) |
