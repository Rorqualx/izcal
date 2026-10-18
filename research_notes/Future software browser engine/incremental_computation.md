# Incremental, Reactive and Query-Based Computation for a Browser Engine's Style/Layout/Rendering Pipeline (as of Oct 2026)

Status labels: **[shipped]** = in production use; **[prototype]** = working code, not production; **[research]** = paper / academic artifact.
Items marked "(background, not re-verified this session)" come from well-known literature. I did not re-fetch them during this research pass (tool budget), so the report writer should treat their details as lower-confidence.

---

## Q1. General incremental-computation frameworks: model, overhead, performance, production use

### Takeaway
Salsa is the most production-proven demand-driven query engine. It runs inside rust-analyzer. Its public record shows that memory, not time, is the dominant cost: in early 2025 the Salsa 3.0 migration left a residual regression of about 250 MB, plus about 300 MB saved by a single representation fix. Adapton-style general self-adjusting computation can be orders of magnitude slower than from-scratch recomputation unless the program is carefully structured with "names". Differential dataflow and Materialize prove that incremental view maintenance works at scale, but memory is the cost they keep working to reduce. I found no published per-node byte overhead for any of these frameworks.

### Cited Findings
**Salsa (rust-analyzer) [shipped]**
- Core model ("red-green" algorithm): each tracked function records its dependencies and the revisions in which they last changed. In a new revision Salsa re-validates the dependencies and returns the memoized value if none changed; otherwise it re-executes. — [Salsa book: algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- **Backdating:** if a re-executed function produces a value equal to the old one, downstream nodes treat it as unchanged. Example: a comment-only edit yields the same AST, so dependants don't re-run. This is directly analogous to "style recomputed but computed values unchanged → no relayout". — [Salsa book](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- **Durability:** inputs are tagged with a durability level (e.g., high for crates.io dependencies, low for the edited workspace). Salsa tracks the last revision in which each durability level changed, so a query that depends only on high-durability inputs can be validated without walking its edges. — [Salsa book](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- Evidence that durability matters (2020): fixing a durability misconfiguration for library crates in rust-analyzer cut validations for `CrateDefMapQuery` "from over 60k to about 7k" and its time "from about ~64ms to ~14ms". — [rust-analyzer commit (mirror)](https://git.joshthomas.dev/language-servers/rust-analyzer/src/commit/be967b94e15f238a6fe806e93359e302c43d0e45); see also [rust-analyzer issue #4712](https://github.com/rust-lang/rust-analyzer/issues/4712)
- Salsa 3.0 migration, Dec 2024: rust-analyzer's port passed its tests, but "performance/memory usage has a concerning regression". — [Salsa meeting notes (HackMD)](https://hackmd.io/@salsa/B19OUlA71l)
- Jan 2025: "Memory usage has gotten under control: it's now only a 250 megabyte regression." Enum representation changes saved about 300 MB. Making sync/memo tables global would save about 240 MB "but might harm parallelism". Open question: how GC should interact with interned values and non-default durabilities. — [Salsa meeting notes (HackMD)](https://hackmd.io/@salsa/SkNtN8Lv1e); [Salsa 3.0 planning](https://hackmd.io/@salsa/ry5ZGyBiA)
- The Salsa 3.0 planning page names memory as the main concern for rust-analyzer ("memory is too high. Working on fixing it"). — [HackMD](https://hackmd.io/@salsa/ry5ZGyBiA); [Niko Matsakis notes](https://hackmd.io/@nikomatsakis/HJHYSwgLR)

**Adapton / Nominal Adapton [research]**
- Nominal Adapton (Hammer et al., OOPSLA 2015) adds programmer-supplied "names" to allocations so that memoized results can be reused across changes. It is proven "from-scratch consistent". — [arXiv 1503.07792](https://arxiv.org/pdf/1503.07792); [OOPSLA 2015 page](https://2015.splashcon.org/event/oopsla2015-incremental-computation-with-names)
- "Nominal Adapton uniformly enjoys speedups over from-scratch computation (up to 10900×) as well as classic Adapton (up to 21000×)." Classic Adapton "is sometimes orders of magnitude slower than from-scratch computation", and is "over 10× slower than Nominal Adapton, even for medium-sized lists (10k elements)". Benchmarks were list and tree algorithms (map, filter, reduce, reverse, median, mergesort, quickhull). — [ar5iv 1503.07792](https://ar5iv.labs.arxiv.org/html/1503.07792)
- Adapton project page. — [cs.umd.edu/~hammer/adapton](https://cs.umd.edu/~hammer/adapton)

**Jane Street Incremental / Bonsai [shipped]**
- Incremental is Jane Street's OCaml library for self-adjusting computations. Incr_dom and Bonsai, their browser-UI frameworks, are built on it. — [opam: incremental v0.14.0](https://opam.ocaml.org/packages/incremental/incremental.v0.14.0/); [opam: bonsai](https://opam-4.ocaml.org/packages/bonsai/bonsai.v0.14.0)
- Bonsai's benchmark harness measures "incremental overhead" as total time minus incremental computation time. That is a harness metric, not a published per-node figure. — [bonsai bench profile.ml](https://ocaml.org/p/bonsai/latest/src/bonsai.bench/profile.ml.html)

**Differential dataflow / Materialize [shipped]**
- Shared arrangements (McSherry et al., VLDB 2020) are indexed, multi-versioned views of maintained state that concurrent queries can share instead of each rebuilding its own copy. The paper reports large gains in response time and resource use. — [Shared Arrangements (ar5iv 1812.02639)](https://ar5iv.org/html/1812.02639)
- Materialize keeps arrangements in memory, keyed for lookup, and new dataflows can reuse them without reprocessing their inputs. — [Materialize docs: arrangements](https://materialize.com/docs/lts/overview/arrangements)
- Memory is the cost centre. McSherry (2024) describes treating memory as a non-durable cache over cheap cloud storage and ongoing work to cut memory cost. — [Materialize and Memory](https://materialize.com/blog/materialize-and-memory/)
- Compaction keeps representations compact but "can't prevent unbounded growth if input is unbounded" (McSherry, 2020). — [Managing memory with differential dataflow](https://materialize.com/blog/managing-memory-with-differential-dataflow/)

**Build Systems à la Carte [research; the model describes shipped systems]** (background, not re-verified this session)
- Mokhov, Mitchell and Peyton Jones (ICFP 2018; extended JFP 2020) decompose build systems into a *scheduler* (topological, restarting, suspending) and a *rebuilder* (dirty bit, verifying traces, constructive traces, deep constructive traces). They map Make, Shake, Bazel, Excel and others into that grid. This vocabulary maps onto engines: a browser's dirty bits are a "dirty-bit rebuilder"; Salsa is a "suspending scheduler + verifying traces with early cutoff". — [Microsoft Research: Build Systems à la Carte](https://www.microsoft.com/en-us/research/publication/build-systems-la-carte/)

### Inferences
- The memory evidence from rust-analyzer suggests that per-query or per-node memo, dependency-edge and revision metadata is significant even with coarse queries per file or item. A browser with millions of DOM nodes × ~hundreds of style properties × several layout fields cannot afford one tracked node per (element, property). Tracking needs to be coarse-grained (per element, per style struct or per layout box) or kept in implicit, structural edges like tree parent/child.
- Durability and backdating are the two Salsa features that transfer most directly. Durability fits UA stylesheets, fonts and author stylesheets (rarely changing) versus DOM attributes and hover state (frequently changing). Backdating is "early cutoff" when computed style or box size is unchanged.
- Adapton's numbers show that *general* dynamic dependency tracking has a high constant factor. The wins come only when the program structure gives the tracker stable identities. A browser has natural stable identities (DOM nodes) in favour of keyed memoization.

### Gaps
- I found no published per-node or per-edge byte overhead for Salsa, Incremental, Adapton or differential dataflow. Absolute rust-analyzer memory totals under Salsa 3.0 in 2025–2026 were also not found; the meeting-note figures above are interim.
- I did not find Jane Street Incremental microbenchmarks (stabilization cost per node).
- Timely dataflow throughput and latency figures were not gathered in this pass.

---

## Q2. Signals / fine-grained reactivity: what is proven, and any engine-level use

### Takeaway
Signals are proven at the JS framework level and share design input from about a dozen frameworks. As standardization, though, the TC39 Signals proposal is still at Stage 1. I found no evidence that it advanced in the 2026 TC39 meetings. No browser engine uses signals internally for style or layout.

### Cited Findings
- The TC39 Signals proposal is at **Stage 1** (reached via the April 2024 agenda). The README says it "can currently be thought of as 'Stage 0'". The champions plan multiple production-grade polyfills and framework integrations before seeking Stage 2, and to be "especially conservative". Async and transactions are explicitly omitted. — [tc39/proposal-signals](https://github.com/tc39/proposal-signals)
- Design input came from Angular, Bubble, Ember, FAST, MobX, Preact, Qwik, RxJS, Solid, Starbeam, Svelte, Vue and Wiz. Champions include Daniel Ehrenberg, Yehuda Katz and Rob Eisenberg. — [tc39/proposal-signals](https://github.com/tc39/proposal-signals)
- Polyfill: [proposal-signals/signal-polyfill](https://github.com/proposal-signals/signal-polyfill). The README warns users not to rely on its stability, and estimates native availability at "at least 2-3 years at a minimum across browsers". — [tc39/proposal-signals](https://github.com/tc39/proposal-signals)
- Community summaries of the Jan, Mar and Jul 2026 TC39 meetings do not mention Signals advancing. This is absence of evidence, not confirmation. — [2026-01](https://scrapbox.io/petamoriken/2026-01_%E3%81%AE_TC39_meeting), [2026-03](https://scrapbox.io/petamoriken/2026-03_%E3%81%AE_TC39_meeting), [2026-07](https://scrapbox.io/petamoriken/2026-07_%E3%81%AE_TC39_meeting)
- Jane Street's Bonsai applies an Incremental-graph model to browser UI in production. That is framework level, on top of the DOM. — [opam: bonsai](https://opam-4.ocaml.org/packages/bonsai/bonsai.v0.14.0)

### Inferences
- Signals frameworks (Solid, Preact Signals, Angular) are push-pull dependency graphs with automatic tracking: push dirty marks, pull recomputation lazily. That is essentially the "double dirty bit" model plus dynamic dependency discovery. The proven scale is application state (thousands to tens of thousands of nodes), not millions of style or layout fields.
- If signals reach Stage 2+ the engine will need to implement them in its JS VM, but that is separate from using them internally.

### Gaps
- I did not fetch first-party benchmarks for SolidJS, Preact Signals or Qwik resumability in this pass (e.g., js-framework-benchmark results). The framework-level performance claims are therefore unsourced here.
- I found no source describing any browser engine using a signals-style graph internally for style or layout.

---

## Q3. Browser invalidation research and production practice

### Takeaway
Production engines (Blink, Gecko, WebKit) use hand-written, conservative invalidation: Blink invalidation sets for style, dirty and child-dirty bits plus constraint-keyed fragment caches for layout. Research shows the remaining traversal overhead is significant. Spineless Traversal (PLDI 2025) replaces the dirty-bit tree walk with an order-maintenance priority queue and gets about 1.8× mean on a Megatron layout fragment. Generating incremental layout engines from declarative specs (Meyerovich's attribute grammars, Medea, Megatron) is research-only and covers subsets of CSS.

### Cited Findings
**Spineless Traversal (Kirisame, Wang, Panchekha; PLDI 2025) [research]**
- Problem: the standard "Double Dirty Bit" algorithm, "used, with variations, in all major rendering engines", visits auxiliary (non-dirty) nodes to find dirty ones. These cause cache misses that stall layout for hundreds of microseconds. On the Google homepage example, a single dirtied node needs 66 auxiliary nodes. — [arXiv v4](https://arxiv.org/html/2411.10659v4)
- Approach: dirty fields go into a priority queue ordered by timestamps from an order-maintenance structure (Bender et al. 2002), so only dirty nodes are visited. Comparisons are branchless at "about five cycles". The queue and OM add roughly "3–5 L2 cache misses" (~100–200 cycles) per operation. — [arXiv v4](https://arxiv.org/html/2411.10659v4)
- Setup: traces from 50 real sites (Amazon, Wikipedia, GitHub, Google, Twitter, Discord, Lichess…), captured with a modified **Ladybird** browser, 2,216 frames. Layout is about 50 layout fields in about 700 lines of the Megatron DSL (box model, line breaking, display, position, intrinsic sizes, flexbox), "only a subset of HTML and CSS". Hardware: i7-8700K. — [arXiv v4](https://arxiv.org/html/2411.10659v4)
- **Headline numbers differ across versions:**
  - PLDI listing: faster on 83.0% of 2,216 benchmarks, mean speedup 1.80×. — [PLDI 2025 page](https://pldi25.sigplan.org/details/pldi-2025-papers/73/Spineless-Traversal-for-Layout-Invalidation); [DOI 10.1145/3729322](https://api.crossref.org/works/10.1145%2F3729322)
  - Earlier arXiv (v1/v3, Nov 2024): faster on 78.2%, mean 3.23×. — [arXiv v1](https://arxiv.org/abs/2411.10659v1); [arXiv v3 PDF](https://arxiv.org/pdf/2411.10659v3)
  - arXiv v4 body text: "1.78×" mean, with 25.14% of frames slower in Fig. 14. — [arXiv v4](https://arxiv.org/html/2411.10659v4)
  - Treat the PLDI figure (1.80×, 83%) as the published one, and note the discrepancy.
- Further results (v4): ≥2.35× speedup on the 60.7% of frames where fewer than 1% of fields are recomputed; 1.99× on the Twitter trace; lazy-loading (bulk insertion) frames about 2× *slower*; removals often 5×+ faster. Overhead is about a third of Spineless runtime. — [arXiv v4](https://arxiv.org/html/2411.10659v4)
- Stated limitations: flow-insensitive dependency analysis (over-dirties); assumes an acyclic static schedule; OM deletion is off by default (a memory leak on long-lived pages; enabling it costs about 2%). — [arXiv v4](https://arxiv.org/html/2411.10659v4)
- Megatron is a DSL plus compiler for layout with unboxing, a custom allocator and pointer compression. — [arXiv v1](https://arxiv.org/html/2411.10659v1)

**Spec-to-incremental-engine synthesis [research]**
- Medea (Liu, Chen, Atkinson, Feng, Bodík; PLDI 2023) synthesizes incremental layout engines from a separate specification and reports up to 1.82× over a naive incremental baseline. — [Medea PDF](https://sites.cs.ucsb.edu/~yanju/publications/pldi23-medea.pdf)
- Leo Meyerovich (Berkeley TR EECS-2013-242, 2013): layout specified as attribute grammars, with a compiler that generates parallel layout engines, statically verified traversal schedules, and SIMD/MIMD-optimized tree traversals. This is the work behind "Superconductor"; the name was not confirmed in the fetched abstract. — [Parallel Layout Engines: Synthesis and Optimization of Tree Traversals](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2013/EECS-2013-242.html)
- Panchekha group verification work (background, not re-verified this session): Cassius (formal CSS layout semantics in SMT, "Automated Reasoning for Web Page Layout", OOPSLA 2016), VizAssert (PLDI 2018, verifying accessibility layout properties) and Troika (modular layout verification, OOPSLA 2019). — [MIT CSAIL talk "Automated Reasoning about Web Page Layout"](https://www.csail.mit.edu/node/7382); NSF award listing [par.nsf.gov award 2122950](https://par.nsf.gov/search/award_ids:2122950)

**Blink style invalidation [shipped]**
- Blink computes a `RuleFeatureSet` from all stylesheet rules, which changes only when rules change. On a DOM mutation, `StyleEngine` schedules **invalidation sets**: criteria plus descend instructions. For `.c1 div.c2`, adding class `c1` schedules a pending set that invalidates `.c2` descendants. Invalidation never walks *up* the tree, so some invalidations are applied immediately and conservatively. — [Chromium: style-invalidation.md](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/core/css/style-invalidation.md)
- Pending sets are applied lazily by `StyleInvalidator::Invalidate` (depth-first) when style is read: "We don't want to do full style recalc at the time of adding/removing because there may be more mutations coming." The sets are deliberately conservative: "they err on the side of correctness, so we invalidate elements that do not need recalculation." — [Chromium: style-invalidation.md](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/core/css/style-invalidation.md)
- Sibling invalidation has a separate design doc. Pseudo-class sets (`:checked`, `:disabled`, `:target`, `:link`) were added in 2014 (CL 540533004), and type-selector sets around 2017. — [Chromium CL 540533004](https://codereview.chromium.org/540533004); [CL 2699883002](https://codereview.chromium.org/2699883002)
- `:has()` required new invalidation machinery. Its cost depends on selector shape (Igalia, 2023). — [Igalia: How Blink invalidates styles when :has() in use](https://blogs.igalia.com/blee/posts/2023/05/31/how-blink-invalidates-styles-when-has-in-use.html)

**LayoutNG caching [shipped] (article last updated 2021-10-08)**
- LayoutNG treats layout as a function of (style + DOM, parent constraints) and outputs an immutable **fragment tree** "designed to reuse large portions of the previous tree for incremental layouts". The parent constraint space is stored with each fragment as its **cache key**, and constraints are diffed in "a centralized place". Percentage sizes force relayout on an available-size change; fixed sizes do not. There are separate measure and layout caches. — [RenderingNG deep-dive: LayoutNG](https://developer.chrome.com/docs/chromium/layoutng)
- Problems the redesign fixed: under-invalidation bugs, "hysteresis" (non-idempotent layout: "It was incredibly difficult to ensure that layout was idempotent", fixed Chrome 93), and exponential two-pass layout (grid fixed in Chrome 93; caching brings it "back to O(n)"). No aggregate before/after numbers are published. — [RenderingNG deep-dive: LayoutNG](https://developer.chrome.com/docs/chromium/layoutng)
- Older (2017-era) code skipped the cache for results with out-of-flow positioned descendants and for fragmentation break tokens. — [crrev layout_ng_mixin.cc](https://crrev.com/3f840d763f0b03b8e1af7c65242f7cf0510aa237/third_party/blink/renderer/core/layout/ng/layout_ng_mixin.cc)

### Inferences
- LayoutNG is effectively a hand-rolled memoized pure function, `layout(node, constraint_space) → fragment`, with early cutoff. It is the closest production analogue to a Salsa query, but the cache is keyed on one constraint-space slot per node, not a general dependency graph. Its exceptions (OOF descendants, fragmentation) show where layout is *not* a clean tree function. Those are exactly the places a demand-driven graph would need non-tree edges.
- Blink's invalidation sets exist because selector matching is a many-to-many dependency (rule ↔ element) that is too expensive to track explicitly per element. A general dependency graph would have to either materialize these edges (memory blow-up) or reproduce invalidation-set-style summarization. That suggests style invalidation should stay a specialized, rule-indexed mechanism even in a query-based engine.
- Spineless Traversal shows that the traversal mechanism alone, given the same dependency semantics, is worth about 1.8×. A demand-driven engine with explicit dirty queues gets that benefit by construction, but pays OM and priority-queue cost on bulk insertions.

### Gaps
- Gecko/Stylo restyle hints and the "snapshot + selector-dependency" invalidation (Stylo's `invalidation` module) were not fetched in this pass.
- Current (2025–2026) LayoutNG cache rules, and Chromium's "simplified layout" pass, were not confirmed from primary sources.
- Megatron does not appear to have its own standalone paper; it is described only inside the Spineless Traversal paper.

---

## Q4. Overhead risks: when does incremental lose?

### Takeaway
The evidence consistently shows incremental machinery losing on bulk changes and when tracking granularity is too fine. Spineless loses on about 17–25% of frames, especially lazy-loading insertions. Classic Adapton is sometimes orders of magnitude slower than from-scratch. Salsa's main production problem is memory (hundreds of MB of regression during the 3.0 port).

### Cited Findings
- Spineless Traversal is slower on bulk-insertion (lazy-loading) frames (about 2×) and slower overall on 17–25% of frames depending on version. — [arXiv v4](https://arxiv.org/html/2411.10659v4); [PLDI 2025 page](https://pldi25.sigplan.org/details/pldi-2025-papers/73/Spineless-Traversal-for-Layout-Invalidation)
- Classic Adapton "is sometimes orders of magnitude slower than from-scratch computation". — [ar5iv 1503.07792](https://ar5iv.labs.arxiv.org/html/1503.07792)
- Salsa 3.0: a 250 MB memory regression remained in rust-analyzer in Jan 2025 after optimizations. Global memo tables trade memory for parallelism. — [HackMD](https://hackmd.io/@salsa/SkNtN8Lv1e)
- Wrong durability (graph too large to validate) cost about 4.5× on one rust-analyzer query (64 ms → 14 ms after the fix). — [rust-analyzer commit](https://git.joshthomas.dev/language-servers/rust-analyzer/src/commit/be967b94e15f238a6fe806e93359e302c43d0e45)
- Differential dataflow state grows without bound under unbounded input unless compacted. — [Materialize blog](https://materialize.com/blog/managing-memory-with-differential-dataflow/)
- Blink deliberately over-invalidates rather than tracking precise dependencies, citing the cost of ancestor checks. — [style-invalidation.md](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/core/css/style-invalidation.md)

### Inferences
- An engine design needs a "give up and recompute subtree" fallback, like Blink's `kSubtreeStyleChange`, plus heuristics to switch modes. Initial page load, large `innerHTML` and stylesheet swaps are batch workloads, where a from-scratch parallel traversal beats graph maintenance.
- Verification cost scales with the number of edges validated, not just the nodes recomputed. Durability-like layering is essential at browser scale.

### Gaps
- No direct measurement exists of a general framework (Salsa, Adapton) running browser-scale style or layout with millions of nodes. The closest is Spineless/Megatron, a specialized compiler output, not a general framework.

---

## Q5. UI / browser engines built on general incremental frameworks

### Takeaway
No production browser engine is built on a general incremental-computation framework. UI toolkits use specialized reconciliation (retained trees + diffing + dirty flags) rather than general dependency graphs. Xilem/Masonry (Linebender) is the most relevant Rust research-and-development effort, and it is still experimental.

### Cited Findings
- Xilem is "an experimental architecture" over Masonry. It diffs lightweight, strongly-typed view objects against a retained widget tree, with ideas from Flutter, SwiftUI and Elm. Masonry uses winit (windowing), Vello/wgpu (rendering), Parley (text) and AccessKit (accessibility). xilem 0.4.0 is on docs.rs. **[prototype]** — [github.com/linebender/xilem](https://github.com/linebender/xilem); [docs.rs xilem 0.4.0](https://docs.rs/crate/xilem/0.4.0)
- Linebender describes Xilem as an R&D project, experimental since May 2022, following earlier efforts (Druid). — [Roadmap for the Xilem backend in 2024](https://linebender.org/blog/xilem-backend-roadmap/); [Xilem 2024 plans](https://linebender.org/blog/xilem-2024/)
- Jane Street Bonsai and Incr_dom run browser UIs in production on the Incremental library, as a framework above the DOM rather than as an engine. **[shipped]** — [opam: bonsai](https://opam-4.ocaml.org/packages/bonsai/bonsai.v0.14.0)
- Spineless Traversal's prototype used Ladybird for trace capture plus a compiled Megatron layout core. This is the closest thing to a "browser layout engine generated from a dependency specification". **[research]** — [arXiv v4](https://arxiv.org/html/2411.10659v4)

### Inferences
- The industry pattern (LayoutNG, Flutter, React Fiber, Compose, Xilem) is: immutable or declarative inputs → pure-ish function → retained output + structural diff + local caches keyed on inputs. That is "memoization with early cutoff" specialized to trees, not a general graph. A first-principles engine can get most of the demand-driven benefit by making each pipeline stage a pure function over explicit inputs (LayoutNG-style constraint-keyed caches, Salsa-style durability and backdating), with specialized invalidation for selector matching and dirty-queue scheduling (Spineless-style) for layout. A general per-field dependency graph looks too costly in memory.
- WPT conformance risk: general-graph approaches have only been demonstrated on CSS subsets (Megatron about 700 lines, Medea, Meyerovich). Features that break tree-locality (OOF positioning, fragmentation, `:has()`, container queries, anchor positioning) are where the research coverage stops.

### Gaps
- I did not fetch first-party measured results for Flutter element/render-tree rebuild costs, React Fiber, Jetpack Compose's snapshot/recomposition system, or Druid. Their mechanisms are well known, but they are unsourced in these notes.
- I found no Linebender posts from 2025–2026 in this pass; Xilem's current maturity should be checked on GitHub.
- Search found no browser or engine project (e.g., Servo, Ladybird, Blitz) documenting a switch to Salsa or similar for style/layout.
