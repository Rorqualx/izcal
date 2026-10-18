# CS-D3 early spike: TokenCell / ScriptToken (18 October 2026)

**Code:** `m0/token-cell-spike/` (builds against the fork archives; `run_all.sh` reruns everything)
- `token-cell/` is V8-free and Miri-able: `src/token.rs` (ScriptToken, `enter`/`lend`/`reenter`), `src/cell.rs` (TokenCell, `borrow_mut_2/3`), `src/dom.rs` (DOM model, `insert_before`), `src/bin/bench.rs`, and `tests/ui/` (12 compile-fail tests).
- `token-cell-v8/` builds against the fork: `src/hg_script.rs` (token-taking wrappers, callback prologue, `binding!`), `src/member_cell.rs` (criterion 3 sketch), `src/holes.rs`, `src/misuse.rs`, `clippy.toml` (59 disallowed methods), `tests/v8_reentry.rs`, and `tests/ui/` (3 compile-fail tests).
- `audit/` holds the entry-point audit scripts and outputs. `logs/` holds every run log. `run_all.sh core|miri|v8|clippy|bench` reruns everything.

**Environment:** fork rusty_v8 152.2.0 (`hgba`) linked from the prebuilt sandbox release archive (`scripts/consume-release.sh`), so V8 was not compiled. rustc 1.99.1 stable; nightly 1.101.0 (2026-10-17) for Miri; Apple M6, macOS 27.0.1. Rig `local` was booked as heavy (b-bad408, 15:21–15:24) and released after the benchmark.

## Decision on the two-token question: the token is a separate type, not the scope

**The scope cannot be the token. This is shown, not argued.** In the fork, every method that runs script takes `&PinScope`, which is a *shared* borrow (`Function::call`, `Script::run`, `Object::get` and the rest). Every V8 callback also receives a *fresh* `&mut PinScope`, which the trampoline builds from `CallbackScope::new(info)` (`function.rs:520`).

Suppose a cell's borrows were tied to the scope. A `&T` would then survive a `call(&scope)`, and the callback could take `&mut T` of the same cell. Both of these compile, and the test `scope_as_token_lets_callback_mutate_under_live_borrow` passes: the outer `&u64` reads 1 before the call and 42 after it.

**Side finding: upstream rusty_v8 `cppgc::GcCell` has the same hole.** It ties `get` to `&Isolate` and `get_mut` to `&mut Isolate`, and both scopes deref to the isolate. Safe code mutates a value under a live shared borrow (test `upstream_gc_cell_has_the_same_hole`: (1, 42)). DOM crates should disallow `GcCell` (`disallowed-types`). It is worth reporting upstream.

**Chosen design:** `ScriptToken` is a ZST with a private field, `!Send`, `!Sync` and not `Clone`. It is unique per thread through a three-state thread-local: `Idle`, then `Held`, then `Lent`. It is only ever handed out as a higher-ranked `&'t mut ScriptToken` inside a closure, so no guard exists that `Box::leak` could keep alive.

| Function | What it does |
|---|---|
| `hg_script::enter(&mut PinScope, ..)` | Mints the token at the top of a task; panics unless the state is `Idle` |
| `lend(&mut tok, f)` | Wraps every V8 entry point and sets the state to `Lent`; `f` gets no token |
| `prologue` / `try_reenter` | The generated callback prologue; mints a token only when the state is `Lent`, otherwise throws a TypeError into JS |

## Criteria

### (1) Token derivable only from `&mut HandleScope` or a callback prologue — PASS, with a runtime component

The borrow checker covers the outer frame. Trybuild compile-fail tests, all passing (the `.stderr` files are in `tests/ui`):

| Test | Error |
|---|---|
| `c1_borrow_mut_across_lend` | E0499 |
| `c1_shared_borrow_across_lend` | E0502 |
| `c1_callback_captures_outer_borrow` | E0499 |
| `c1_callback_captures_outer_token` | E0500/E0501 |
| `c1_mint_token_by_struct` | E0451, private field |
| `c1_token_not_clone` | E0599 |
| `c1_token_escapes_frame` | E0521 |
| `c1_token_not_send` | E0277 |
| `c1_cell_not_sync` | E0277 |
| V8: `v1_borrow_mut_across_call` (`hg_script::call`) | E0499 |
| V8: `v1_shared_borrow_across_run` | E0502 |

The inner frame cannot be proven at compile time. The link from `lend` to the callback runs through C++, so "a nested callback cannot mint a second token while the outer one is lent" rests on the state check. Tests that cover it:
- V8-free: `reenter_needs_lend`, `nested_enter_panics`, `deep_lend_reenter_chain` (64 levels), `state_restored_on_unwind`, `tokens_are_per_thread`.
- V8: `unwrapped_script_call_is_refused`. A raw `Function::call` without `lend` makes the binding throw `re-entrancy violation: script ran without a lent ScriptToken`, and the DOM is unchanged.
- V8: `callback_outside_any_task_is_refused`.

**Caveat:** "derivable only from `&mut HandleScope`" is weaker than it sounds, because every callback also has a `&mut PinScope`. A binding body could call `hg_script::enter(scope, ..)`, which compiles and is refused only at runtime (panic, which aborts inside a callback). To make that a compile error, `enter` must be visible only to the event-loop crate. This spike did not build that.

### (2) Every script-capable entry point behind token wrappers via `disallowed-methods` — PASS for the mechanism; the list needs an M2 review

**How the audit was done.** V8 marks every API function that may run script with `EnterV8Scope` or `PrepareForExecutionScope`, as opposed to `EnterV8NoScriptScope`. The script `audit/v8scopes.py` found 75 such functions in the fork's `v8/src/api/api.cc`. `audit/map.py` then mapped them, with ast-grep, to the 70 rusty_v8 wrappers that call them (`audit/rust_entry.txt`). I added the runners outside `api.cc` and the transitive wrappers.

**Disallowed in `clippy.toml` (59 entries, Tier A: can reach user JS):**
- Calls: `Function::{call, call_with_context, new_instance}`; `Object::{call_as_function, call_as_constructor}` and their `_with_context` forms
- Scripts and modules: `Script::run`; `Module::{evaluate, evaluate_for_import_defer, instantiate_module, instantiate_module2}`. Instantiation reaches host callbacks rather than JS.
- Property access (getters, setters, proxy traps): `Object::{get, get_with_receiver, get_index, set, set_with_receiver, set_index, create_data_property, define_own_property, define_property, has, has_index, has_own_property, delete, delete_index, get_property_names, get_own_property_names, get_prototype, set_prototype, get_property_attributes, get_own_property_descriptor, get_real_named_property, set_integrity_level}`
- Conversions (`valueOf`/`toString`): `Value::{to_string, to_number, to_big_int, to_integer, to_int32, to_uint32, number_value, integer_value, int32_value, uint32_value, instance_of}`
- Transitive: `Value::to_rust_string_lossy`, which calls `to_string` inside rusty_v8
- Other user-JS paths:
  - `json::stringify` (`toJSON`, getters)
  - `Promise::{then, then2, catch}` (species constructor)
  - `PromiseResolver::resolve` (the thenable's `then` getter)
  - `RegExp::exec` (V8 fetches `exec`, per its own TODO)
  - `ValueSerializer::write_value`, `ValueDeserializer::read_value`
  - `TryCatch::stack_trace` (as `PinnedRef::stack_trace`)
- Runners outside `api.cc`:
  - `Isolate::perform_microtask_checkpoint`, `MicrotaskQueue::perform_checkpoint`
  - `Platform::pump_message_loop` (posted tasks, e.g. FinalizationRegistry cleanup)
  - `inspector::V8InspectorSession::dispatch_protocol_message` (`Runtime.evaluate`)

**Tier B, marked by V8 as may-run but cannot reach user JS (not disallowed):** `ScriptCompiler::compile*`, `JSON::parse` (no reviver), `Value::to_object`, `Map`/`Set` operations (original builtins), `Date::new`, `Proxy::new`, `RegExp::new`, `Array::new*`, `ObjectTemplate::new_instance`, `FunctionTemplate::get_function`, `PromiseResolver::{new, reject}`. These need an M2 decision.

**Enforcement, verified** (`logs/clippy_misuse.log`): clippy flagged 7 of the 7 raw calls in `src/misuse.rs`:
- method syntax through `Local`'s Deref
- UFCS `Function::call(&f, ..)`
- `Function::call` taken as a value
- `Object::get`
- the transitive `to_rust_string_lossy`
- `perform_microtask_checkpoint` through `PinScope`'s DerefMut
- `Script::run`

The call through the `hg_script` wrapper was not flagged, and the clean build has 0 warnings. Clippy does report unresolvable config paths: a bogus probe entry gave "does not refer to a reachable function". All 59 real entries therefore resolve.

**Limits:**
- Clippy cannot see calls made inside rusty_v8 itself, so the audit has to be transitive and repeated on every fork bump. A generated diff against `api.cc` would do it.
- The api.cc regex may miss functions with unusual signatures. The `api-natives`/`api-arguments` files were not scanned.
- `V8::GetSharedMemoryStatistics` was a regex false positive.
- Every other callback kind (accessors, interceptors, host hooks such as import, prepareStackTrace and promise-reject, interrupt callbacks) needs the same generated prologue.
- GC-time callbacks (V8 weak callbacks, GC prologue/epilogue, near-heap-limit) must use a no-token prologue. When GC is triggered from JS the state is `Lent`, so a script prologue would mint a token in the middle of GC.
- **Not gated in this spike: allocation, which can trigger GC.** I treated GC as safe without the token, given criterion (3) and CS-V8-9. CS-TY-10 as written says GC triggers take `&mut ScriptToken`. Applied to every `v8::String::new`, that would forbid holding any borrow while creating a JS value — a large ergonomic cost. This needs an explicit decision.

### (3) Traced `Member` fields kept out of TokenCell — PASS (reasoning, a sketch, and one GC test)

**Design.** In `member_cell.rs`, each link is a `MemberCell<T>` (an `UnsafeCell<Member<T>>`) that is written through `&self` and never lends out a reference into the slot. `get` returns a stack `UnsafePtr` to the pointee. No token is needed, because no Rust borrow of the slot can be alive. In C++, `Member` assignment is an atomic store plus a write barrier (`member.h`: `SetRawAtomic` + `AssigningWriteBarrier`). A `DomNode` traces only its four `MemberCell`s; its `TokenCell<NodeData>` is untraced and holds no GC pointers.

**Test.** `member_cell_links_keep_nodes_alive_under_gc` runs on the fork's cppgc with a detached heap. Linked nodes survive GC (0 drops). After `next_sibling.clear()` the unlinked node is collected (1 drop), and the rest go once the root is dropped (3 drops). The TokenCell data was readable with a token after GC.

**Gap for M2.** The fork's `Member::set` takes `&mut self`, so `MemberCell::set` forms a `&mut Member` that the concurrent marker may read from another thread. The actual accesses are C++ atomics, but in Rust's model it is a `&mut` aliased by another thread. Add a fork API that assigns through a raw pointer or `&self`.

**Not built:** the generator check that rejects `TokenCell<T>` when `T` contains a `Member`. A `NoTraced` marker or a derive is the obvious route. This test also did not exercise concurrent marking.

### (4) Multi-node mutation pattern — PASS

`Document::insert_before` in `dom.rs` uses all three CS-D3 patterns:
- clone-out handles (`Rc<Node>`, which stands in for GC handles)
- a per-node split for the old parent
- a scoped multi-borrow for parent, child and reference: `borrow_mut_3`, which panics on overlapping cells, including a cell nested inside another and a ZST
- after reactions run, a re-borrow, because state read earlier may be stale

Reactions get `&mut ScriptToken`; the DOM crate never lends, only `hg_script` does.

**V8-free tests** (`dom::tests`):
- links and counts correct
- an observer re-enters and mutates
- an observer re-enters `insertBefore` recursively
- an observer moves the outer call's reference node

**V8 tests with real JS:**
- `js_observer_reenters_native_dom`: Rust `insertBefore` → JS observer → native `setText`
- `js_observer_nested_insert_before`: JS observer → native `insertBefore` → JS observer again → native `setText`, two levels through V8; delivery depth 2 and order `[a, y, x, b]` verified
- `microtask_reentry_through_checkpoint_wrapper`: with the policy set to Explicit, a Promise job runs `setText` only inside `hg_script::microtask_checkpoint`

**Misuse does not compile:** `c4_multi_borrow_across_lend` (E0499), `c4_insert_before_holds_node_data` (E0499), `c4_text_ref_across_reaction` (E0502), and V8 `v4_binding_holds_borrow_across_reaction` (E0499).

**Ergonomics.** It is acceptable. `insert_before` needed one `borrow_mut_3` and no contortions. The real cost is that `tok` is threaded through every DOM and binding signature. Argument conversion in bindings needs the token too (`int32`, `to_rust_string`), because `valueOf` can run script. Not tested: a custom-element reaction queue as such (the sync observer stands in for it), and a real `MutationObserver` with record batching.

### (5) Miri on the cell API — PASS. loom: not run

`cargo +nightly miri test --lib` passed 20/20 tests under Stacked Borrows with `-Zmiri-strict-provenance` (`logs/miri.log`) and 20/20 under `-Zmiri-tree-borrows` (`logs/miri_tree_borrows.log`), with 0 Undefined Behavior reports.

The tests cover:
- borrows and mutation, replace/set/drop
- unsized slices, ZSTs, overlap panics (same cell, nested cell, ZST)
- the lend/reenter state machine, including unwinding and per-thread tokens
- all DOM `insert_before` re-entry scenarios

The same 20 pass natively, plus the 12 compile-fail tests.

**Loom was not run.** The token state is thread-local and TokenCell is `!Sync`, so the cell has no concurrent interleavings for loom to explore. Loom would matter for `MemberCell` against the marker, and Miri/loom cannot model that because it is C++.

## Cost proxy (release, Apple M6, 3 interleaved rounds, each 15 reps × 200 passes × 16,384 nodes)

The loop is a borrow-heavy DOM-like workload in shuffled pointer-chasing order. Per node it does a mutable borrow of the parent, shared borrows of the node and a sibling, and a mutable borrow of the node. Instruction counts are whole-process `/usr/bin/time -l` figures, identical in setup across modes.

| Mode | Median ns/node (3 rounds) | Min ns/node | Instructions retired | Cycles |
|---|---|---|---|---|
| TokenCell | 2.490 / 2.319 / 2.317 | 2.304 | 1.400–1.414 G | 0.53–0.59 G |
| RefCell | 3.224 / 3.228 / 3.242 | 3.207 | 2.334 G | 0.73–0.74 G |

**Result:** TokenCell is about 28% faster in wall time and runs about 40% fewer instructions. RefCell's flag checks are pure overhead that TokenCell does not have, since it compiles to plain loads and stores.

**Cost of a lend + reenter pair** (paid on every script call and callback), measured with 49.2 M pairs per mode:

| Mode | Median ns/op | Instructions |
|---|---|---|
| `lend` (pair around a write) | 0.68–0.69 | 458.4 M |
| `nolend` (the write alone) | 0.876 (min 0.218, bimodal) | 360.0 M |

That is about 2 extra instructions per pair: one TLS cell, mostly folded by the compiler. Against a V8 call of ~50–100 ns or more, this is noise.

This is a proxy. It says (a) cannot cost more than RefCell on borrow-heavy code; it says nothing about the 2% M2 bindings threshold, which depends on the real generated bindings.

## Not tested

- The binding generator itself (the `binding!` macro stands in).
- Restricting `enter`'s visibility.
- GC-gated allocation (the open design point in criterion 2).
- Concurrent marking against `MemberCell`.
- Accessor, interceptor and host-hook prologues.
- The Tier B review.
- Debug-V8 runs: only the release archive was used.
- Linux and Windows.
- An M2 bindings benchmark.

## Conclusion

Option (a) looks viable. No criterion failed:
- Re-entrant misuse is a compile error in every shape tried: 15 compile-fail tests.
- The multi-node `insertBefore` with JS re-entry works through the real fork's V8.
- Miri is clean under both aliasing models.
- On the proxy it is faster than RefCell, not slower.

The spike also settled the two-token question: the scope cannot be the token. The fork's script calls take `&PinScope` and callbacks receive a fresh `&mut PinScope`, so a scope-tied cell is unsound in safe code. Upstream `GcCell` has exactly this hole, demonstrated in a test.

**The biggest risk is criterion (2).** Soundness of the inner half depends on every path into V8 going through a token-taking wrapper. The mechanism works: clippy caught 7/7 misuse shapes and resolved all 59 paths. What is fragile is the list:
- It is a hand-curated list, from a regex audit of `api.cc`, plus transitive wrappers clippy cannot see.
- Every callback kind needs the right prologue (script versus GC-time).
- There is an unresolved conflict with CS-TY-10's wording that GC triggers take the token, which would be ergonomically heavy if applied to allocation.

The runtime prologue check (a TypeError instead of a second token) turns a missed path into a contained error, not unsoundness — provided the generator never gives GC-time callbacks a minting prologue. At M2 the list should be regenerated from `api.cc` mechanically and the allocation/GC question decided.

## Review notes (added when committing)

- The two "hole" tests (`scope_as_token_lets_callback_mutate_under_live_borrow`, `upstream_gc_cell_has_the_same_hole`) demonstrate the hole by observing a write through `&mut` from a live `&u64`. That observation is itself undefined behaviour, so it is only reliable in debug builds; an optimised build may legally return the cached value. The tests show that the code compiles in safe Rust, which is the finding; their runtime assertion is not evidence.
- `MemberCell::set` forms a `&mut Member` that the concurrent marker may read. Its SAFETY argument covers the mutator thread only. This is the fork gap recorded for criterion (3) (`Member::set` needs a `&self` or raw-pointer assign) and must be fixed before M2 code relies on it.
