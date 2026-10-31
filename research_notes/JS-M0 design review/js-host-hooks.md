# JS-M0 design review: the `hg-js-host` hook table

*Part 1 of the JS-M0 design review for Izcal's JavaScript engine. Date: 31 October 2026. Status: draft 0.1, for owner review; not yet fact-checked or critiqued. Relates to: [JS spec 0.4.5](../../reports/Izcal%20JavaScript%20engine.md) 4.2, 6.5 (weak references), 6.9 (determinism), 6.14 (modules and host hooks), 6.15 (G6, G13), 9.2 (critical path); [Stage 1 design](../../reports/HGBA%20Stage%201%20architecture%20design.md) 3.3 ("the hook table is audited at JS-M0"); the substrate traits in `crates/hg-substrate/src/` and the `DecisionLog` on branch `substrate/js-m0-decisions`; [specs and semantics note](../Izcal%20JavaScript%20engine/specs_semantics.md) 1.6 and 8. Index and decision list: [README](README.md).*

*Revision 0.1 (31 October 2026): first draft.*

This note is the audit Stage 1 3.3 asks for at JS-M0, and it freezes the `hg-js-host` trait early, as the JS spec's critical path needs (9.2: the DOM and bindings at M2 need a running engine, and `hg-script`, `hg-bindings` and the DOM-less embedders all implement this trait). **The current list is 16 host-defined abstract operations in ECMA-262** (the ES2027 draft at [tc39.es/ecma262](https://tc39.es/ecma262/) on 31 October 2026), **plus HostSystemUTCEpochNanoseconds from Temporal** (Stage 4; its merge, [PR #3966](https://github.com/tc39/ecma262/pull/3966), was still open on 29 October 2026) **plus HostGetCodeForEval from Dynamic Code Brand Checks** (Stage 3), which JS spec 6.14 already adopts for Trusted Types and which also changes HostEnsureCanCompileStrings's signature. The trait therefore has **18 hook methods**. The design rules: the host never holds a GC reference, only engine-owned IDs and handles; the spec's requirements on a hook are enforced by its Rust types where possible; each hook's token requirements say whether it may allocate or run script; and each hook has a determinism class under JS spec 6.9.

## 1. How the list was established

- **Source of truth.** ECMA-262 marks each hook as a clause of type `host-defined abstract operation`. In the current draft there are exactly 16 such clauses (counted in the rendered [spec](https://tc39.es/ecma262/) on 31 October 2026, the same method as [SP 1.6](../Izcal%20JavaScript%20engine/specs_semantics.md)). The same draft has 5 clauses of type `implementation-defined abstract operation`, all about time zones and list formatting (Section 5).
- **Proposals the engine implements.** The [Temporal specification](https://tc39.es/proposal-temporal/#sec-hostsystemutcepochnanoseconds) adds HostSystemUTCEpochNanoseconds and routes `Date.now` and `Date()` through it ([Date.now](https://tc39.es/proposal-temporal/#sec-temporal-date.now)). The [Dynamic Code Brand Checks specification](https://tc39.es/proposal-dynamic-code-brand-checks/#sec-hostgetcodeforeval) adds HostGetCodeForEval and gives [HostEnsureCanCompileStrings](https://tc39.es/proposal-dynamic-code-brand-checks/#sec-hostensurecancompilestrings) eight parameters. Both proposals mark their hooks host-defined.
- **What HTML implements.** HTML defines 14 of the 18 in [8.1.6](https://html.spec.whatwg.org/multipage/webappapis.html#javascript-specification-host-hooks), HostGetCodeForEval and HostSystemUTCEpochNanoseconds included ([HTML](https://html.spec.whatwg.org/multipage/webappapis.html#hostsystemutcepochnanoseconds)). HostFinalizeImportMeta, HostHasSourceTextAvailable, HostResizeArrayBuffer and HostGrowSharedArrayBuffer take their defaults. HTML's prose calls these hooks "implementation-defined"; ECMA-262 types them host-defined, and this note follows ECMA-262.
- **Annex D is not the list.** ECMA-262's informative [Annex D.1](https://tc39.es/ecma262/#sec-host-hooks-summary) lists InitializeHostDefinedRealm, which is an ordinary abstract operation, and omits HostEnsureCanAddPrivateElement, which is a host-defined one. The clause types are normative and Annex D is informative, so this audit counts clause types. Annex D's other layering points (D.2–D.6) are covered in Section 4.
- **Repeatable.** A CI test in `hg-js-host` parses the pinned `spec.html` (and the pinned proposal texts) for clauses of type `host-defined abstract operation` and fails if one has no trait method whose documentation names its clause id. The audit is thereby rerun at every specification pin (ANC-1), not only at JS-M0 (R0-H10).

## 2. Determinism classes (R0-H8)

JS spec 6.9's rule is that everything JS can observe is a function of B1's recorded inputs and the build. Each hook falls into one or more of these classes, which say how it stays inside that rule:

| Class | Meaning | How replay reproduces it |
|---|---|---|
| **P** pure | The result is a function of the arguments and of configuration fixed for the agent's life | Nothing to record |
| **Q** queued | Appends to a host FIFO drained at a point the host's algorithm fixes (HTML's microtask checkpoint), with no scheduler choice | Nothing to record |
| **T** task | Posts a task; which ready task runs next is a scheduler choice | The seeded scheduler in simulation; the recorded scheduler choices in production recordings (Stage 1 3.3) |
| **I** input | Reads a B1 input (a clock, network bytes, policy that arrived in network bytes) | The recorded input |
| **D** decision | Is invoked at a JS-D15 decision point, whose canonical position `DecisionLog` records | The logged position (`DecisionKind::GC_COMPLETE` and the others) |
| **U** user code | May run page script; deterministic given the other classes, but a re-entry point | Nothing beyond the other classes; the effect system marks it (`call_user`) |

## 3. The hook table (R0-H1)

**Contexts.** Names of helper types are illustrative; the method set, the parameters, the return types and the token requirements are frozen (R0-H4).

| Context | What it holds | What it allows |
|---|---|---|
| none (`&self` or `&mut self`) | — | No heap access at all: plain data in, plain data out |
| `&HostCx<'h, '_>` | A shared borrow of the agent's `Mutator<'h>` | Read the heap; hold handles in engine root tables; **no GC point** |
| `&mut HostCx<'h, 's>` | `&mut Mutator<'h>` and a root scope | Allocate (a GC point); **no script** |
| `&mut ScriptCx<'h, 's>` | `&mut Mutator<'h>`, `&mut ScriptToken` and a root scope | Allocate and **run script** (CS-TY-10, JS spec 6.5) |

**Identifiers, not references (R0-H2).** Jobs, cleanup jobs, module-load payloads, realms, modules and buffers are passed as generational IDs into engine-owned per-agent tables (`JobId`, `CleanupJobId`, `LoadPayloadId`, `RealmId`, `ModuleId`, `BufferId`). The host later hands the ID back to an engine entry point (`agent.run_job(cx, id)` and the like). A `[[HostDefined]]` field holds `HostData`, a small `Copy` value of host IDs (for HTML, a settings-object ID and a script ID), never a GC reference. A `[[HostDefined]]` field lives in the cage, so it is untrusted when read back (H-2): the host re-checks the IDs' generations. Where the host must keep a JS object across tasks (HTML keeps rejected promises), it asks for a **held handle** from an engine root table, strong or weak, which is an index with a generation, not a pointer.

The table, in ECMA-262 clause order. "Calls" lists where ECMA-262 invokes the hook. The substrate traits are those of `hg_substrate` (`Clock`, `Entropy`, `Executor`, `Ipc`, and `DecisionLog` from the `substrate/js-m0-decisions` branch); a hook reaches them through `hg-script`'s implementation, never from engine code (G13).

| # | Hook (anchor) | Calls | HTML | Rust method | Context | Substrate reached | Class |
|---|---|---|---|---|---|---|---|
| 1 | [HostEnsureCanAddPrivateElement](https://tc39.es/ecma262/#sec-hostensurecanaddprivateelement) (7.3.29) | PrivateFieldAdd, PrivateMethodOrAccessorAdd | [Throws a TypeError for WindowProxy and Location](https://html.spec.whatwg.org/multipage/webappapis.html#the-hostensurecanaddprivateelement-implementation) | `fn ensure_can_add_private_element(&self, object: HostObjectBrand) -> Result<(), HostError>` | none | — | P |
| 2 | [HostMakeJobCallback](https://tc39.es/ecma262/#sec-hostmakejobcallback) (9.5.2) | PerformPromiseThen, CreateResolvingFunctions, the FinalizationRegistry constructor | [Saves the incumbent settings object and the active script](https://html.spec.whatwg.org/multipage/webappapis.html#hostmakejobcallback) | `fn make_job_callback(&mut self, cx: &HostCx<'h, '_>, callback: FunctionInfo) -> HostData` | `&HostCx` | — | P |
| 3 | [HostCallJobCallback](https://tc39.es/ecma262/#sec-hostcalljobcallback) (9.5.3) | NewPromiseReactionJob, NewPromiseResolveThenableJob, CleanupFinalizationRegistry | [Prepares, calls, cleans up](https://html.spec.whatwg.org/multipage/webappapis.html#hostcalljobcallback) | `fn call_job_callback<'s>(&mut self, cx: &mut ScriptCx<'h, 's>, host_defined: HostData, call: JobCall<'h, 's>) -> Completion<'h, 's>` | `&mut ScriptCx` | — | U |
| 4 | [HostEnqueueGenericJob](https://tc39.es/ecma262/#sec-hostenqueuegenericjob) (9.5.4) | EnqueueResolveInAgentJob (`Atomics.waitAsync`) | [Queues a global task](https://html.spec.whatwg.org/multipage/webappapis.html#hostenqueuegenericjob) | `fn enqueue_generic_job(&mut self, job: JobId, realm: RealmId)` | none | `Executor` | T |
| 5 | [HostEnqueuePromiseJob](https://tc39.es/ecma262/#sec-hostenqueuepromisejob) (9.5.5) | PerformPromiseThen, TriggerPromiseReactions, CreateResolvingFunctions | [Queues a microtask](https://html.spec.whatwg.org/multipage/webappapis.html#hostenqueuepromisejob) | `fn enqueue_promise_job(&mut self, job: JobId, realm: Option<RealmId>)` | none | — (the microtask queue is `hg-script`'s memory) | Q |
| 6 | [HostEnqueueTimeoutJob](https://tc39.es/ecma262/#sec-hostenqueuetimeoutjob) (9.5.6) | EnqueueAtomicsWaitAsyncTimeoutJob | [Runs steps after a timeout, then queues a global task](https://html.spec.whatwg.org/multipage/webappapis.html#hostenqueuetimeoutjob) | `fn enqueue_timeout_job(&mut self, job: JobId, realm: RealmId, milliseconds: f64)` | none | `Clock::monotonic`, `Executor::post` with a delay | T, I |
| 7 | [HostEnqueueFinalizationRegistryCleanupJob](https://tc39.es/ecma262/#sec-host-cleanup-finalization-registry) (9.9.4.1) | [Execution](https://tc39.es/ecma262/#sec-weakref-execution) (weak clearing) | [Queues a global task that runs CleanupFinalizationRegistry](https://html.spec.whatwg.org/multipage/webappapis.html#hostenqueuefinalizationregistrycleanupjob) | `fn enqueue_cleanup_job(&mut self, job: CleanupJobId, realm: RealmId)` | none | `Executor`; called at a decision point that `hg-heap` logs in `DecisionLog` | D, T |
| 8 | [HostGetImportMetaProperties](https://tc39.es/ecma262/#sec-hostgetimportmetaproperties) (13.3.12.1.1) | `import.meta` evaluation | [`url` and `resolve`](https://html.spec.whatwg.org/multipage/webappapis.html#hostgetimportmetaproperties) | `fn get_import_meta_properties<'s>(&mut self, cx: &mut HostCx<'h, 's>, module: ModuleId) -> Vec<(PropertyKey<'h, 's>, Root<'h, 's, Value>)>` | `&mut HostCx` | — | P |
| 9 | [HostFinalizeImportMeta](https://tc39.es/ecma262/#sec-hostfinalizeimportmeta) (13.3.12.1.2) | `import.meta` evaluation | Default | `fn finalize_import_meta<'s>(&mut self, cx: &mut HostCx<'h, 's>, import_meta: Root<'h, 's, Object>, module: ModuleId)` | `&mut HostCx` | — | P |
| 10 | [HostLoadImportedModule](https://tc39.es/ecma262/#sec-HostLoadImportedModule) (16.2.1.10) | LoadRequestedModules, InnerModuleLoading, ContinueModuleLoading, EvaluateImportCall | [Fetches, with import maps](https://html.spec.whatwg.org/multipage/webappapis.html#hostloadimportedmodule) | `fn load_imported_module(&mut self, cx: &mut HostCx<'h, '_>, referrer: Referrer, request: &ModuleRequest<'_>, host_defined: HostData, payload: LoadPayloadId)`; the host completes with `agent.finish_loading_imported_module(cx, payload, result)` | `&mut HostCx` | `Ipc` (the network service), `Executor` | I, T |
| 11 | [HostGetSupportedImportAttributes](https://tc39.es/ecma262/#sec-hostgetsupportedimportattributes) (16.2.1.12.1) | AllImportAttributesSupported | [«"type"»](https://html.spec.whatwg.org/multipage/webappapis.html#hostgetsupportedimportattributes) | `fn supported_import_attributes(&self) -> &'static [&'static str]` | none | — | P |
| 12 | [HostEnsureCanCompileStrings](https://tc39.es/ecma262/#sec-hostensurecancompilestrings) (19.2.1.2), in the [Dynamic Code Brand Checks form](https://tc39.es/proposal-dynamic-code-brand-checks/#sec-hostensurecancompilestrings) | PerformEval, CreateDynamicFunction | [CSP's EnsureCSPDoesNotBlockStringCompilation](https://html.spec.whatwg.org/multipage/webappapis.html#javascript-specification-host-hooks), which may run a Trusted Types default policy | `fn ensure_can_compile_strings<'s>(&mut self, cx: &mut ScriptCx<'h, 's>, request: CompileStringsRequest<'h, 's>) -> Result<(), Throw<'h, 's>>`, where the request carries the callee realm, parameter strings, body string, `direct`, code string, compilation type, parameter arguments and body argument | `&mut ScriptCx` | — | I (policy from headers and markup), U |
| 13 | [HostGetCodeForEval](https://tc39.es/proposal-dynamic-code-brand-checks/#sec-hostgetcodeforeval) (Dynamic Code Brand Checks) | PerformEval (in the proposal) | [The TrustedScript's data, or no-code](https://html.spec.whatwg.org/multipage/webappapis.html#javascript-specification-host-hooks) | `fn get_code_for_eval<'s>(&mut self, cx: &mut HostCx<'h, 's>, argument: Root<'h, 's, Object>) -> Option<Root<'h, 's, JsString>>` | `&mut HostCx` | — | P |
| 14 | [HostHasSourceTextAvailable](https://tc39.es/ecma262/#sec-hosthassourcetextavailable) (20.2.5) | `Function.prototype.toString` | Default | `fn has_source_text_available(&self, function: FunctionInfo) -> bool` | none | — | P (the spec requires it) |
| 15 | [HostResizeArrayBuffer](https://tc39.es/ecma262/#sec-hostresizearraybuffer) (25.1.3.8) | `ArrayBuffer.prototype.resize` | Default | `fn resize_array_buffer(&mut self, buffer: BufferId, new_byte_length: u64) -> Result<Handled, HostError>` | none | — (the engine grows in place through `Memory::commit`) | P |
| 16 | [HostGrowSharedArrayBuffer](https://tc39.es/ecma262/#sec-hostgrowsharedarraybuffer) (25.2.2.4) | `SharedArrayBuffer.prototype.grow` | Default | `fn grow_shared_array_buffer(&mut self, buffer: BufferId, new_byte_length: u64) -> Result<Handled, HostError>` | none | — (as 15) | P |
| 17 | [HostPromiseRejectionTracker](https://tc39.es/ecma262/#sec-host-promise-rejection-tracker) (27.5.1.9) | RejectPromise, PerformPromiseThen, EvaluateModuleSync | [Records rejections for `unhandledrejection`](https://html.spec.whatwg.org/multipage/webappapis.html#the-hostpromiserejectiontracker-implementation) | `fn promise_rejection_tracker(&mut self, cx: &HostCx<'h, '_>, promise: PromiseRef<'h, '_>, operation: RejectionOperation)` | `&HostCx` (may take a strong or weak held handle) | — | P (the later event is a T task of `hg-script`) |
| 18 | [HostSystemUTCEpochNanoseconds](https://tc39.es/proposal-temporal/#sec-hostsystemutcepochnanoseconds) (Temporal 2.3.1) | SystemUTCEpochNanoseconds, SystemUTCEpochMilliseconds (`Temporal.Now`, `Date.now`, `Date()`) | [The settings object's current wall time](https://html.spec.whatwg.org/multipage/webappapis.html#hostsystemutcepochnanoseconds) | `fn system_utc_epoch_nanoseconds(&mut self, realm: RealmId) -> i128` | none | `Clock::wall` | I |

**Notes on the rows.**

- **Row 1** is called by the engine only for host exotic objects, decided by a shape flag; for every other object the engine returns normally without calling it, which is exactly the spec's requirement "if obj is not a host-defined exotic object, this abstract operation must return NormalCompletion(unused) and perform no other steps". Taking `&self` and a brand (plain data) makes "any two calls with the same argument return the same kind of Completion Record" hold by construction.
- **Row 3.** The spec requires the hook to "perform and return the result of Call(jobCallback.[[Callback]], thisValue, argList)". `JobCall` is an engine-made value whose only method, `invoke(cx)`, performs that call and returns the only kind of `Completion` the method can return; the host cannot call anything else in its place, and `JobCall` is consumed by `invoke`. The host prepares and cleans up around it (HTML's incumbent handling).
- **Rows 4–7** take no heap context: they record an ID and return. They are called from inside engine operations that hold unrooted references, and they cannot disturb them.
- **Row 7's choice is pinned.** [Execution](https://tc39.es/ecma262/#sec-weakref-execution) makes "enqueueCleanup" an implementation-defined choice of true or false. Izcal always chooses true, so cleanup scheduling depends only on which cells a canonical cycle cleared (JS spec 6.5, 6.9). The hook is called while the GC-completion decision is applied, after `hg-heap` has logged it, so the host sees the same calls in the same order on replay.
- **Row 10.** The host may complete synchronously from inside the hook (the spec allows "either synchronously or asynchronously"); `&mut HostCx` lets it, because FinishLoadingImportedModule allocates but runs no script.
- **Row 11.** The spec requires the same list "with the same contents in the same order" every time; `&'static [&'static str]` makes any other implementation fail to compile.
- **Row 12.** HTML's definition of this hook lists seven parameters and omits `direct`; the proposal's has eight. The trait takes the proposal's eight, a superset, so nothing changes whichever text the other is aligned to.
- **Rows 15–16.** Izcal's engine grows buffers in place inside their reservations and checks the broker-granted limit in canonical units (JS spec 6.5, FP-D19). `hg-script` returns `Unhandled`, the default; a real commit failure crashes the renderer and is never JS-visible.
- **Row 18.** The result must lie between nsMinInstant and nsMaxInstant (±8.64 × 10^21 ns), which exceeds the 64-bit range, hence `i128`. `hg_types::SystemTime` holds signed 64-bit nanoseconds, so the substrate clock is always inside the range and the clamp never fires; HTML's coarsening of the current wall time happens in `hg-script` ([HTML](https://html.spec.whatwg.org/multipage/webappapis.html#hostsystemutcepochnanoseconds)).

### 3.1 The trait as frozen

```rust
/// What the engine asks of its embedder: one implementation per agent. Generic over
/// the agent heap's brand 'h (JS-D18). Each method's doc names its clause id; a CI
/// test checks the pinned specs' host-defined clauses against these ids (R0-H10).
pub trait JsHost<'h> {
    // ECMA-262 9.5: jobs
    fn make_job_callback(&mut self, cx: &HostCx<'h, '_>, callback: FunctionInfo) -> HostData {
        HostData::EMPTY
    }
    fn call_job_callback<'s>(&mut self, cx: &mut ScriptCx<'h, 's>, host_defined: HostData,
                             call: JobCall<'h, 's>) -> Completion<'h, 's> { call.invoke(cx) }
    fn enqueue_generic_job(&mut self, job: JobId, realm: RealmId);
    fn enqueue_promise_job(&mut self, job: JobId, realm: Option<RealmId>);
    fn enqueue_timeout_job(&mut self, job: JobId, realm: RealmId, milliseconds: f64);
    // ECMA-262 9.9.4
    fn enqueue_cleanup_job(&mut self, job: CleanupJobId, realm: RealmId);
    // ECMA-262 13.3.12 and 16.2: modules
    fn get_import_meta_properties<'s>(&mut self, cx: &mut HostCx<'h, 's>, module: ModuleId)
        -> Vec<(PropertyKey<'h, 's>, Root<'h, 's, Value>)> { Vec::new() }
    fn finalize_import_meta<'s>(&mut self, cx: &mut HostCx<'h, 's>,
                                import_meta: Root<'h, 's, Object>, module: ModuleId) {}
    fn load_imported_module(&mut self, cx: &mut HostCx<'h, '_>, referrer: Referrer,
                            request: &ModuleRequest<'_>, host_defined: HostData,
                            payload: LoadPayloadId);
    fn supported_import_attributes(&self) -> &'static [&'static str] { &[] }
    // ECMA-262 19.2.1.2 and Dynamic Code Brand Checks: dynamic code
    fn ensure_can_compile_strings<'s>(&mut self, cx: &mut ScriptCx<'h, 's>,
                                      request: CompileStringsRequest<'h, 's>)
        -> Result<(), Throw<'h, 's>> { Ok(()) }
    fn get_code_for_eval<'s>(&mut self, cx: &mut HostCx<'h, 's>, argument: Root<'h, 's, Object>)
        -> Option<Root<'h, 's, JsString>> { None }
    // ECMA-262 7.3.29 and 20.2.5: objects and functions
    fn ensure_can_add_private_element(&self, object: HostObjectBrand) -> Result<(), HostError> { Ok(()) }
    fn has_source_text_available(&self, function: FunctionInfo) -> bool { true }
    // ECMA-262 25.1.3.8 and 25.2.2.4: buffers
    fn resize_array_buffer(&mut self, buffer: BufferId, new_byte_length: u64)
        -> Result<Handled, HostError> { Ok(Handled::Unhandled) }
    fn grow_shared_array_buffer(&mut self, buffer: BufferId, new_byte_length: u64)
        -> Result<Handled, HostError> { Ok(Handled::Unhandled) }
    // ECMA-262 27.5.1.9: promises
    fn promise_rejection_tracker(&mut self, cx: &HostCx<'h, '_>, promise: PromiseRef<'h, '_>,
                                 operation: RejectionOperation) {}
    // Temporal 2.3.1: time
    fn system_utc_epoch_nanoseconds(&mut self, realm: RealmId) -> i128;
}
```

Methods with a spec default implement it. The six without a body have none an embedder could inherit: the four enqueue hooks and module loading have no default in the specification, and system time's default reads a clock, which engine code may not do (G13). Every host decides them.

**Hosts that are not web browsers (R0-H5).** ECMA-262 requires hosts that are not web browsers to use the default HostMakeJobCallback and HostCallJobCallback, and Temporal requires the default HostSystemUTCEpochNanoseconds; HostEnsureCanAddPrivateElement "is only invoked by ECMAScript hosts that are web browsers". Izcal's DOM-less embedders (the proxy resolver, the code-cache builder, the component host's extension and app configurations, JS spec 6.0) are not web browsers in HTML's sense. They implement `JsHost` through a provided `NonBrowserHost` adapter that fixes those four methods to the defaults and forwards the rest, and G6's embedder crates are checked to use it.

## 4. Host layering points that are not hooks

Annex D lists further points where the host decides behaviour. They are not trait methods of `JsHost`, but `hg-js-host` must expose them, and they are frozen with it:

| Point | ECMA-262 | How `hg-js-host` exposes it | Class |
|---|---|---|---|
| Realm creation and the global object | [InitializeHostDefinedRealm](https://tc39.es/ecma262/#sec-initializehostdefinedrealm), [D.3](https://tc39.es/ecma262/#sec-host-defined-objects-summary) | `agent.create_realm(cx, global)`: the host supplies the global object's shape family and `this` value | P |
| `[[HostDefined]]` on realms, scripts, modules and job callbacks | [D.2](https://tc39.es/ecma262/#sec-host-defined-fields-summary) | A `HostData` per record, set by the host, untrusted on read (H-2) | P |
| Job preparation and cleanup steps | [9.5](https://tc39.es/ecma262/#sec-jobs), [D.4](https://tc39.es/ecma262/#sec-host-running-jobs) | The host runs jobs itself (`agent.run_job`, `agent.run_cleanup_job`) and does its own preparation and cleanup around them; the engine exposes whether the execution-context stack is empty, cheaply (JS spec 6.14) | Q or T, by job kind |
| Releasing kept objects | [ClearKeptObjects](https://tc39.es/ecma262/#sec-clear-kept-objects) | `agent.clear_kept_objects()`, called by the host when a synchronous run of script ends (HTML: in the microtask checkpoint) | P |
| Reporting a cleanup job's error | [9.9.4.1](https://tc39.es/ecma262/#sec-host-cleanup-finalization-registry) ("host-defined steps for reporting the error") | `agent.run_cleanup_job` returns the completion; the host reports it | P |
| Exotic objects' internal methods | [D.5](https://tc39.es/ecma262/#sec-host-internal-methods-of-exotic-objects) | Shape families with host hooks for the overridden internal methods, generated by `hg-webidl-codegen` (JS spec 6.2) | per object |
| Built-ins not in the specification | [D.6](https://tc39.es/ecma262/#sec-host-built-in-objects-and-methods) | Host functions created through `hg-js-host` (`%CreateHostFunction` is JS-M0's test form) | per function |
| Agent records | [Agents](https://tc39.es/ecma262/#sec-agents) | `[[CanBlock]]` and the other agent fields are set at agent creation, from HTML's agent kind | P |
| `[[IsHTMLDDA]]` | [Annex B](https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot) | A shape flag a host exotic family may set (`document.all`) | P |

## 5. Environment reads that are not hooks (R0-H6)

Stage 1 3.3 says `Math.random`, `Date.now` and `performance.now` "are host hooks". In ECMA-262 none of them is: `Math.random` uses "an implementation-defined algorithm or strategy" ([Math.random](https://tc39.es/ecma262/#sec-math.random)), `Date.now` returns the current time with no hook until Temporal merges, and `performance.now` is HTML's and HR-Time's, not the engine's. What reaches the engine from outside must still go through the substrate or the host (G13). The decisions:

| Read | Spec status | Izcal's route | Substrate | Class |
|---|---|---|---|---|
| `Math.random` | Implementation-defined algorithm; a distinct sequence per realm | Directly from the substrate, not through `JsHost`: each realm draws from a stream of `Entropy::stream(EntropyPurpose::Random)` derived for that realm, so every embedder gets the same behaviour and no host can weaken it | `Entropy` | I |
| `Date.now`, `Date()` | No hook in ECMA-262; HostSystemUTCEpochNanoseconds once Temporal merges | Through row 18 **now**, in Temporal's form, so nothing changes when the merge lands | `Clock::wall` | I |
| `performance.now` | HTML and HR-Time | Not engine work: `hg-script` and `hg-bindings` | `Clock::monotonic` | I |
| System time zone | [SystemTimeZoneIdentifier](https://tc39.es/ecma262/#sec-systemtimezoneidentifier), implementation-defined | A value of the agent's configuration, set at agent creation by the host and recorded (it is also a fingerprinting surface the privacy design decides) | — | I |
| Time-zone data | [GetNamedTimeZoneEpochNanoseconds](https://tc39.es/ecma262/#sec-getnamedtimezoneepochnanoseconds), [GetNamedTimeZoneOffsetNanoseconds](https://tc39.es/ecma262/#sec-getnamedtimezoneoffsetnanoseconds), [AvailableNamedTimeZoneIdentifiers](https://tc39.es/ecma262/#sec-availablenamedtimezoneidentifiers), implementation-defined | `hg-tz` over the tz release pinned in the build (JS spec 6.13) | — | P |
| List separator | [LocalizedListSeparator](https://tc39.es/ecma262/#sec-localizedlistseparator), implementation-defined | `hg-intl` over the CLDR release pinned in the build | — | P |
| Weak clearing and cleanup | [Execution](https://tc39.es/ecma262/#sec-weakref-execution) "at any time" | Only when a canonical cycle completes, at a decision point (JS-D15); enqueueCleanup always true (row 7) | `DecisionLog` | D |

**A substrate finding.** `hg-substrate-sim`'s `stream(purpose)` seeds each new stream from the seed, the purpose and a counter shared by all purposes (`crates/hg-substrate-sim/src/entropy.rs`). So the realm streams are reproducible from a seed, but creating one more stream of any purpose earlier in a run shifts every later realm's sequence, which the trait's own documentation says should not happen ("adding a consumer does not perturb the others' sequences"). The fix proposed for Phase 0 is a keyed stream, `stream_keyed(purpose, key)`, with the realm's canonical index within its agent as the key (open question Q-R0-8).

## 6. What JS-M0 implements (R0-H9)

The plan puts host hooks at JS-M1, but JS-M0's core operation set already needs some ([bytecode note](bytecode-and-core-set.md) Section 5): the FinalizationRegistry constructor calls HostMakeJobCallback, weak clearing calls HostEnqueueFinalizationRegistryCleanupJob, the cleanup job calls HostCallJobCallback, kept objects need ClearKeptObjects at job boundaries, and `%DateNow` needs HostSystemUTCEpochNanoseconds. JS-M0 therefore freezes the whole trait on paper and implements a **test host** in the probe (plan Phase 7) with:

| Hook or point | Test host's implementation |
|---|---|
| HostMakeJobCallback, HostCallJobCallback | The defaults (the test host is not a web browser, R0-H5) |
| HostEnqueueFinalizationRegistryCleanupJob | Posts a task on the simulator's executor; the task runs `agent.run_cleanup_job`, then `agent.clear_kept_objects()` |
| HostSystemUTCEpochNanoseconds | `Clock::wall` of the simulation substrate (virtual time), no coarsening |
| ClearKeptObjects | Called after each job the probe runs |
| Every other hook | Present in the trait; the test host records an unexpected call as a probe failure |

## 7. Freezing and changing the trait (R0-H10)

- **Frozen at JS-M0:** the 18 methods, their parameters, return types and contexts; the ID-not-reference rule; the non-browser adapter; the layering points of Section 4; the environment routes of Section 5.
- **What changes it:** a new host-defined clause in a pinned specification (the CI audit fails until a method exists), or an owner decision recorded in JS spec 6.14. Helper-type names may change at JS-M1's first implementation without changing those properties.
- **Why freeze now.** `hg-script` (M2), `hg-bindings`, the DOM-less embedders and the test host all implement this trait. A late change to a context, for example letting HostPromiseRejectionTracker allocate, would move a GC point into every caller of RejectPromise and invalidate their rooting; deciding the contexts now is what lets the operation facts of [hgop-language.md](hgop-language.md) name each hook's effects from JS-M0.

## 8. Decisions in this note

| ID | Decision | Alternative | Cost | Reversal trigger |
|---|---|---|---|---|
| R0-H1 | 18 hook methods: ECMA-262's 16, Temporal's HostSystemUTCEpochNanoseconds and Dynamic Code Brand Checks' HostGetCodeForEval, with that proposal's eight-parameter HostEnsureCanCompileStrings | 16 or 17 now, the rest when merged | Two methods ahead of ECMA-262's merge | The proposal drops back below Stage 3, or its hook changes shape |
| R0-H2 | The host never holds a GC reference: IDs, `HostData` and held handles from engine root tables | Hosts hold `Root`s across tasks | Engine tables for jobs, payloads and held handles | — |
| R0-H3 | Spec requirements enforced by types where possible (`JobCall`, `&'static` attribute lists, `&self` with plain data for deterministic hooks) | Documentation and tests only | Slightly unusual signatures | — |
| R0-H4 | Each hook's context (none, `&HostCx`, `&mut HostCx`, `&mut ScriptCx`) is frozen with its signature | Every hook gets full script access | Hosts needing more must ask for a spec change | A hook HTML redefines to run script where this table says it cannot |
| R0-H5 | DOM-less embedders implement the trait through a `NonBrowserHost` adapter that fixes the four browser-only hooks to their defaults | Each embedder takes care | One adapter | — |
| R0-H6 | `Math.random` draws from the substrate per realm; `Date.now` goes through HostSystemUTCEpochNanoseconds already; time zone is a recorded agent configuration | `Math.random` through a host method | Stage 1 3.3's wording changes | — |
| R0-H7 | enqueueCleanup is always true, and cleanup is enqueued while the GC-completion decision is applied | Implementation-chosen | Every cleared cell's registry is scheduled | — |
| R0-H8 | Each hook has a determinism class (P, Q, T, I, D, U) | Unclassified | — | — |
| R0-H9 | JS-M0 implements a test host for the core set's hooks | Wait for JS-M1 | A small test host | — |
| R0-H10 | The audit is a CI test against the pinned specifications | A one-time review | One test | — |

## 9. Open questions

- **Q-R0-7.** Will HTML's HostEnsureCanCompileStrings gain the proposal's `direct` parameter, or the proposal drop it? The trait takes the superset either way; the answer only changes which arguments `hg-script` reads.
- **Q-R0-8.** Should `Entropy` gain `stream_keyed(purpose, key)` in Phase 0 (Section 5), or should the engine derive realm streams from one per-agent stream in realm-creation order? Recommended: `stream_keyed`, because the per-agent derivation still depends on the order agents are created in the process.
- **Q-R0-9.** Is the system time zone exposed to pages at all, or fixed per container for privacy? The engine supports either through the agent configuration; the privacy and fonts and text specs own the choice.
