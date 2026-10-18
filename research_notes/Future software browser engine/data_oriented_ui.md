# Data-oriented design, ECS and GPU-driven UI architectures: implications for a browser engine's DOM, style and layout data models (as of Oct 2026)

Status labels: **[shipped]**, **[prototype]**, **[research]**. Research date: 2026-10-14. Budget-limited pass (~16 tool calls). Several sub-topics (Flutter, Slint, egui, Makepad, Skia Graphite, WebRender) were not verified this session and are listed under Gaps instead of being stated from memory.

## Q1. Modern non-browser UI/rendering stacks (GPUI, Vello, Taffy, Parley, Linebender, Bevy UI, others): architecture, data layout, performance evidence

### Takeaway
The Rust UI ecosystem has settled on shared, swappable parts: Taffy for layout, Parley for text, and Vello / Vello CPU / Vello Hybrid for rasterization. Each part uses index- or ID-based storage instead of object graphs. The published performance evidence comes mostly from the projects themselves: microbenchmarks such as Taffy vs Yoga, and design claims such as GPUI reaching 120 FPS. I found no independent, controlled comparisons against browser engines.

### Cited Findings
**Zed GPUI [shipped, inside Zed]**
- GPUI was built because Zed's earlier Atom-era rendering approach was too slow for drawing simple boxes and glyphs. Zed chose a custom shader per primitive over a general-purpose 2D library. — [Zed blog: Leveraging Rust and the GPU to render UIs at 120 FPS](https://zed.dev/blog/videogame)
- GPUI reduces the UI to a few primitives: rectangles (drawn with signed distance fields), shadows, text, icons and images. The CPU describes each primitive's properties as data, and the GPU draws them in parallel. — [zed.dev/blog/videogame](https://zed.dev/blog/videogame)
- A February 2024 follow-up post, "Optimizing the Metal pipeline to maintain 120 FPS in GPUI", was prompted by frame drops on newer MacBooks that the team diagnosed with the Metal HUD. — [Zed performance tag](https://zed.dev/blog/tagged/performance)
- GPUI uses Taffy for layout. — [Taffy README](https://github.com/DioxusLabs/taffy)

**Vello family (Linebender) [Vello classic: shipped as a crate; Vello Hybrid: beta; Vello CPU: released crate, pre-1.0]**
- The sparse-strips effort is a new Vello implementation. It targets a wider range of devices, including GPUs without compute shaders, and aims to cut performance cliffs and memory pressure. It follows Raph Levien's "Potato" hybrid CPU/GPU design and uses tiling, sorting and sparse strip allocation. The README in these mirrored revisions says it is "not yet suitable for production". — [vello sparse_strips README (mirror)](https://skia.googlesource.com/external/github.com/linebender/vello/+show/de2b7514d80964598a2ebc99ff209b1fbb291d92/sparse_strips/README.md)
- `vello_hybrid` keeps path processing on the CPU and sends fine rasterization to the GPU. There is a WebGL backend, with WGSL-to-GLSL compilation in `vello_sparse_shaders`. — [vello_hybrid tree](https://skia.googlesource.com/external/github.com/linebender/vello/+/bf5b0f968356bec23fefe76e715e2f91502893eb/sparse_strips/vello_hybrid)
- Linebender's Q1 2026 report gives the latest releases as sparse strips 0.0.7 (Vello CPU and Vello Hybrid) and Vello 0.8.0. It says Vello Hybrid is "roughly beta quality" with "rough edges still and performance work to be done". Optimizations listed: fast paths that skip full coarse rasterization, special-cased rectangles, a first cut at glyph caching, and fixes for heavy gradients. A planned unified "Vello API" covering both CPU and Hybrid "proved harder than expected". Glifo (formerly parley_draw), an atlas-based glyph cache, now lives in the Vello repo. — [Linebender in 2026 Q1](https://linebender.org/blog/tmil-25/)
- Work continued through mid/late 2026, for example an integer fast path for rectangle clips (Aug 2026) and a GPU fast-rect path under rectangular clips. — [vello#1786](https://github.com/linebender/vello/pull/1786); [vello#1926](https://github.com/linebender/vello/pull/1926)
- There is an open report of Vello Hybrid rendering old.reddit.com/r/rust slowly, so browser-like workloads still hit performance problems. — [vello#1380](https://github.com/linebender/vello/issues/1380)

**Taffy [shipped, widely embedded]**
- Taffy implements CSS Block, Flexbox and CSS Grid. — [Taffy README](https://github.com/DioxusLabs/taffy)
- Projects it powers: Servo, Blitz, Bevy, Takumi, iocraft, Slint, Lapce (via Floem) and Zed (via GPUI). There are Python and work-in-progress C/WASM bindings. — [Taffy README](https://github.com/DioxusLabs/taffy)
- Servo uses Taffy only for CSS Grid. This comes from a search-result summary of the blitz-html crate page and was not re-verified. — [lib.rs blitz-html](https://lib.rs/crates/blitz-html)
- Storage model: `TaffyTree` is built on `slotmap::SlotMap` and `SecondaryMap`, keyed by `NodeId`. Taffy also exposes trait-based tree access (`LayoutPartialTree`), so embedders can run Taffy's algorithms over their own tree. Blitz does this. — [taffy/src/tree/taffy_tree.rs](https://github.com/DioxusLabs/taffy/blob/main/src/tree/taffy_tree.rs)
- Benchmarks vs Yoga 3.2.1 (M1 Pro, criterion, layout only):
  - "super deep" (300 nodes, depth 100): Taffy 366.89 µs; Yoga had not finished after more than 9 minutes.
  - 100k-node "deep, auto size": Taffy 164.24 ms vs Yoga 2.35 s.
  - Yoga wins narrowly on 100k "deep, random size": 63.6 ms vs 70.0 ms.
  - These figures were published by Taffy itself. — [Taffy README](https://github.com/DioxusLabs/taffy)

**Parley and Masonry/Xilem [Parley: shipped crate with adopters; Masonry/Xilem: prototype/alpha apps]**
- Parley merged CoreText font enumeration, CSS `text-indent` and full AccessKit text properties in Q1 2026. Adopters: Bevy (switched to Parley/Fontique), the Gosub browser engine, Takumi, and CuTTY (an Alacritty port). — [Linebender 2026 Q1](https://linebender.org/blog/tmil-25/)
- Masonry moved off hardcoded Vello onto an `imaging` rendering abstraction, which lets Vello CPU render without a GPU. A new layout system landed (xilem#1560). — [Linebender 2026 Q1](https://linebender.org/blog/tmil-25/)

**Bevy UI (ECS) [shipped]**
- UI nodes are ECS entities. A `Node` component holds layout inputs set by the user, and a `ComputedNode` component holds the computed results. A `UiSurface` resource maps entities to Taffy node IDs. Source is secondary (DeepWiki); verify against the bevy_ui source. — [DeepWiki: Bevy window/UI system](https://deepwiki.com/bevyengine/bevy/6.1-window-system)

### Inferences
- In Bevy, splitting layout input and output into separate components (`Node` / `ComputedNode`) and keeping a side-table mapping entities to layout-tree IDs (`UiSurface`) is the ECS version of a common pattern: identity lives in one place (entity / DOM node), and each subsystem keeps its own dense storage keyed by that ID.
- None of these stacks has to support JS-visible node identity, ~500 properties or arbitrary mutation from script. Their performance wins come largely from a smaller scope: few primitives, fixed layout modes, no cascade.

### Gaps
- Flutter's render pipeline (RenderObject tree, layer tree, Impeller), Slint, egui (immediate mode) and Makepad were not researched in this session. No sourced facts are given for them.
- I found no independent benchmarks for GPUI, and no current numbers for Vello vs Skia/Graphite.

## Q2. Blitz (Dioxus Labs): architecture, status, compatibility, reuse

### Takeaway
Blitz is a modular HTML/CSS engine with no JavaScript. It reuses Stylo, Taffy, Parley, html5ever and Vello (via AnyRender). Its DOM is an index-based node tree, and Stylo's per-element data is stored inside each node. Status is pre-alpha/beta (0.3.0-alpha.4, May 2026), and I found no WPT pass rates.

### Cited Findings
- Crates [prototype/beta]:
  - `blitz-dom`: style resolution, layout and events.
  - `blitz-html`: html5ever/xml5ever parsing.
  - `blitz-net`.
  - `blitz-paint`: DOM to AnyRender commands.
  - `blitz-shell`: winit, AccessKit, muda.
  - `dioxus-native`.
  - Stylo handles CSS parsing and resolution, Taffy box layout, Parley text, and Vello rendering via `blitz-renderer-vello`.
  - — [Blitz README](https://cdn.jsdelivr.net/gh/dioxuslabs/blitz@main/README.md)
- Status and scope:
  - The README says it is in "beta": "usable for making apps if you are an early adopter".
  - It renders "many popular no-JS websites (Wikipedia, (Old) Reddit, etc)".
  - It has no WebRTC, WebSockets or localStorage, and does not aim for full browser coverage.
  - The README does not mention a JS engine.
  - — [Blitz README](https://cdn.jsdelivr.net/gh/dioxuslabs/blitz@main/README.md)
- Sources disagree on maturity: the crate page calls it "pre-alpha"; latest blitz-html is 0.3.0-alpha.4 (2026-05-17). — [lib.rs blitz-html](https://lib.rs/crates/blitz-html)
- DOM data model:
  - `BaseDocument` holds `nodes: Box<NodeTree>`, a Stylo `Stylist`, a Stylo `SnapshotMap` (used for restyle invalidation), and dirty sets keyed by `NodeId`.
  - Each `Node` stores `id: NodeId`, `parent: Option<NodeId>` and `children: ThinVec<NodeId>`.
  - It also keeps separate `layout_children` / `paint_children` lists (paint children sorted by z-index), plus Stylo `ElementData` and a Taffy `Cache` / `Layout` inline.
  - — [blitz-dom document.rs](https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/src/document.rs); [blitz-dom node.rs](https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/src/node/node.rs)
- Floats are an opt-in Cargo feature (`taffy/float_layout`, `stylo_taffy/floats`). `stylo_taffy` is the bridge that converts Stylo computed values into Taffy styles. — [blitz-dom Cargo.toml](https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/Cargo.toml)
- Unverified (single source): a beri.net article reports that the Dioxus team joined Cognition (announced 10 September) and that Cloudflare's "Kitesurf" agent browser is built on Blitz. — [beri.net](https://www.beri.net/article/cognition-dioxus-acquihire-blitz-taffy-crate-owners-kitesurf-dependency)

### Inferences
- Blitz is an array-of-structs arena: nodes are fat structs stored contiguously and addressed by integer IDs. It is not struct-of-arrays. Style still goes through Stylo's Arc-shared structs. It shows that Stylo can run against a non-Gecko, non-Servo index-based DOM, which supports the idea that style, layout and text are separable components.
- Because Blitz has no JS, it does not show how index-based nodes would interact with GC'd wrapper objects.

### Gaps
- No WPT or CSS test-suite pass rates were found. The README points to a status page that I did not fetch.
- No memory or performance measurements of Blitz against browsers were found.

## Q3. Coherent Labs Hummingbird/Gameface: data-oriented HTML renderer for games

### Takeaway
Gameface/Hummingbird is a commercial engine written from scratch (not based on WebKit/Chromium/Gecko) that applies data-oriented design. To do so it cuts the web platform down sharply: layout is Yoga flexbox only, and the only display values are `flex` and `none`. The best-known speedup figure, 6.12x for animation ticks over Chromium 66, is a vendor engineer's conference claim.

### Cited Findings
- Cohtml was built "from the ground up with games and real-time applications in mind" and is not based on WebKit/Chromium/Gecko/IE. Cohtml handles layout and logic, then sends commands to the Renoir renderer. The SDK never spawns threads on its own. — [Gameface docs overview](https://docs.coherent-labs.com/cpp-gameface/what_is_gfp/overview/)
- Vendor claim: the showcase UI runs in under 1 ms on a standard PS4. No method is given. — [Coherent Gameface features](https://coherent-labs.com/all-features-gameface)
- Supported subset:
  - "Gameface uses the Yoga layout engine … as its sole layout algorithm."
  - "The only valid display values are `flex` and `none`."
  - "Even elements that default to block … are internally treated as flex containers."
  - `display: grid` is absent and `grid-*` properties are silently ignored.
  - — [Coherent guide: Laying out the screen](https://guide.coherent-labs.com/layout-assets-and-styling/laying-out-the-screen)
- The Gameface linter also flags:
  - unsupported `clamp()`/`min()`/`max()`
  - `calc()` that mixes units (e.g. `calc(50% - 20px)`)
  - `var()` inside `@keyframes`
  - partially supported pseudo-selectors
  - table elements, `<select>` and `<iframe>`
  - — [eslint-plugin-gameface guide](https://frontend-tools.coherent-labs.com/eslint-plugin-gameface/guides/styles-and-markup/)
- Per-property support (including partial values for align-*) is in the CSS properties table. — [Gameface CSS properties](https://coherent-labs.com/Documentation/cpp-gameface/df/d2c/css_props.html)
- Stoyan Nikolov (Coherent Labs, formerly on Chromium/WebKit) compared Chromium 66's OOP animation system with Hummingbird's DoD rewrite. His Meeting C++ 2018 slides report:
  - average animation tick time 6.833 ms (OOP/Chromium) vs 1.116 ms (DoD): "DoD Animations are 6.12x faster".
  - The data was gathered on an Intel i7 PC.
  - The slides attribute the difference to cache misses from dynamic type erasure and to coupling between animations and style solving.
  - The data-oriented version is easier to multithread: each job keeps a private table of modified nodes.
  - — [Nikolov slides, Meeting C++ 2018 (PDF)](https://meetingcpp.com/mcpp/slides/2018/Data-oriented%20design%20in%20practice_Nikolov_MeetingCpp18.pdf); [CppCon 2018 talk listing](https://isocpp.org/blog/2019/08/cppcon-2018-oop-is-dead-long-live-data-oriented-design-stoyan-nikolov)
- The CppCon abstract claims Hummingbird is "several times faster" than Chrome, attributing this to dropping OOP in performance-critical areas. — [isocpp.org](https://isocpp.org/blog/2019/08/cppcon-2018-oop-is-dead-long-live-data-oriented-design-stoyan-nikolov)
- The only independent remark I found is anecdotal, from a forum user about the older Coherent UI plugin: "performance of the system is sub-par". — [Unigine forum](https://developer.unigine.com/forum/topic/2891-coherent-ui-and-unigine)

### Inferences
- The 6.12x result is for one subsystem (CSS animations), compares against Chromium from 2018, and comes from the vendor. It shows that data-oriented layouts help in subsystems with homogeneous work (animation ticks), not that a whole engine can be built this way. The large reduction in layout modes is a big part of why Gameface can be data-oriented.

### Gaps
- No independent benchmark of Gameface against Chromium or Gecko was found. I found no public description of Gameface's internal DOM/style storage beyond the animation talk.

## Q4. Index/arena/SoA trees for DOM-like data; JS identity and GC interaction

### Takeaway
Shipping Rust UI and layout stacks use generational slotmaps (Taffy) or ID-indexed node arenas (Blitz, Bevy entities). I found no measured cache or memory study of these against pointer-based DOMs, and no system that combines index-based DOM storage with JS-visible identity in production.

### Cited Findings
- Taffy uses `SlotMap` with `SecondaryMap`/`SparseSecondaryMap` side tables keyed by `NodeId`. — [taffy_tree.rs](https://github.com/DioxusLabs/taffy/blob/main/src/tree/taffy_tree.rs)
- Blitz uses a `NodeTree` of `Node` structs addressed by `NodeId`, with child lists stored as `ThinVec<NodeId>`. — [blitz-dom node.rs](https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/src/node/node.rs)
- Bevy maps ECS entities to Taffy node IDs through `UiSurface`. — [DeepWiki Bevy](https://deepwiki.com/bevyengine/bevy/6.1-window-system)
- Chromium's LayoutNG inline formatting context is stored as a flat, depth-first list where each item records its descendant count. Chromium credits this with fast iteration, memory efficiency and cursor-based access. This is a data-oriented layout already shipping in a browser. — [Chrome: Key data structures in RenderingNG (2021-08-24)](https://developer.chrome.com/docs/chromium/renderingng-data-structures)

### Inferences
- A practical hybrid: keep a stable node handle (index plus generation) as the identity, and let the JS wrapper or GC object hold that handle. Hot per-phase data (layout results, paint chunks) then goes in dense side tables. Generational indices prevent stale handles from aliasing reused slots. This is a design inference; I found no source that measures it.

### Gaps
- I found no sources measuring cache misses or memory for arena-based vs pointer-based DOMs.
- I found no sources on how Servo's or Ladybird's JS reflectors and GC would work with arena-indexed nodes.
- indextree and generational-arena were not examined.

## Q5. Stylo's data model and why per-property SoA would lose sharing

### Takeaway
Stylo saves memory and work through sharing at three levels: the style sharing cache (identical computed style objects across siblings and cousins), the rule tree (shared matched-rule paths), and style structs (groups of properties shared by pointer). Storing each property in its own column (SoA) would replace pointer sharing with per-element copies.

### Cited Findings
- Style sharing cache:
  - Before computing a node's style, Stylo checks for a reusable cached style.
  - Requirements: same IDs and classes, same inline styles, and parents pointing to the same computed style.
  - For structural pseudo-classes such as `:first-child`, Stylo records a bit vector of match results, and only nodes with identical bit vectors share.
  - Blink had removed its equivalent because modern selectors made it less effective.
  - — [Mozilla Hacks, "Inside a super fast CSS engine: Quantum CSS" (Aug 2017)](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)
- Rule tree: matched rules are stored as specificity-ordered paths in a shared tree. On restyle, if a change cannot affect the descendants' matched rules, they keep their existing rule-tree pointers and skip matching. The design was inherited from Gecko's earlier style system. — [Mozilla Hacks 2017](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)
- Style structs: properties that usually change together (e.g. font) are grouped into structs. A computed style holds pointers to these structs, so siblings and descendants that don't override a struct share it. — [Mozilla Hacks 2017](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)
- Parallelism: Stylo uses work-stealing parallel traversal. The article calls it "embarrassingly parallel" with near-linear speedup in theory, and claims "up to 2 or 4 or even 18 times faster" without stating conditions. — [Mozilla Hacks 2017](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)
- Blitz runs Stylo unmodified-in-spirit over its own arena DOM: Stylo `ElementData` is stored per node, and `SnapshotMap` and `Stylist` are kept per document. — [blitz-dom document.rs](https://github.com/DioxusLabs/blitz/blob/main/packages/blitz-dom/src/document.rs)

### Inferences
- Stylo's model is already partly data-oriented, but it is shaped around sharing: a few pointers per element into deduplicated structs. A flat per-property SoA layout (~500 columns × N elements) would cost far more memory unless it reintroduced deduplication, for example by having columns of indices into interned struct tables. That brings back the struct model, plus an extra level of indirection.
- SoA works better for phase outputs (layout boxes, paint chunks, animation state) than for computed style.

### Gaps
- The article does not say Arc explicitly, and I found no sourced measurements of Stylo style memory (e.g. bytes per element, or sharing-cache hit rates). The Stylo source and Firefox about:memory reports would be the places to look.

## Q6. Data-oriented ideas already inside shipping engines (LayoutNG, RenderingNG property trees, Graphite, WebRender)

### Takeaway
Chromium's RenderingNG already uses several data-oriented patterns: immutable layout output, flat lists, centralized property trees, and cached display lists. These patterns are layered on top of a pointer-based DOM rather than replacing it. **[shipped]**

### Cited Findings
- LayoutNG fragment tree:
  - Fragments are immutable after layout, with no "up" references and no data bubbling down.
  - This allows reuse in later layouts. Incremental updates usually rebuild only the spine of the tree.
  - Possible future uses listed: cross-thread passing, speculative layout and multithreaded layout.
  - — [Chrome RenderingNG data structures](https://developer.chrome.com/docs/chromium/renderingng-data-structures)
- Property trees:
  - There are four per document: transform, clip, effect and scroll.
  - Each element has a 4-tuple "property tree state".
  - They centralize geometry, are cheaper to send across threads and processes than DOM state, and back compositing, IntersectionObserver, culling, invalidation and layout-shift/LCP measurement.
  - — [same](https://developer.chrome.com/docs/chromium/renderingng-data-structures)
- Display lists and paint chunks: display items are stored and reused, unchanged objects copy their items from the previous list, and unchanged stacking contexts are skipped. Paint chunks group items that share a property tree state and feed layerization. — [same](https://developer.chrome.com/docs/chromium/renderingng-data-structures)
- The page gives no quantified performance numbers and does not use the term "data-oriented design". — [same](https://developer.chrome.com/docs/chromium/renderingng-data-structures)

### Inferences
- The patterns that work at web scale are immutability and flat lists at phase boundaries. Identity-heavy mutable state stays in the DOM, and compact derived structures are produced per phase.

### Gaps
- Skia Graphite and WebRender display lists were not verified in this session; no sourced facts are given.

## Q7. GPU-side layout or style computation

### Takeaway
I found no product that runs CSS selector matching, cascade or general web layout on the GPU. Shipping systems use the GPU for rasterization and compositing. Published parallel layout research targets multicore CPUs. **[research only]**

### Cited Findings
- Meyerovich and Bodík, "Fast and Parallel Webpage Layout", presents parallel algorithms for selector matching, layout solving and font rendering. It reports speedups of up to 80x on popular sites and frames layout as an attribute grammar. The target was multicore CPUs, not GPUs. — [paper PDF](https://lmeyerov.github.io/projects/pbrowser/pubfiles/paper.pdf)
- GPU work in this space is on the rasterization side:
  - GPUI draws primitives on the GPU. — [zed.dev/blog/videogame](https://zed.dev/blog/videogame)
  - Vello Hybrid moves fine rasterization to the GPU but keeps path processing on the CPU. — [vello_hybrid](https://skia.googlesource.com/external/github.com/linebender/vello/+/bf5b0f968356bec23fefe76e715e2f91502893eb/sparse_strips/vello_hybrid)
- The Vello project's shift from all-compute-shader rendering toward hybrid CPU/GPU and pure-CPU sparse strips was motivated by device coverage (GPUs without compute shaders), performance cliffs and memory pressure. — [sparse_strips README](https://skia.googlesource.com/external/github.com/linebender/vello/+show/de2b7514d80964598a2ebc99ff209b1fbb291d92/sparse_strips/README.md)

### Inferences
- Selector matching is branchy and depends on the tree, and text shaping depends on sequences and fonts. Both are a poor fit for SIMT GPUs. Even 2D path rendering has moved partly back to the CPU in Vello. A realistic split is CPU (multicore, Stylo-style) style and layout, with GPU rasterization and compositing.

### Gaps
- I found no credible recent (2024–2026) papers or products on GPU CSS cascade or layout. A deeper academic search might find niche work.
