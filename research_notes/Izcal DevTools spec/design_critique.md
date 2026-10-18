# Izcal DevTools spec: design critique (phase 8)

Independent design critique of `reports/Izcal DevTools spec.md` (draft 0.1, 17 October 2026), written 18 October 2026 by a critic who did not write the spec. Method: the first-principles-spec skill (phase 8) and its references. I checked the spec against HGBA-2 (Section 2 P1–P6 and M-1, Section 4.1 I-1 to I-6, ANC-1, Section 7), HGBA-3 (P1–P7, B1, B2, B4, B5, B8, B9, B10, Section 7, DET-1/2, EQ-1, INC-1, AG-0 to AG-4), the Stage 1 design (Sections 1–4, 5.4–5.8, 6, 7, 8, 11, 13.2) and the shell UI design (UI-1, UI-2, UI-10, UI-11), plus `m0/determinism_crossplatform.md`. I did no web research. Line numbers refer to the spec as of this critique.

**Counts:** High 10, Medium 14, Low 9 (33 findings).

---

## High

### 1. Recording starts in the middle of a session, so the log cannot be replayed (DT-D2, line 230; DT-1)
**Issue.** B1 and Stage 1 design 3.4 record **inputs** (clock, entropy, scheduler choices, IPC, network bytes). A log of inputs replays only from a known initial state. DT-D2 starts recording "when the user opens DevTools on a tab", which is usually long after the page loaded, and it keeps the log in a 512 MB **ring buffer** that drops its oldest entries. The renderer's state at the moment recording starts (V8 heap, `cppgc` DOM, data plane, storage responses already received) is not in the log, and once the ring wraps, the start of the log is gone too. The "checkpoints" in `hg-recording` are **hashes**, not state snapshots, so a replay has nowhere to start from. Q-DT-3 treats checkpointing as a latency question; it is a correctness question.
**Why it matters.** The spec's central idea (everyday views as queries over a recording, retroactive logpoints, scrubbing) fails in the most common case: DevTools opened on a page that is already loaded.
**Fix.** State that a replayable recording begins at a navigation boundary. When DevTools opens, offer "Reload to record" and label everything before that as live-only. Make the ring buffer evict whole navigations, never part of one. If replay must start mid-session, specify real checkpoints: snapshots of simulated state taken inside the replay host (for example fork-based), never of the production process. Add the case to DT-1, and move Q-DT-3 into the design as a prerequisite.

### 2. "That tab's substrate" does not exist: renderers are shared across tabs of a site (DT-D2, line 230; Section 6.5 item 9)
**Issue.** Renderers are "1 per site … per site instance" (Stage 1 design Section 1). S-3 allows "process sharing *within* a site", and RES-3 drops per-tab figures "because process sharing makes them ill-defined". The substrate is per process, not per tab. To replay one tab you need every input to the shared event loop and V8 isolate, so the inputs of other same-site tabs, shared workers, service workers and `BroadcastChannel` peers are recorded too. That contradicts "Tabs without DevTools are never recorded". If those inputs are left out, the replay is wrong.
**Why it matters.** It is both a privacy breach (other tabs' content and keystrokes get recorded) and a correctness break (replays diverge).
**Fix.** Choose one rule and write it down. (a) While a tab is inspected with recording on, it gets a dedicated process from its next navigation. Record that this is a process-model change, and check it for P2 effects. Or (b) record the whole process and tell the user which other tabs are included. Workers and service workers that the tab talks to need an explicit scope. Amend Stage 1 design 3.4 to match.

### 3. Production recordings depend on V8 platform determinism that M0 showed exists only in the predictable configuration (DT-D2, DT-1(b), DT-15; Q-DT-1, line 455)
**Issue.** M0 (`m0/determinism_crossplatform.md`) found that the stock platform diverges on every OS: 848 distinct GC hashes, and `Date.now` differs on every run because the stock platform clock cannot be routed through the substrate (Stage 1 design 3.3). Determinism came from the fork's predictable platform with worker tasks run inline or queued on one scheduler, and the worker-task mode itself changes results. Production renderers will run V8's worker threads for real (concurrent GC and compilation). The spec reduces this to "GC-observable behavior … may not replay" and a Stage 2 experiment, but it already commits to recording in production at ≤5% overhead (DT-15) and ≥95% exact replays (DT-1(b)).
**Why it matters.** If recording needs a "recordable V8 mode" (predictable platform, no concurrent GC or compilation), that is the "separate replay mode" DT-D2 rejects. Its performance cost is unmeasured and probably well above 5%, and Performance views would profile an engine configuration that differs from production.
**Fix.** State the prerequisite in DT-D2: production `Date.now` routed through the substrate clock in the production configuration of the fork, and a decision on whether recording tabs run a recordable V8 configuration. Move Q-DT-1 to an M1–M2 probe (production platform plus recording, corpus pages), before any recording-backed view is designed. Add DT-1(b) and that probe's result to DT-D2's reversal trigger (line 320).

### 4. The replay host co-locates cross-site data and runs hostile script for every frame, with no P1 exception (Processes table, line 198; `hg-why`)
**Issue.** The replay host runs `hg-substrate-sim`, which by Stage 1 design 3.2 runs the **whole multi-process browser in one OS process**. Replaying a tab with cross-site iframes therefore puts every site's DOM, V8 heap and response bytes in one address space, and runs each site's hostile JavaScript there. P1 says "never co-locate cross-site data in one address space", and HGBA-2 Section 2 requires a recorded exception for any trade-off. The spec records an exception only for the DevTools renderer (item 6), not for the replay host.
**Why it matters.** A renderer exploit replayed in the replay host reaches every site's data in the recording. This is exactly the Spectre and site-isolation case I-1 exists to prevent.
**Fix.** Replay **per recorded process**: B2 already records each process's IPC inputs, so each site's renderer can be replayed in its own replay host, keyed by (site, container). Stitch cross-process why-chains together in `hg-inspect`. If whole-browser simulation is kept, record a P1 exception with an owner sign-off, and add the replay host to the per-process threat models (Stage 1 design 13.2).

### 5. The DevTools renderer's P1 exception is wider than data concentration: it is a cross-site pivot (Section 6.5 item 6, line 310)
**Issue.** The DevTools renderer is a full Izcal renderer (V8, layout) that parses data from page renderers it does not trust, and it holds IIP endpoints into every inspected frame, including write capabilities (evaluate, edit, navigate). A compromised site-A renderer that corrupts the DevTools renderer gains evaluate rights into site-B frames. That is a site-isolation bypass, not only a read of B's data. Trusted Types and CSP block HTML injection, but they do nothing against memory corruption or logic bugs in the engine. HGBA-3 B8 already ruled against browser-side aggregation across origins because of CVE-2026-95301. The reversal trigger fires only after an exploit has been found.
**Why it matters.** It weakens I-1 and I-2 for every user who opens DevTools on a page with cross-site frames, and the spec gives no rationale for the trade-off of the kind HGBA-2 Section 2 requires.
**Fix.** Default to per-site panes for cross-site frames: one DevTools renderer per (site, container), composited like out-of-process iframes. Or, at minimum, the top-level DevTools renderer holds **no write capability** for cross-site frames; evaluate and edit there go through a per-site helper, and sensitive classes for cross-site frames are fetched only into that helper. Put the decision in front of the owner before M6, and add the DevTools renderer, replay host and automation gateway to Stage 1 design 13.2's threat-model list.

### 6. CDP, BiDi and DAP attachment bypasses B9 and makes AI-off meaningless for agents (Section 6.3 Transports, lines 290–295; 6.5 items 1–3)
**Issue.** P1 (HGBA-3) says "an AI agent is a separate principal", and B9 confines agents to origin sets. The spec admits that the broker cannot tell a human client from an agent (item 1). Yet the `automation(client)` and `editor(dap)` principals, once consented, get what amounts to ambient capabilities over the named container: evaluate and full DOM access. Any agent (Chrome DevTools MCP, Playwright MCP, Claude Code over `izcal-dap`) can drive Izcal through `hg-cdp`, `hg-bidi` or `hg-dap` with AI off, with no origin sets, no provenance filtering and no B9 action log. DT-13 tests only the MCP server.
**Why it matters.** The MCP hardening (DT-12, DT-13) is easy to step around. It repeats the "agents as CDP clients with ambient access to every origin" pattern that B9 says it **replaces**.
**Fix.** Make origin-set enforcement a property of **every** external session. The consent prompt chooses the origins, read or write, and the sensitive classes, and the default is read-only with no sensitive classes. B9's enforcement mechanism (not the agent host) must therefore exist with AI off; state that this is not an "agent-only computation" under B10. Attaching to a user container over CDP or DAP needs a setting plus a per-session grant. Run the DT-13 injection and exfiltration suite against the CDP, BiDi and DAP surfaces too.

### 7. Redacted recordings cannot reproduce, and recordings contain keystrokes (T9; Section 5 Elm/Replay row; 6.5 items 5 and 9; DT-10)
**Issue.** T9 promises "a portable, redacted artifact that reproduces". A recording is an input log. Redacting response bodies, cookies or storage values changes what the replay computes, so it diverges at the first use of a redacted value. The input log also holds keystrokes typed into password and form fields while DevTools was open, and DT-10's canary list covers structured export fields, not raw input events or network bytes inside recording bundles.
**Why it matters.** Either bundles are redacted and do not reproduce, or they reproduce and leak secrets, which is the Okta HAR lesson the spec cites (line 169).
**Fix.** Specify redaction with verification: redact, replay the bundle, and show the user where the first divergence falls ("exact until 00:41"). Where values reach only opaque sinks, substitute them consistently. Do not record keystrokes into password fields; replace them with length-preserving placeholders, and label the replay approximate from that point. Extend DT-10 to recording bundles (seed canaries in typed input and response bodies), and add a DT-1 sub-target for redacted bundles.

### 8. DT-2 is unsound: it tests only sufficiency, can be passed by over-reporting, and is not what B5's reference path checks (DT-D3 Verification, line 236; DT-2, line 359)
**Issue.** (a) DT-2 checks that "applying only the reported chain to the earlier state reproduces the new value". An answer that reports **every** mutation since the earlier state always passes, so the target cannot fail for over-broad answers, which are exactly the useless ones. (b) "Earlier state" is undefined, and "apply only these mutations" is ill-defined when the excluded mutations created the nodes the kept ones touch, or when the chain contains script. (c) "Reverting reproduces the old value" follows from (a) and adds nothing. (d) B5's reference path verifies that incremental output equals from-scratch output. It says nothing about causation. (e) INC-1's corpus is mutation traces without JavaScript, so the input → script → mutation links are never verified.
**Why it matters.** "Exact why-answers confirmed by the reference path" is the spec's headline differentiator, and as written its correctness target cannot catch a wrong answer.
**Fix.** Define correctness as **sufficiency plus necessity**, tested in simulation by counterfactual re-execution. Suppressing each reported cause (an input event, a script task, or a mutation) and re-running from the last real checkpoint must change the target's value. Applying the chain on the reference path must reproduce it. Publish the chain-size distribution. Run the test on the DT-1 recorded session set (with JS), not on INC-1 traces. Describe the reference path as the value oracle, not the causal oracle.

### 9. Option A's staffing does not add up and pushes cost onto other tracks (Section 9.2, lines 404–415)
**Issue.** The spec's own table costs IIP at 1 engineer and the frontend at 2 (3 in total). Option A keeps the track at **2†** while adding to D-S11's four panels: IIP plus the codegen backend, the broker session and consent model, the conformance suite, table-first accessibility, the CLI, closed transports and the CDP JS profile. It removes only the CDP DOM, CSS, Network, Page and Overlay domains, which 13.2 says "share code with" the WebDriver server anyway. It also depends on Infrastructure (WebDriver on `hg-bidi` over IIP), Substrate (production recording and the ring buffer), Network (qlog and events), Shell (prompts) and Accessibility (audits) doing work that their 8.3 estimates do not include. Yet the spec says it "does not change Section 8.3's numbers".
**Why it matters.** Under D-S8, funding is the gate on staffing. An under-costed Stage 1 track turns into either a missed M6/M8 or quiet scope cuts in security surfaces (consent, redaction).
**Fix.** Re-cost Option A honestly. It is more likely 3† than 2†, or it has to drop items (for example the CLI's CI features, or the conformance suite until M7). List the added work per track in 9.3, with an engineer-month estimate for each, and give the owner the total change to 8.3.

### 10. Making IIP the base of the M2 WebDriver server puts the DevTools track on the critical path (Section 9.1 M2, line 388)
**Issue.** Stage 1 design 7.3 makes the WebDriver server M2 infrastructure, because WPT drives everything through it, and M2 sits on the critical path (M0 → M1 → M2 …). The spec builds that server "on `hg-bidi` over IIP" at M2. That needs the `hg-inspect` broker, `hg-inspect-renderer`, the codegen backend and a stable schema to be ready at M2, all from a new track whose staffing is blocked on D-S8.
**Why it matters.** A slip in IIP stalls WPT measurement for the whole engine, and the M2 review sets C-1 floors and the duration estimate.
**Fix.** Let the M2 WebDriver server use a minimal, experimental-tier IIP slice owned by Infrastructure (navigation, script evaluation, element lookup, input, screenshots), with a documented migration to the full IIP by M7. Or let it call broker endpoints directly until M5. State the fallback in 9.1 and add it to the risks table.

---

## Medium

### 11. Probes and debuggers perturb GC, so "probe cost cannot change what the page observes" is false for some behavior (DT-D3, line 234; DT-7; DT-16)
**Issue.** Under the predictable platform, GC timing is a function of allocation. Probes that capture JS stacks, logpoint expressions and an attached V8 inspector (source positions, breakpoints, side-effect checks) allocate on the JS heap. That shifts GC and so changes `WeakRef` and `FinalizationRegistry` outcomes. DT-16 leaves out breakpoints and evaluation, and DT-7 compares one debugged replay against another debugged replay, never against the original recording.
**Fix.** Allocate probe data off-heap (Rust side). Evaluate retroactive logpoints in V8's side-effect-free mode and label them "may perturb GC-observable behavior". Add a DT-1 clause: replay with probes, logpoints or breakpoints attached matches the recording's checkpoint hashes, or flags the divergence.

### 12. Execution points finer than a task need a V8 progress counter that V8's public API does not provide (Identity and time, line 186; DT-7)
**Issue.** "Execution point (recording ID, sequence, task number, **step**)" and exact `reverseContinue` need a way to name "the N-th point inside a task". Replay-based debuggers get this from an instruction or progress counter inside the JS engine. The fork's policy is "changes written against V8's public interfaces", and the public inspector offers pauses and steps, not counters.
**Fix.** Define the step as task plus microtask checkpoint plus the count of inspector pause events at a location. Implement reverse-continue as: go back to the previous real checkpoint, replay forward, and count breakpoint hits. State the cost, and restrict the DT-7 scenarios to that model. If a counter in the fork is wanted, raise it as a fork-policy decision.

### 13. Per-layer fault injection under replay is not possible for production recordings (DT-D5, line 240)
**Issue.** TLS alerts, QUIC loss and DNS failures sit below the boundary the renderer replay records. Re-running the network stack would need the TLS handshake randomness, and the production `Entropy::crypto` is the OS CSPRNG only (Stage 1 design 3.1), so it cannot be reproduced. Also, everything after an injected fault is a new execution, not a replay. Stage 1 design 3.2's faults are byte-level (resets, slow or reordered bytes), not protocol-level.
**Fix.** Offer per-layer faults only in **simulation**, against corpus archives or Izcal-run mock servers in the simulator, and call them "what-if runs" (diverged from the recording at the fault). Amend Stage 1 design 3.2 to add protocol-level faults, and add a target (for example: a seeded fault produces the expected error path in 100% of a suite).

### 14. "Mapped" explanations are never checked for being right (DT-D4, line 238; DT-3)
**Issue.** For behavior inside Stylo or Taffy, `hg-explain` uses a hand-curated table that maps decisions to spec sections. DT-3 checks only that links resolve and are labelled. A wrong mapping shows a confident, spec-cited explanation that is false, which is the AI-distrust problem the spec uses to argue against AI explanations (line 151).
**Fix.** Join each mapping entry to at least one WPT test, or a spec-derived property test, that exercises the decision it explains. Run a sampled expert audit each release with a published error-rate threshold†. Show "mapped" visibly as weaker than "anchored".

### 15. Stylo's API limits threaten the Stage 1 Styles pane, not only why-queries (M3, line 388; Q-DT-4)
**Issue.** Matched rules, cascade order, overridden declarations and source locations are the core of the Elements panel at M6. The spec defers what Stylo exposes to Q-DT-4 and treats it as a why-query risk. Under M-1, Stylo is used unmodified, so if its public API lacks this data, Izcal needs its own cascade explainer written from CSS Cascade. That is unbudgeted work.
**Fix.** Run a Stylo public-API spike in M1–M2. Budget the fallback: an Izcal-built explainer that recomputes the cascade for one element, from the CSS Cascade specification, and is checked against Stylo's computed values, which also makes it an oracle.

### 16. The CDP profile's oracle collides with M-1 part 4, and DT-6 checks itself (`hg-cdp` anchoring, line 341; DT-6)
**Issue.** M-1 part 4 says other engines' outputs are never the definition of correct. For CDP no specification exists, so compatibility **means** matching Chrome's observed behavior. The spec settles implicit semantics by black-box observation but never records that this departs from part 4 for a compatibility interface. DT-6's "100% pass Izcal's profile suite" is self-referential, so it can be written to pass.
**Fix.** Record an explicit M-1 ruling that compatibility adapters (`hg-cdp`, and the legacy HAR fields) may use recorded black-box observations as their oracle, and that this does not cover web-visible behavior. Measure DT-6 with third-party client test suites run as black-box drivers (for example Puppeteer's own tests and js-debug's integration tests), with pass rates published.

### 17. V8's inspector protocol is CDP, which contradicts "one source" and DT-4(c) (lines 184, 361; Section 6.3 CDP row)
**Issue.** The script domains come from V8's inspector, and those messages are CDP JSON defined by V8, not generated from `.hgidl`. IIP therefore either passes CDP through, which puts an engine-shaped protocol at the core (the thing DT-D1 rejects), or translates CDP to IIP and back for `hg-cdp`, the bridge cost that killed RemoteDebug and Valence (line 54). DT-4(c), "100% of IIP messages are generated from `.hgidl`", cannot hold.
**Fix.** Pick one approach and say so. Either (a) a script domain that passes V8's inspector through, with its own stability rules (tracking V8 updates) and exempt from DT-4(c); or (b) an IIP script domain with a costed translation layer and a conformance test against js-debug.

### 18. The consent model does not cover stateless MCP, client identity, CI or headless embedders (Transports, line 293; 6.5 item 2)
**Issue.** MCP 2026-07-28 is stateless, so "consent per connection" means either a storm of prompts or long-lived grants, and the spec forbids "always allow". A peer-credential check proves only that the client runs as the same user. A client-supplied name ("Claude Code") can be spoofed, so the prompt cannot honestly say who is asking. Headless and cloud agent-runtime builds (B10's embedder path) have no trusted region in which to show a prompt.
**Fix.** After consent, issue scoped, time-bounded grant tokens (origins, read or write, classes, expiry), revocable from the "being controlled" banner. Show the verified executable path or code signature in the prompt. For headless use, take grants from launch-time configuration over the pipe (never from project files), and record it in the B9 action log.

### 19. Source write-back is promised through DAP, which has no request for editing files (Section 6.3 Editors row, line 284)
**Issue.** DAP carries debugging, not edits to files in the workspace. LSP edits come from server to client (`workspace/applyEdit`), which would require Izcal to be an LSP server. Mapping DOM edits back to framework source (JSX, templates) through ECMA-426 is not possible in general. Only CSS with source maps to the authored files maps cleanly.
**Fix.** Specify the mechanism (for example `izcal-lsp`, a small LSP server that applies CSS change sets as workspace edits). Limit Stage 2 to CSS, and leave DOM edits as reviewable patches.

### 20. The "permanent surfaces" copy Chrome's panel set, which contradicts arranging by task (line 257)
**Issue.** "Console, Elements, Sources, Network, Timeline and Application" are today's tab-per-tool layout, the template the skill warns against. They are not derived from T1–T10 (Section 4.1). The spec says DevTools are "arranged by task" (summary, DT-D7), but the permanent surfaces are justified only by familiarity.
**Fix.** Derive each permanent surface from the tasks, or justify keeping each subsystem panel explicitly in Section 4.4's table. Apply DT-19 to the permanent surfaces too (with the exemptions in finding 29).

### 21. "Why was this interaction slow?" (T2) cannot be answered by replay alone (DT-D3; T2; DT-15)
**Issue.** A replay runs at a different speed, under probes, from recorded clock reads. It shows causes but not costs. Cost attribution needs a live sampling profiler during the original run, and possibly in a recordable V8 configuration whose timings differ from production (finding 3).
**Fix.** Specify a live sampling profiler (V8's CPU profiler through `hg-script`) that runs during recording, with samples tied to execution points. Include it in the DT-15 overhead measurement. State that times shown under replay are recorded-clock times.

### 22. The WebExtension `devtools.*` API contradicts W-1 (Author views, line 288)
**Issue.** W-1 and D-S3 put extensions in WASI 0.3 components on Wasmtime, with no extension code in renderers. Neither decides on a WebExtension API, and W-1 lists "keep the legacy extension model" as the alternative it rejected. The spec quietly assumes `devtools.*`.
**Fix.** Define author views as IIP `views`-domain clients and W-1 components with declared capabilities. If WebExtension compatibility is wanted, raise it as an owner decision.

### 23. The owner's "new ways to interact with protocols" is only partly met (DT-D5; Section 6.3)
**Issue.** The lens asks for mock servers and the developer driving the network. Missing are: local response overrides and mock endpoints (in a live session and in simulation); per-request protocol forcing and A/B comparison (H1/H2/H3, ECH on or off, the same page side by side); replay of a recorded exchange against the live server, with a diff; a Reporting API view. Edit-and-resend has no capability rule (it sends the user's cookies), and no target covers any of the interactive network features.
**Fix.** Add these to DT-D5. Gate edit-and-resend and overrides on the session's write capability for that origin. Add a target (for example: overrides and forced protocols take effect in 100% of a suite, and are labelled in every affected view).

### 24. Retention and secondary uses of recordings are not specified (6.5 item 9)
**Issue.** The spec does not say where saved bundles live, whether they are encrypted at rest, how long they are kept, whether deleting a container deletes them, or whether a renderer crash attaches a recording or the replay host's memory to a D-S4 opt-out crash report (minidumps can hold stack memory). It also does not say what "Share with agent … recording range" actually transfers (raw bundle or filtered query results).
**Fix.** Never attach recordings to crash reports. Encrypt saved bundles at rest with the profile key. Delete a container's recordings with the container. Agents receive only provenance-filtered query results, never raw bundles.

---

## Low

### 25. DT-11's "0 listening TCP or UDP sockets" is ill-defined for QUIC and WebRTC (line 368)
The network service binds UDP sockets for QUIC, and WebRTC ICE binds them too. Tools often list bound UDP sockets as "listening". **Fix:** define the check as no TCP socket in LISTEN state and no UDP socket accepting unsolicited inbound traffic outside client-initiated flows, and name the scan tool and its criteria per OS.

### 26. G12 cannot be built as written (line 227)
`hg-replay-main` needs the OS substrate for its own process plumbing (IPC to the broker, threads), and G3 makes `hg-substrate-os` the only crate allowed those. **Fix:** "engine crates inside the replay host are given only a `hg-substrate-sim` instance; only the host shell may use `hg-substrate-os`", enforced by type (the simulated `Substrate` is constructed in one place).

### 27. G10 and G11 are hard to enforce mechanically (lines 225–226)
`hg-devtools` is HTML and JS, so a lint over `cargo metadata` cannot see its imports. A call-graph lint over Rust with `dyn` dispatch is unsound. **Fix:** a JS import lint for `hg-devtools`. Export writers accept only a `Redacted<T>` type that only `hg-redact` can construct.

### 28. DT-12 conflicts with the crate table on `izcal-mcp` (lines 220, 369)
The crate table says `izcal-mcp` is "built only with `ai`", but DT-12 expects it to "report unavailable". **Fix:** settle Q-DT-7 first. If the shim is built only with `ai`, DT-12 checks that it is absent from default packages.

### 29. DT-19 can remove accessibility-specific views; opt-in telemetry is biased (line 376)
A view used by screen-reader developers falls below 0.5% of sessions by construction. **Fix:** exempt views that serve UI-1, UI-2 and DT-9 tasks, measure per task rather than per view, and publish the size of the opt-in sample next to each figure.

### 30. DT-14 can be met by truncating results (line 371)
**Fix:** pair it with task success on the same task set. Compact results must not lower success compared with uncompressed results (the AG-2 method: fixed model and scaffold).

### 31. DT-1(b)'s exclusions are open-ended and missing it costs nothing (line 358)
DET-1's out-of-scope list ends with "anything else the simulator does not model", so any failure can be excluded. **Fix:** list the excluded classes, publish the share of sessions that hit each class, and add "fewer than X%† of sessions fully exact" to DT-D2's reversal trigger.

### 32. A stable tier committed early freezes an immature data model (DT-4, line 361)
Twelve months of deprecation on domains that are stable from M2 would lock in node IDs, execution points and the recording format before Stage 2 changes them (B5 incremental layout, a possible D8 fallback). **Fix:** keep every IIP domain experimental or beta until Stage 2 exits. BiDi, DAP and MCP carry the outside stability promise until then.

### 33. DT-16 checks only under replay, which misses observer effects in production and through capture (lines 257, 373)
Opening DevTools adds a process and memory (affecting bfcache eviction and discard, BF-1), adds timing overhead (the classic DevTools-detection tricks), and the overlay layer may appear in tab capture (`getDisplayMedia`, Region Capture). **Fix:** keep overlays out of capture surfaces, add a production A/B check on bfcache and discard behavior, and document that the timing overhead is detectable.

---

## Verified sound

- **M-1 compliance section** (Section 7): it is explicit that no code is taken from any DevTools, chromium-bidi, Replay or any MCP server, AI tools included. Ideas (RDP's actors, Perfetto flows, CodeTalk, Safari's enablement) are cited as evidence, not templates. The dependencies (neqo, rustls, AccessKit, hickory-dns, gimli as a candidate) are used through public APIs. Anchoring of `hg-bidi`, `hg-cdp` and `hg-dap` under G9 is consistent.
- **D-S11 conflict handled properly.** The spec proposes amendments (9.4) instead of silently overriding D-S11, D-S2 or Stage 1 design 3.4, and it keeps "no listening port" stricter than D-S11's "port off".
- **B10 and AG-0 alignment.** The MCP server lives in the agent host under feature `ai`, and G4 applies. There are no AI entry points in `hg-devtools`, and "Share with agent" does not exist with AI off. DT-12 reuses AG-0's harness. The case that semantic history is not an "agent extra" (DT-D6) holds, because it is recorded only for the local user.
- **The MCP hardening itself is sound:** empty origin sets, provenance and `untrusted` tags, hashed tool descriptions with rug-pull revocation, no "always allow", no telemetry or external lookups, and DT-13 run with the model's own defenses disabled, extending AG-3.
- **Export redaction defaults, the manifest and a second step for secrets** (DT-10) match the Okta/Chrome 130 lesson. RFC 9850 key logs are confined correctly.
- **Accessibility.** Table-first rendering with a lint (DT-8) matches UI-1 and UI-2. DT-9 is a falsifiable task study with a power analysis.
- **Target hygiene.** Thresholds set by the spec carry †, and none is presented as measured. The `DT-` prefix collides with nothing in `reports/`. Most targets have a failure consequence. Flagged claims (11.2) are carried honestly.
- **Firebug lesson applied structurally** (every client is a remote client), and BiDi conformance is tracked on WPT with non-decreasing floors (DT-5).
- **Section 3.2's "which gaps stay closed"** applies P1 correctly to page-facing API limits.
