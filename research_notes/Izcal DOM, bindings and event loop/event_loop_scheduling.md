# Izcal DOM, bindings and event loop: event loop, scheduling, timers and determinism

Compiled 19 October 2026. Primary sources were preferred (specifications, papers, maintainers' posts, release notes, bug-tracker discussion text).
Confidence: **High** = primary source read and the claim checked in its text; **Medium** = primary abstract or snippet, or a vendor claim without data; **Low** = secondary source only, or recollection not checked this session.

M-1 note: no engine source code was read for this note. A Chromium commit page appeared in one search result (the 1 ms clamp removal); it was not opened and is not cited. The WebKit row cites the bug-tracker discussion text only, not the attached patches.

**Note on row 2.3 (phase 8, 20 October 2026):** the Speedometer figures are from comments in WebKit bug 221124's discussion (Wanming Lin, Ryosuke Niwa, Cameron McCormack), not from a patch, ChangeLog or review. Row 1.6's conflict is resolved: BCD lists `scheduler.yield()` in Firefox 142.

**Method notes**
- This strand was researched by the synthesis author, because the session's concurrent-agent limit was reached when it was due. The session-wide web-search budget ran out after one search; every later row was checked by fetching known primary pages directly. Rows that rest on recollection are graded Medium or Low and listed under "Open questions".
- The specifications strand (`specs_semantics.md`) records the HTML processing model step by step; this note covers how engines schedule in practice, measured congestion, timers, energy, determinism and timing channels.

## 1. Scheduler APIs and their semantics

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | The Prioritized Task Scheduling document defines three priorities (user-blocking, user-visible, background), "run in strict priority order" within one `Scheduler`; continuations outrank tasks of the same priority | Effective priorities 0–5: background task 0, background continuation 1, user-visible 2/3, user-blocking 4/5 | [WICG Scheduling APIs](https://wicg.github.io/scheduling-apis/#scheduler-task-queue-effective-priority) | 2025-05-30 (draft CG report) | High |
| 1.2 | Scheduler tasks use their own "posted task" task source; each event-loop iteration picks a candidate scheduler queue (highest effective priority, then oldest enqueue order), and the UA then chooses between it and ordinary task queues in an **implementation-defined** way; the suggested strategies are non-normative | — | [WICG Scheduling APIs §patches to HTML](https://wicg.github.io/scheduling-apis/#sec-patches-html-event-loop-processing) | 2025-05-30 | High |
| 1.3 | `scheduler.yield()` inherits the originating task's priority and abort signal ("continuation state" carried through promise jobs); `TaskController.setPriority()` changes a live signal's priority and fires `prioritychange` | — | [WICG Scheduling APIs §controlling tasks](https://wicg.github.io/scheduling-apis/#sec-controlling-tasks) | 2025-05-30 | High |
| 1.4 | The document is a WICG Community Group report, "not a W3C Standard nor … on the W3C Standards Track" | — | [WICG Scheduling APIs, status](https://wicg.github.io/scheduling-apis/) | 2025-05-30 | High |
| 1.5 | The spec's own security section says `postTask` delays are whole milliseconds, so it is not a high-resolution timer, but scheduling can still expose cross-origin task timing where origins share a thread | — | [WICG Scheduling APIs §5](https://wicg.github.io/scheduling-apis/) | 2025-05-30 | High |
| 1.6 | `scheduler.yield()` is listed as supported in Chrome 129 (and Edge 129, Firefox 142 per the page's badges; the body text still says "only … Chromium-based browsers"); Safari not supported | Chrome 129; Firefox 142 (badge only) | [Chrome for Developers, Use scheduler.yield()](https://developer.chrome.com/blog/use-scheduler-yield) | 2025-03-06 | Medium (badge and text disagree) |
| 1.7 | `scheduler.yield()` origin trial and flag from Chrome 115; the reason given for a new API: `setTimeout(…, 0)` puts the rest of the work at the **back** of the queue, so other queued tasks run first | Chrome 115 | [Chrome for Developers](https://developer.chrome.com/blog/introducing-scheduler-yield-origin-trial) | 2023-08-29 | High |
| 1.8 | requestIdleCallback caps idle periods at **50 ms** when no frame is pending, so that input arriving just after an idle callback starts can still be answered within 100 ms; inter-frame idle periods are under 16 ms at 60 Hz; `timeout` guarantees the callback runs, with `didTimeout` true | 50 ms cap; 100 ms perception bound | [W3C requestIdleCallback](https://w3c.github.io/requestidlecallback/) | undated ED, read 2026-10-19 | High |
| 1.9 | requestIdleCallback is "not Baseline" because a widely used browser lacks it; Safari support is absent per the synthesis author's recollection | — | [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestIdleCallback) | read 2026-10-19 | Medium |
| 1.10 | The scheduling document runs `requestIdleCallback` callbacks at background priority | — | [WICG Scheduling APIs](https://wicg.github.io/scheduling-apis/) | 2025-05-30 | High |

**Implications:** The scheduler API leaves the cross-source choice implementation-defined, so Izcal's task-selection policy is a design decision it must state and record (B1), not a spec reading. Priorities belong to the event loop of one agent; within a process shared by several same-site agents (JS 6.12) the policy must also decide between agents.

## 2. Timers: clamping, throttling, energy

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 2.1 | HTML's timer initialization steps: "If nestingLevel is greater than 5, and timeout is less than 4, then set timeout to 4" | 5 levels; 4 ms | [HTML timers](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps) | read 2026-10-19 | High |
| 2.2 | "Run steps after a timeout" lets UAs "optionally, wait a further implementation-defined length of time … to pad timeouts as needed to optimize the power usage of the device"; ordering is guaranteed only among calls with the same global and ordering identifier and an equal or shorter delay | — | [HTML timers](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#run-steps-after-a-timeout) | read 2026-10-19 | High |
| 2.3 | WebKit removed its 1 ms clamp for `setTimeout(…, 0)` (r291998, 28 March 2022). Reasons: no such clamp in the spec, and it **reordered timers** (a 1 ms timer scheduled before a 0 ms timer ran first). Measured gains: Speedometer +1% on M1 (2021), none on Intel (2021), +1.7% M1 MacBook Air and +1.2% Intel MacBook Pro (2022). Firefox had no clamp; Chrome planned removal. A later regression broke `setTimeout`/`setInterval` ordering at 0 ms (bug 253423) | +1.0–1.7% Speedometer | [WebKit bug 221124](https://bugs.webkit.org/show_bug.cgi?id=221124) | 2021-01 to 2023-03 | High |
| 2.4 | Chrome 57 budget-based background throttling: after 10 s in the background, each tab has a wall-time budget that regenerates at 0.01 s per second; audible pages exempt; WebSocket and WebRTC pages exempt from the budget but still aligned to 1 s | 10 s grace; 1% regeneration | [Chrome for Developers](https://developer.chrome.com/blog/background_tabs) | 2017-03-09 | High |
| 2.5 | Chrome 88 intensive wake-up throttling: hidden > 5 min, chain count ≥ 5, silent ≥ 30 s, no WebRTC ⇒ timers checked **once per minute**; ordinary hidden throttling checks once per second; visible pages get only the 4 ms nesting clamp. The post gives **no measured savings** | 1/min; 1/s; 5 min | [Chrome for Developers (Archibald)](https://developer.chrome.com/blog/timer-throttling-in-chrome-88) | 2021-01-18 | High |
| 2.6 | Apple's energy guidance: give timers a tolerance of at least 10% of the interval so the system can batch wakeups; waking from idle has an energy cost; "more than one wakeup per second when it should be idle" warrants investigation | 10% tolerance; 1 wakeup/s | [Apple Energy Efficiency Guide](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/power_efficiency_guidelines_osx/Timers.html) | archived guide | High |
| 2.7 | Each vendor added throttle → freeze → discard to engines whose pages run by default (footprint spec's history) | — | [FP Section 2][FP] | 2026-10-19 | High (Izcal doc) |

**Implications:** The power padding HTML permits and the throttling rules every engine adds are web-observable timing; in Izcal they are deterministic functions of recorded inputs (visibility, audibility, the virtual clock), with wakeups aligned to a stated grid, not a hidden heuristic.

## 3. Rendering cadence and measured main-thread congestion

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 3.1 | INP replaced FID as a Core Web Vital on **12 March 2024** because FID did not capture aspects of interactivity | — | [web.dev](https://web.dev/blog/inp-cwv-march-12) | 2024-01-31 | High |
| 3.2 | Web Almanac 2024: good INP (< 200 ms) on **74%** of mobile and **97%** of desktop origins; median TBT 1,209 ms mobile, 67 ms desktop; median long task 108 ms mobile, 90 ms desktop; fewer than 25% of sites below 50 ms | 74% / 97%; TBT 1,209 / 67 ms | [Web Almanac 2024 Performance](https://almanac.httparchive.org/en/2024/performance) | 2024-11-11 | High |
| 3.3 | INP sub-parts (RUMvision data in the Almanac): presentation delay is the largest median contributor (36 ms); input delay dominates poor INP (37 ms p75, 155 ms p90); processing 56 ms p75 | 36 / 155 / 56 ms | [Web Almanac 2024 Performance](https://almanac.httparchive.org/en/2024/performance) | 2024-11-11 | High |
| 3.4 | Long Animation Frames (Chrome 123, origin trial 116–122) report frames whose rendering is delayed beyond 50 ms, with per-script attribution (invoker, `forcedStyleAndLayoutDuration`, `pauseDuration`, source location). It does **not** attribute cross-origin iframes, workers, service workers or extension isolated worlds, and it points to the entry point, not the slow code. Not in Firefox or Safari | 50 ms; Chrome 123 | [Chrome for Developers, LoAF](https://developer.chrome.com/docs/web-platform/long-animation-frames) | 2024-10-14 | High |
| 3.5 | Long Tasks API available since Chrome 58; flags single tasks > 50 ms only | Chrome 58 | [Chrome for Developers, LoAF](https://developer.chrome.com/docs/web-platform/long-animation-frames) | 2024-10-14 | High |
| 3.6 | Input delay (time a task waits in queue behind other tasks) is the largest part of bad INP (3.3), which is the scheduler's responsibility, not script speed alone | — | derived from 3.3 | — | Medium |

**Implications:** The worst responsiveness tails come from queueing, which the scheduler controls: input must be able to preempt between tasks, and the reason any task waited must be observable (no shipping tool shows queue state, `gaps_opportunities.md`).

## 4. Clocks, precision and timing channels

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 4.1 | High Resolution Time's "coarsen time": 100 µs (or coarser) by default, **5 µs** with cross-origin isolated capability, "potentially jitter[ed]"; no concrete jitter rule | 100 µs / 5 µs | [HR-Time §4](https://w3c.github.io/hr-time/#dfn-coarsen-time) | read 2026-10-19 | High |
| 4.2 | The monotonic clock "never decreases" and exists only within one execution of the UA | — | [HR-Time §2.1](https://w3c.github.io/hr-time/#dfn-monotonic-clock) | read 2026-10-19 | High |
| 4.3 | MDN: the spec requires `performance.now()` to keep ticking during system sleep, but "only browsers on Windows keep ticking during sleep" | — | [MDN performance.now](https://developer.mozilla.org/en-US/docs/Web/API/Performance/now) | read 2026-10-19 | Medium |
| 4.4 | Vila and Köpf, "Loophole": shared event loops leak through queueing delay; a spy enqueues events and measures dispatch delay on Chrome's renderer main thread and the browser's I/O thread, enabling page identification, user-behaviour detection and covert channels "with high resolution and low overhead" (Distinguished Paper) | — | [USENIX Security 2017](https://www.usenix.org/conference/usenixsecurity17/technical-sessions/presentation/vila) | 2017-08 | Medium (abstract) |
| 4.5 | Schwarz, Maurice, Gruss, Mangard, "Fantastic Timers": new timing sources (including a `SharedArrayBuffer` counting worker and message passing) exceed the resolution of official timers by orders of magnitude in every major browser and in Tor Browser (whose timers were coarsened to 100 ms); "quick-fix mitigations can establish a dangerous false sense of security" | 5 µs official; Tor 100 ms | [FC 2017 paper](https://gruss.cc/files/fantastictimers.pdf) | 2017 | Medium (exact factors lost in text extraction) |
| 4.6 | Kohlbrenner and Shacham, "Trusted Browsers for Uncertain Times": implicit clocks defeat degraded explicit clocks, raising effective resolution by two orders of magnitude; proposes **Fermata**, a browser in which all timing sources are mediated, and **Fuzzyfox**, a Firefox fork with fuzzy time "with sufficient compatibility and performance for deployment today by privacy-sensitive users" | ×100 | [USENIX Security 2016](https://www.usenix.org/conference/usenixsecurity16/technical-sessions/presentation/kohlbrenner) | 2016-08 | Medium (abstract) |
| 4.7 | HGBA-2 I-6: `SharedArrayBuffer` and high-resolution timers only under COOP+COEP; Spectre confidentiality comes from process isolation | — | [HGBA-2 4.1][H2] | 2026-10-19 | High (Izcal doc) |

**Implications:** Coarsening the explicit clock does not close timing channels; an engine whose every clock and every scheduling decision passes through one mediated substrate (B1) is the Fermata design, and Izcal can state which channels remain (shared event loops of same-site agents only, because I-1 keeps sites apart).

## 5. Determinism, record and replay

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 5.1 | Replay.io aims for "effective determinism": replay need only reproduce JS state and DOM contents; pointer values may vary; hashtable iteration was made independent of pointers; GC and JIT may behave differently in replay; **finalizers** running at different points could diverge, so finalized objects were leaked during recording (works for recordings of a few minutes), with deterministic finalizer points planned | — | [Replay.io (Hackett)](https://www.replay.io/blog/effective-determinism) | 2021-09-14 | High |
| 5.2 | Playwright's clock overrides `Date`, `Temporal.Now`, timers, `requestAnimationFrame`, `requestIdleCallback`, `performance` and `Event.timeStamp` from script-level shims, which is evidence that test authors need clock control and that page-level shims are the only option today | — | [Playwright docs](https://playwright.dev/docs/clock) | read 2026-10-19 | High |
| 5.3 | Mugshot (Mickens, Elson, Howell, NSDI 2010) recorded JavaScript-visible non-determinism in the page itself (events, timers, `Date`, random) with a library and replayed it; it could not control the browser's own scheduling | — | NSDI 2010 paper (URL 404 at fetch time) | 2010 | Low (recollection) |
| 5.4 | Izcal's B1 already rests on DST evidence from FoundationDB, TigerBeetle and Antithesis; no published case applies DST to a browser engine | — | [HGBA-3 B1][H3] | 2026-10-19 | High (Izcal doc) |
| 5.5 | Izcal's JS engine makes every timing-dependent decision (tier install, GC completion and weak clearing, off-thread installs, pressure) a recorded B1 input (JS-D15), and FinalizationRegistry cleanup is posted on the host's seeded scheduler | — | [JS 6.9, 6.5][JS] | 2026-10-19 | High (Izcal doc) |
| 5.6 | Sources of non-determinism in a browser event loop, enumerated from the HTML processing model and the substrate: which task queue is chosen (implementation-defined); when a rendering opportunity occurs (vsync, throttling); timer firing and padding; IPC message arrival order across endpoints; network chunk boundaries; input event coalescing; parser yields; speculative parsing; GC and finalization; worker and service-worker start and termination; storage callbacks; `Atomics`/SAB races; clock reads | — | derived ([HTML](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model), [ST 6.4][ST]) | — | Medium |

**Implications:** Previous record/replay systems reproduced the page from outside the engine and had to leak or approximate engine-internal timing (5.1, 5.3). Izcal can make the event loop's own choices B1 inputs, which no published system has done.

## 6. Workers and other event loops

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | HTML defines window, worker and worklet event loops; each agent has one; worklet event loops have no rendering steps | — | [HTML event loops](https://html.spec.whatwg.org/multipage/webappapis.html#event-loops) | read 2026-10-19 | Medium (structure from TOC; details in `specs_semantics.md`) |
| 6.2 | `Atomics.wait` throws on agents whose `[[CanBlock]]` is false, which HTML sets for window agents | — | [ECMA-262 AgentCanSuspend](https://tc39.es/ecma262/#sec-agentcansuspend), [HTML](https://html.spec.whatwg.org/multipage/webappapis.html#integration-with-the-javascript-agent-formalism) | read 2026-10-19 | Medium |

## Open questions

- Q-EL-1. Does Chrome throttle rendering of offscreen cross-origin iframes, and with what published savings? Not fetched (search budget). Black-box check: rAF rate of an offscreen cross-origin iframe in each engine.
- Q-EL-2. `isInputPending`: shipped in Chrome 87; whether and why it was deprecated in favour of `scheduler.yield` was not verified this session.
- Q-EL-3. Safari's `requestIdleCallback` status in 2026 (row 1.9 rests on recollection).
- Q-EL-4. Per-engine `performance.now` precision and jitter in 2026: black-box measurement in the lab.
- Q-EL-5. Exact improvement factors in Fantastic Timers (digits lost in text extraction).
- Q-EL-6. Mugshot's overhead numbers (paper URL returned 404).

[H2]: ../../reports/HGBA%202%20browser%20architecture%20spec.md
[H3]: ../../reports/Future%20software%20browser%20engine.md
[JS]: ../../reports/Izcal%20JavaScript%20engine.md
[ST]: ../../reports/Izcal%20HTML%20and%20CSS%20parsing%20and%20style.md
[FP]: ../../reports/Izcal%20resource%20footprint.md
