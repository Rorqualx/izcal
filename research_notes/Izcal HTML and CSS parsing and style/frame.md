# Izcal HTML and CSS parsing and style: frame

Written 19 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Izcal needs its own HTML parser, CSS parser, selector matcher, cascade and computed-value pipeline, style invalidation and style data structures. HGBA-2 2.3.0 (M-1 part 1) excludes html5ever, Stylo, `selectors`, `cssparser`, lightningcss and every other library that does browser-engine work, so the Stage 1 design's "Stylo adapter" (Section 5.5) and "html5ever builds the DOM" (M2) are void. The owner's direction (19 October 2026): **engine work is all ours; no shortcuts for time or complexity; the best component from first principles.** Cost is a planning fact, never a reason for a lesser design. Staging is allowed only for a first-principles reason (correctness, security, measurement).

Scope of this spec:
- **HTML**: byte stream decoding and encoding sniffing (HTML 13.2.3), the WHATWG tokenizer and tree builder, fragment parsing (`innerHTML`, `insertAdjacentHTML`, `setHTMLUnsafe`, `parseHTMLUnsafe`, the sanitizer's `setHTML`), `document.write` and script-driven re-entrancy, speculative parsing and the preload scanner, streaming and incremental parsing, declarative shadow DOM, foreign content (SVG, MathML), XML/XHTML parsing hand-off (owner of the XML parser to be stated).
- **CSS**: CSS Syntax 3 tokenizer and parser (streaming, error recovery), property value grammars, at-rules, nesting, `@layer`, `@scope`, `@container`, `@media`, `@supports`, `@property`, `@font-face` (descriptors only; font loading belongs to the fonts/text spec), CSSOM data model.
- **Selectors**: Selectors 4 matching including `:has()`, `:is()`/`:where()`/`:not()`, nesting `&`, shadow-tree selectors (`:host`, `::slotted`, `::part`), pseudo-elements, `:visited` privacy rules, container queries' selector side.
- **Cascade and computed values**: origins, importance, layers, scope proximity, specificity, order; inheritance; custom properties and `var()`, `env()`, `attr()`, `if()`, registered properties; computed-value pipeline to the values layout consumes; animations and transitions as cascade inputs (the interface, not the animation engine).
- **Invalidation**: what restyles after a DOM mutation, state change, stylesheet change, media/container change; `:has()` invalidation; incremental vs from-scratch.
- **Data structures**: how parsed sheets, rule indexes, computed styles and per-element style data are stored, shared and discarded, sitting beside JS-D4's one-copy node columns.

Not in scope (sibling specs, being written in parallel): fonts and text shaping (fonts/text spec), networking and fetch (networking spec), layout and paint (future spec), the JS engine (done). Interfaces to them are stated here.

Who has the problem:
- **People browsing**: pages must render correctly and quickly; style and parsing are a large share of main-thread time on real pages; parser and style bugs have been exploited (use-after-free around script re-entrancy) and parser differentials enable XSS.
- **Web developers**: "why does this element have this value?", "why did this restyle?", "which selector is slow?", "why did `:has()` make the page slow?", "why did the parser put my element there?".
- **Izcal's engineers and agents**: the style system must be deterministic (B1, DET-1), checkable against a from-scratch reference (B5), parallel without changing results (phase purity), and must fit the footprint budgets (FP-11: DOM + style + layout + paint ≤ 512 B per node).

## What Izcal's specs already decide or constrain

- **HGBA-2 2.3.0** (`reports/HGBA 2 browser architecture spec.md`): P1–P6 precedence; **M-1** parts 1–4 (no engine libraries; spec anchors at pinned revisions; first-principles architecture; spec-anchored oracles, other engines only as black-box differential signals); I-1 (process per (site, container)); Section 4.3 (adaptive parallelism, bit-identical parallel and sequential output; its Stylo text is **superseded**); C-1 (WPT floors), ANC-1 (anchor coverage); S-4c superseded.
- **HGBA-3** (`reports/Future software browser engine.md`): B1 (deterministic substrate), B3 (two schedulers: event loop sequences, fork/join phases), B4 (data plane, restated by JS-D4), **B5** (specialized invalidation, early cutoff, from-scratch reference path, bulk fallback; its "keep Stylo's invalidation" text must be replaced), **B6** (shared-struct computed style; per-property SoA rejected as a non-goal), B8 (semantic plane reads style), DET-1, INC-1 to INC-4, FLUSH-1, D9.
- **Stage 1 design** (`reports/HGBA Stage 1 architecture design.md`): crate graph and rules G1–G9 (`hg-style` and `hg-tree-traits`; G1 dual implementation; G3 determinism; G9 anchors); 3.3 phase purity (style-sharing cache must not make results depend on thread assignment); 5.4 mutation log with old values; 5.5 Style (Stylo adapter, superseded); M2/M3 milestones; Style and layout track 3–4 engineers (sized for an adapter).
- **JavaScript engine spec** draft 0.3 (`reports/Izcal JavaScript engine.md`), owner-accepted: **JS-D4** one-copy DOM (node columns outside the cage, 32-bit slot plus generation; tag as static index or atom; flags and element state; attribute-set reference; inline-style reference; rare-data index; workers read the columns only during a flush with the script thread blocked; mutation log kept); **JS-D15** decision points are recorded B1 inputs; G6 and G13–G18 crate rules; text arena shared with JS as external strings; precise GC; release-by-ID finalization.
- **Resource-footprint spec** draft 0.3 (`reports/Izcal resource footprint.md`), owner-accepted: §8.2 budgets (FP-2 empty site process ≤ 4 MiB, FP-5 median page ≤ 32 MiB, FP-11 node columns ≤ 48 B, columns + rare + wrappers ≤ 72 B, DOM + style + layout + paint ≤ 512 B per node); §6.4 sharing keyed by partition (attribute sets, computed style groups, parsed style sheets); derived-data classes (a)/(b) and the safepoint handshake; **FP-D16** regenerable-cache class; engine image (UA style data lives there); FP-36 cross-partition channel suite.
- **DevTools spec** 0.3: T1 "why does this look like this", DT-D3 why-queries under replay, M3 Style domain, Q-DT-4 (Stylo's API) and the contingent cascade explainer, all of which change once the style system is Izcal's own.
- **Security scope** 0.3 (in review) and **coding standard** 0.3.1 (in review): Tier 1 fuzz surfaces, tiers, unsafe inventory, CS-PF-1 type-size asserts.

## What success looks like (measurable)

Candidate target dimensions, to become ST- targets:
- **Correctness**: html5lib-tests tokenizer and tree-construction 100%; WPT `html/syntax`, `html/dom`, `domparsing`, `css/css-syntax`, `css/selectors`, `css/css-cascade`, `css/css-nesting`, `css/css-contain` (container queries), `css/css-variables`, `css/css-properties-values-api`, `css/cssom` pass rates.
- **Security**: no memory-safety bug class reachable from markup or CSS; parser differential (mXSS) suite; depth and size limits with spec-conformant behaviour; DoS bounds on selector matching and `var()` expansion.
- **Performance**: parse throughput (MB/s) and first-byte-to-first-node latency; preload discovery latency; full style time and incremental restyle time per corpus page and per Speedometer 3 workload, relative to the best shipping engine; `:has()` invalidation cost; parallel scaling with bit-identical results.
- **Memory**: style bytes per element within FP-11's 512 B total; sharing hit rates; discard and rebuild bit-identical.
- **Determinism and verifiability**: incremental restyle equals from-scratch restyle on every frame in CI; parallel equals sequential; DET-1 hashes.
- **Visibility**: every computed value explainable to its winning declaration and spec step; every restyle attributable to its mutation.
