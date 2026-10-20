# Izcal DOM, bindings and event loop: Izcal interfaces (internal digest)

Compiled 19–20 October 2026 from Izcal's own documents; no web sources and no engine source code. Written by the synthesis author: the researcher assigned to this digest did not finish, and the design critique (`critique.md`) cross-checked the expectations below against the cited specs. Section references are to each spec's revision current on 20 October 2026 (JS 0.3.1, FP 0.3.1, ST 0.3.3, LY 0.4, GR 0.4, NW 0.3.3, MD 0.4, TX, AX 0.2, CB 0.3, DT 0.3, Stage 1 design 0.11, HGBA-2 2.3.2, HGBA-3, Phase 3 ledger).

Confidence: every row was read in the cited document by the synthesis author or by the critic, and is **High** unless marked.

## 1. HGBA-2, HGBA-3 and the Stage 1 design

| # | Decision or expectation | ID(s) | Where | Consequence for the DOM spec |
|---|---|---|---|---|
| 1.1 | No code from another engine, no library doing engine work; MIT-only shipped code and data; another engine's source code never evidence; spec-anchored implementation at pinned revisions | M-1 (2.3.2), ANC-1 | HGBA-2 Section 2 | Own IDL compiler, DOM, event loop, navigation; anchors on every algorithm and generated item |
| 1.2 | One locked renderer per site on desktop; (site, container) key; origin keying of subdomains; documented reduced-isolation mode below about 2 GB | I-1 and its 2.1.2 extensions | HGBA-2 4.1 | Agent clusters must fit inside processes; reduced isolation needs explicit handling |
| 1.3 | Zero ambient authority; SAB and high-resolution timers only under COOP+COEP | I-2, I-6 | HGBA-2 4.1 | Broker decides; timer coarsening |
| 1.4 | bfcache hit rate target; one memory policy for bfcache and discard | BF-1 | HGBA-2 Section 6 | DM-31, DM-D14 |
| 1.5 | Deterministic substrate: no direct OS clock, thread, randomness; simulation with seeded scheduler and virtual clock | B1 | HGBA-3 | Scheduler picks and clocks must be functions of recorded inputs |
| 1.6 | Typed capability IPC, sequence affinity, sync calls only on an allowlist | B2; Stage 1 4.1 | HGBA-3; Stage 1 | Message ports as endpoints; `localStorage` and `document.cookie` leave the sync allowlist |
| 1.7 | One serial sequence per agent cluster for the event loop; fork/join for phases | B3 | HGBA-3 | One sequence per cluster |
| 1.8 | Semantic plane with stable IDs; agent principal; AI off by default | B8, B9, B10 | HGBA-3 | Generational node IDs; agent world only with feature `ai` |
| 1.9 | Process map: broker owns navigables, session history, frame tree, site-to-process map | Stage 1 Section 1 | Stage 1 | `hg-navigation`, `hg-agents` in the broker |
| 1.10 | `hg-navigation` from HTML's navigables and session history, pure functions over IPC events | Stage 1 4.3 | Stage 1 | Kept and divided (DM 6.9) |
| 1.11 | Script thread and event loop; DOM in `cppgc`; WebIDL bindings with `weedle` and a resolver, `webidl2.js` as CI oracle; data plane and mutation log | Stage 1 5.1–5.4 | Stage 1 | 5.2 and 5.4 replaced by JS-D4; 5.1 and 5.3 by this spec |
| 1.12 | Crate rules G1–G9 (G3 clocks only in `hg-substrate-os`; G4 AI crates not linked by default; G5 unsafe inventory; G9 anchors) | G1–G9 | Stage 1 2.2 | This spec's rules G90–G99 sit beside them |
| 1.13 | "JS, bindings and DOM" track of 5–6 engineers covering V8 embedding, codegen, DOM, event loop, web APIs | Stage 1 8.3 | Stage 1 | Replaced by this spec's tracks (9.3), including web APIs |

## 2. JavaScript engine spec

| # | Decision or expectation | ID(s) | Where | Consequence |
|---|---|---|---|---|
| 2.1 | One copy of the DOM: node columns outside the cage (five links, tag, flags and state, attribute-set and inline-style references, rare-data index, generation; 48 B†), traced as `hg-heap`'s node space; lazy wrapper held strongly once created; text arena shared with JS as external strings; mirror retired; mutation log kept | JS-D4 | JS 6.2, 9.4 | The DOM is designed on these columns; no second copy |
| 2.2 | Release by ID at event-loop points between tasks, with `&mut ScriptToken`; no `Drop` on GC objects | — | JS 6.5 | Slot reuse protocol |
| 2.3 | Two tokens: `Mutator` for GC references, `ScriptToken` for state script can reach (`TokenCell`) | CS-D3 | JS 6.5 | The re-entrancy discipline builds on `ScriptToken` |
| 2.4 | Timing-dependent decisions are recorded B1 inputs; production never waits | JS-D15 | JS 6.9 | The decision-point register |
| 2.5 | One cage per process, agent heaps in own blocks; every handle carries its heap's brand | JS-D17, JS-D18 | JS 6.1, 6.5 | Several agents may share a thread soundly |
| 2.6 | Host exotic objects (`WindowProxy`, `Location`, legacy platform objects, `[[IsHTMLDDA]]`) are shapes with host hooks; `hg-webidl-codegen` emits shape families and IC generators | — | JS 6.2 | Bindings emit `.hgop` |
| 2.7 | `hg-js-host`: 16 host hooks, jobs owned by the host, realms and settings objects, modules, typed serializer interface for structured clone | — | JS 6.14 | `hg-script` implements the host side |
| 2.8 | Window agents of one (site, container) in one process may share the script thread | — | JS 6.12 | DM-D19 changes it (amendment) |
| 2.9 | Same-site verified bytecode shared across agent heaps of a process, with reference counts released through release lists | — | JS 6.11 | Becomes atomic under DM-D19 (amendment) |
| 2.10 | Only `hg-dom`, `hg-bindings`, `hg-script`, `hg-inspect-renderer` depend on `hg-js-host` and `hg-heap` | G6 (rewritten) | JS 6.15 | Other crates use `hg-script-api` |
| 2.11 | G13–G18 (L2e layering, jit feature, unsafe in `hg-heap` only, generated crates from `.hgop`, H-2 lint, checked arithmetic) | G13–G18 | JS 6.15 | Clone decoder follows G18's rule |
| 2.12 | T0 per ISA: x86-64, AArch64; RISC-V 64 later | — | JS 6.3 | Owner decision of 19 October 2026 makes riscv64 a target from the start (amendment) |
| 2.13 | Critical path: the DOM and bindings track cannot generate interfaces until `hg-js-host` exists; mitigation: design and freeze it early | — | JS 9.2 | Mock host from DM-M0 |

## 3. Resource footprint spec

| # | Decision or expectation | ID(s) | Where | Consequence |
|---|---|---|---|---|
| 3.1 | One process per (site, container), split on a budget only at agent-cluster boundaries | FP-D3 | FP 9.3 | Whole clusters per process |
| 3.2 | Origin keying on every desktop class; no trigger turns it off | FP-D8 | FP 9.3 | DM-D6 recommends origin-keyed clusters always |
| 3.3 | Hidden ⇒ frozen after 30 s† unless a declared reason; a frozen process gets no scheduled wakeups | FP-D4 | FP 6.8 | Broker-decided group freeze; inputs held outside the renderer |
| 3.4 | Node columns ≤ 48 B†; columns + rare data + wrappers ≤ 72 B† per node across every copy; DOM + style + layout + paint ≤ 512 B† | FP-11 | FP 8.1 | DM-30 |
| 3.5 | Per realm ≤ 64 KiB† with untouched intrinsics; per further agent ≤ 768 KiB† | FP-4, FP-34 | FP 8.1 | DM-32, DM-26 |
| 3.6 | Idle empty site process: 2 threads; agent threads (dedicated workers) reported per page | FP-32 | FP 6.3, 8.1 | DM-33 |
| 3.7 | bfcache budgets in bytes (a share of RAM, a cap, a per-(site, container) line); evicted before live tabs | — | FP 6.10 | DM-D14's last row |
| 3.8 | Suspend to disk with bfcache-like eligibility (no SAB shared with another agent; dedicated workers frozen with the page) | — | FP 6.9 | bfcache worker row aligned |

## 4. HTML and CSS parsing and style spec

| # | Decision or expectation | ID(s) | Where | Consequence |
|---|---|---|---|---|
| 4.1 | One HTML parser core with sinks; `DomSink` writes `hg-dom`'s columns on the script thread | ST-D1 | ST 6.2 | `hg-dom` API for the sink |
| 4.2 | Parser call-outs return `Yield` values; nothing held across them; node IDs with generations | — | ST 6.3 | Same discipline as DM 6.2 |
| 4.3 | Parser yields and speculation are recorded decision points | ST-D2 | ST 6.4 | Register rows |
| 4.4 | Mutation events and XSLT not implemented | ST-D4 | ST 6.3 | DM-D15 |
| 4.5 | Style runs at "update the rendering" and forced flushes with the script thread blocked; the environment snapshot is one per document per frame, and its changes are mutation-log entries | — | ST 6.10 | `SetEnvironment` record kind |
| 4.6 | 32 bits of element state in the column; style writes its own columns; slot reuse notifies style | — | ST 6.11 | Column layout; release protocol |
| 4.7 | Attribute sets and parsed sheets hash-consed per partition; interned style data per agent | — | ST 6.11 | Per-cluster tables under DM-D19 (amendment) |
| 4.8 | `hg-cssom` (L3) owns CSSOM; G19–G24 (script-free style, one parser per grammar, no content-depth recursion) | G19–G24 | ST 6.15 | G98 extends G21's rule |

## 5. Layout, graphics, networking, media, text

| # | Decision or expectation | ID(s) | Where | Consequence |
|---|---|---|---|---|
| 5.1 | Layout runs in "update the rendering" and at forced flushes; documents processed container first, children in tree order | — | LY 6.1 | Rendering task iterates in that order |
| 5.2 | `hg-layout-query` owns CSSOM View, ResizeObserver, IntersectionObserver, the forcing table; scroll offsets primary state in `hg-scroll` | — | LY 6.14–6.15 | Ownership table |
| 5.3 | Scroll-state snapshot after the ResizeObserver loop; last remembered sizes | — | LY 6.9 | Row in the rendering-steps table |
| 5.4 | Single-engine features need an owner exception (LY-D10) | LY-D10 | LY 6.15 | DM-D15 exceptions |
| 5.5 | The compositor is the frame scheduler and sends BeginFrame; throttling is process-independent | GR-D15 | GR 6.9 | Rendering opportunities |
| 5.6 | Compositor scrolling per scroll node when no non-passive wheel or touch listener is in the path; listener timeout open | Q-GR-20 | GR 6.8 | Flag through property trees |
| 5.7 | Canvas contexts and OffscreenCanvas are GR's | — | GR 6.15 | Element plumbing only |
| 5.8 | Partition service owns service-worker routing; renderer constructs requests (Request, Headers, body streams) | — | NW 6.7 | Web-API track builds the renderer side |
| 5.9 | `document.cookie` without sync IPC: a versioned read-only region | NW-D10 | NW 6.9 | DM-D11 follows the pattern |
| 5.10 | Image selection algorithms are HTML's, in `hg-dom`; media decodes | — | MD 6.20 | `hg-html` owns `img` selection |
| 5.11 | Autoplay gated on transient activation | MD-D13 | MD | Broker activation check |
| 5.12 | Opaque responses never readable in renderers, including through service workers and the Cache API | MD-31, G56 | MD | Opaque handles in SW and Cache |
| 5.13 | Font installations are mutation-log entries; font timeline in DevTools | — | TX 6.8, 9.5 | `InstallFont` record kind |

## 6. Accessibility, content blocking, DevTools

| # | Decision or expectation | ID(s) | Where | Consequence |
|---|---|---|---|---|
| 6.1 | The semantic tree is computed on a semantic thread from sealed mutation-log segments, style semantic records and fragment epochs | AX-D3 | AX 6.3 | Sealed segments; extra record kinds |
| 6.2 | One IDREF resolver in `hg-dom` for element reflection and the semantic plane | — | AX 6.2 | `hg-dom` owns it |
| 6.3 | AT actions only through the input-equivalence table of AX 6.9, in the input crate the DOM spec names; no script inside an IPC handler | G106, AX-24 | AX 6.9 | `hg-input::equivalent` |
| 6.4 | No crate reachable from `hg-dom` or `hg-script` can read the semantic level | G105 | AX 6.17 | Dependency check |
| 6.5 | `ariaNotify` is constant-time, with no return value, travelling with the next update | — | AX 6.7 | Not a task-source event |
| 6.6 | Caret browsing (F7) is engine-provided by AX | — | AX 6.12 | Over the Selection model |
| 6.7 | Restriction profiles per script (cookie-write cap, CPU and wakeup budget) and creator inheritance through `eval`, `Function`, string timers, `javascript:` URLs, inserted inline scripts, same-origin child documents | I5, CB-27 | CB 6.6, 8 | Always-on attribution ID |
| 6.8 | `$popup` blocks a popup before the new top-level traversable exists | — | CB 6.2 | Broker check in `hg-navigation` |
| 6.9 | `hg-inspect-renderer` reads DOM and data-plane state, mutation taps with principal and JS stack; script only through `hg-script` | — | DT 6.x | `hg-dom-observe` feeds it |
| 6.10 | Recorded units run in dedicated, freshly launched processes, workers outside; execution points count inspector pauses | — | DT 6.x | Placement exception; *paused in debugger* state |
| 6.11 | DevTools' 3 engineers for the `hg-script` inspector interface counted in DT | — | DT 9.2 | Counting rule in 9.3 |

## 7. Phase 3 ledger

| # | Item | Where | Consequence |
|---|---|---|---|
| 7.1 | G-11: DOM, WebIDL bindings and event loop has no spec; blocks Stage 1 5.1–5.4 | Ledger, components with no spec | Closed by this spec |
| 7.2 | G-12 SVG, G-13 animations, G-16 editing, G-17 accessibility, G-18 WebCrypto, G-19 storage, G-20 extensions | Ledger | Ownership boundaries in DM 6.18 |
| 7.3 | G-4: riscv64 in the JS spec | Ledger | Owner decision 19 October 2026: target from the start |
| 7.4 | A-S1-35, A-S1-36: Stage 1 5.2 and 5.4 one-copy DOM; 5.3 bindings target `hg-js-host` | Ledger | Already applied in JS's amendment rows |

## Conflicts and gaps found

- JS 6.12's shared script thread for same-(site, container) agents conflicts with per-cluster threads (DM-D19); ST 6.11's per-partition interning and JS 6.11's shared bytecode counts assume one script thread per process.
- FP-D8 (origin keying everywhere, no exception) conflicts with the WHATWG default of site-keyed agent clusters and with honouring `Origin-Agent-Cluster: ?0`.
- Stage 1 8.3's "web APIs" had no owner once the DOM spec replaced that track; WebCrypto stays ledger G-18.
- The accessibility spec's sealed segments and record kinds were not in JS-D4's mutation-log description.
