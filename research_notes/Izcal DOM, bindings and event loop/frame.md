# Izcal DOM, bindings and event loop: frame (phase 1)

Compiled 19 October 2026 by the synthesis author from Izcal's own documents. No web sources; no engine source code.

## 1. The problem and who has it

| User | Task | What the component must give them |
|---|---|---|
| People browsing | Pages that respond, keep their state on back and forward, do not drain the battery and cannot reach other sites' data | A scheduler that puts input and rendering first; bfcache and freeze that keep state; no cross-origin read through a binding, WindowProxy or `Location` bug |
| Web developers | "Why did this task run late?", "which script changed this attribute?", "why is this element leaking?", "why did this event not reach my listener?", "why is my test flaky?" | Observable task queues with causes; mutation provenance; retainers for nodes and listeners; exact replay of event-loop interleavings |
| Site authors of large apps | Thousands of DOM operations per frame, web components, forms, workers, messaging | DOM operations that are as cheap as a property access; a cheap worker and a cheap `postMessage` |
| Izcal's own UI (shell, DevTools, prompts) | HTML applications rendered by Izcal itself (D-S2, D-S11) | The same DOM and event loop, in jitless renderers, with fast start |
| Izcal's engineers and agents | Generate tens of thousands of binding members from the specs, keep them correct as the specs move, reproduce any bug | One IDL compiler, pinned IDL, anchors on every generated item, recorded scheduler decisions |
| Agents (B9), only with AI on | Act on a page through stable node IDs and the semantic plane | Stable generational node IDs; mutation subscriptions; deterministic waits |

## 2. What Izcal's specs already decide or constrain

- **HGBA-2 2.3.2.** M-1 (all four parts): Izcal builds its own DOM, bindings and event loop from WHATWG DOM, HTML and WebIDL at pinned revisions, with spec anchors (ANC-1); no other engine's code, and no library doing engine work. P1 > P2 > P3 > P4. I-1: one locked renderer process per (site, container) on desktop; origin keying above a RAM threshold; a documented reduced-isolation mode below about 2 GB. I-2 zero ambient authority. I-6: `SharedArrayBuffer` and high-resolution timers only under cross-origin isolation. BF-1: bfcache hit rate, one memory policy for bfcache and discard.
- **HGBA-3.** B1 (deterministic substrate, recorded inputs, seeded scheduler, virtual clock), B2 (typed capability IPC, sequence affinity), B3 (one serial sequence per agent cluster for the event loop; fork/join for phases), B4 as restated by JS-D4 (the node columns are the data plane; mutation log with old values), B5 (incremental core), B8 (semantic plane, stable node IDs), B9 (agent principal), B10 (AI off).
- **Stage 1 design (to be redesigned in phase 4).** 4.3 `hg-navigation` in the broker from HTML's navigables and session history; 5.1 script thread and event loop; 5.2 (`cppgc` DOM, superseded by JS-D4); 5.3 `hg-webidl-codegen` (weedle plus resolver, `webidl2.js` as a CI oracle; superseded in part by JS 9.5 and the MIT/engine-work rules); 5.4 data plane (superseded by JS-D4). G1–G9.
- **JavaScript engine spec 0.3 (accepted).** JS-D4: one copy of the DOM, node columns outside the cage, traced as `hg-heap`'s node space, lazy wrapper held strongly once created, text arena shared with JS as external strings, release-by-ID at event-loop points. JS-D15: decision points are recorded B1 inputs. JS-D17: one cage per process, agent heaps in their own blocks. JS-D18: branded handles. 6.2: host exotic objects (legacy platform objects, `WindowProxy`, `Location`, `[[IsHTMLDDA]]`) are shapes with host hooks; `hg-webidl-codegen` emits shape families and IC generators per interface. 6.14: `hg-js-host` (16 host hooks, jobs belong to the host, realms and settings objects, modules, typed serializer interface for structured clone). G6: only `hg-dom`, `hg-bindings`, `hg-script`, `hg-inspect-renderer` may depend on `hg-js-host` and `hg-heap`'s public API.
- **Resource footprint (accepted).** FP-11's 48 B† column core and DOM's 72 B† per node; FP-D3 never splits an agent cluster; FP-D4 hidden ⇒ frozen after 30 s†; suspend to disk; bfcache in bytes; per-agent costs (FP-34).
- **HTML and CSS parsing and style (accepted).** The tree builder writes `hg-dom`'s columns through `DomSink` on the script thread; parser call-outs return `Yield` values; parser yields are recorded decision points; `hg-cssom` owns CSSOM; style's state sources write through `hg-dom` on the script thread; 32 bits of element state in the column.
- **Other accepted specs** (layout, graphics, media, networking, fonts and text, content blocking): see `izcal_interfaces.md`.

## 3. Success, in terms someone can measure

- WPT `dom/`, `html/` (DOM-side directories), `webidl/`, `workers/`, `service-workers/`, `navigation-api/` and `idlharness` pass rates at or above the best shipping engine's black-box rate per directory, non-decreasing.
- Every generated binding item and every hand-written DOM and event-loop algorithm carries a spec anchor (ANC-1).
- Zero cross-origin read through bindings, WindowProxy or `Location` in a planted-bug and fuzzing campaign; every re-entrancy class closed by construction (types), not by review.
- DOM bytes per node inside FP-11; binding call overhead measured against a property access.
- Input-to-next-paint and long-task tails on the corpus at or better than the best shipping engine; zero main-thread waits on background work in production.
- Every event-loop interleaving of a recorded session replays exactly (DET-1), and a seeded scheduler explores the interleavings HTML permits.
- bfcache hit rate per BF-1.

## 4. Strands chosen for phases 2–3

history_postmortems, specs_semantics, bindings_object_model, event_loop_scheduling, navigation_agents_workers, security_robustness, gaps_opportunities; plus `izcal_interfaces.md`, an internal digest of what Izcal's documents decide about or expect from this component.
