# Browser Engine Architectures, Implementation Languages, and JS Engine Options (state as of Oct 2026)

Research date: 2026-10-13. Older data is labelled with its date. Items marked **[unverified-URL]** are primary sources cited from prior knowledge whose URLs were not fetched in this session; the report writer should treat them as lower-confidence and prefer the fetched sources.

## Q1. Servo: process/threading model, 2025-2026 status, measured parallel layout speedups, origin of "2x-18x"

### Takeaway
Servo is now a Linux Foundation Europe project that ships monthly releases (first GitHub release Oct 2025; `servo` crate 0.1.0 on crates.io with a 6-monthly LTS in April 2026; 0.5 by end of Aug 2026), focused on being an embeddable engine. The only rigorous measured parallel-layout figure remains HiPC 2016 (3.2x and 2.5x on 4 threads on *some* sites, with no gain or a loss on others). The "2x-18x" figure is not a Servo layout measurement. It traces to Mozilla's 2017 Stylo (CSS style) explainer, which said "up to 2 or 4 or even 18 times faster", meaning it scales with core count. It is a rhetorical upper bound, not a benchmark.

### Cited Findings
**Architecture**
- One constellation per Servo instance. It manages the web content processes for all frames in all WebViews, tracks global state including the set of all `EventLoop`s, and may live in a different process from the pipelines, talking to them over IPC. — [Servo book: architecture](https://book.servo.org/design-documentation/architecture.html); [Constellation API docs](https://doc.servo.org/servo_constellation/struct.Constellation.html)
- A "pipeline" is the constellation's view of a window/frame: a script thread plus layout. A content process can run several script threads, each with its own layout. — [Servo book: architecture](https://book.servo.org/design-documentation/architecture.html)
- Multiprocess mode is optional. When it is enabled, each script thread runs in its own web content process. When it is disabled, script threads run inside the embedder process. — [Servo book: architecture](https://book.servo.org/design-documentation/architecture.html)
- The compositor passes display lists to WebRender, which handles rasterization and display for both Servo and Firefox. — [Servo book: architecture](https://book.servo.org/design-documentation/architecture.html)
- The book describes Rust as providing a task-parallel infrastructure plus a type system that enforces memory safety and freedom from data races. — [Servo book: architecture](https://book.servo.org/design-documentation/architecture.html)
- Servo uses SpiderMonkey via the `mozjs` crate (Rust bindings). — [servo/mozjs GitHub](https://github.com/servo/mozjs) **[unverified-URL]**

**2025-2026 status**
- The servo.org footer states that Servo is a project of Linux Foundation Europe. Monthly "This month in Servo" posts appeared throughout 2026 (Jan through Aug), and on 2026-09-15 the project posted "Your Donations at Work: One Year of Sponsored Servo Development". — [servo.org blog](https://servo.org/blog/)
- On 2026-04-13 the project published `servo` 0.1.0 to crates.io as its first crates.io release, "the fifth release since the initial GitHub release in October 2025". The embedding API is documented on docs.rs. An LTS version is offered because monthly releases include breaking changes, so embedders can upgrade on a half-yearly schedule and still receive security updates. servoshell will not be published to crates.io, and the team has not yet defined what 1.0 means. — [Servo is now available on crates.io](https://servo.org/blog/2026/04/13/servo-0.1.0-release/); [Phoronix](https://phoronix.com/news/Servo-Embed-Crates-LTS)
- Servo 0.4.0 shipped on 31 Jul 2026 with 558 changes. — [linux.org.ru](https://www.linux.org.ru/news/internet/18350669) (secondary)
- Servo 0.5 shipped on 31 Aug 2026 with better embedding docs (positioned as an alternative to CEF), up to 10x faster text rendering, and Linux AArch64 binaries. — [Phoronix](https://phoronix.com/news/Servo-0.5-Released)
- The Feb 2026 monthly post covered "faster layout" and embedding API improvements. Other 2026 posts covered multiple windows, proxy support, FreeBSD support, a new Android UI, SharedWorker and user scripts. — [servo.org blog](https://servo.org/blog/)
- The Verso browser (built on Servo) is reported archived as of mid-2026. — [youngju.dev](https://www.youngju.dev/split/2026-07-17-servo-embedding-crates-io-verso-archived) (secondary, low confidence)

**Measured parallel speedups**
- HiPC 2016 (Zambre et al., UC Irvine): Servo layout reached 3.2x and 2.5x with 4 threads on some websites. Parallelization helped pinterest.com but not google.com, and "in many cases, the overhead of creating, deleting, and coordinating parallel work outweighs any of its benefits." On a quad-core Ivy Bridge (i7-3615QM) laptop across 535 pages, a predictive model that picks the thread count improved performance by "up to 94.52%" and energy by up to 46.32%. These are maximums. — [HiPC 2016 paper PDF](https://hpcforge.eng.uci.edu/publication/hipc16-browser/hipc16-browser.pdf); [arXiv 2002.03850](https://arxiv.org/abs/2002.03850v1)
- Servo has since made parallel table layout the default, spreading row and column layout across all cores with Rayon. No benchmark figure was published. — [Phoronix](https://phoronix.com/news/Servo-Multi-Core-HTML-Tables)
- Source of "2x-18x": Mozilla Hacks (Lin Clark, Aug 2017), about Stylo: it parallelizes across all cores, "which means it can run up to 2 or 4 or even 18 times faster". — [Mozilla Hacks 2017](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)

### Inferences
- The "2x-18x" phrase describes CSS *style* (Stylo) scaling with core count (2, 4 or 18 cores), not measured *layout* speedups. An HGBA-2.0 spec should not cite it as evidence for parallel layout. The defensible layout figure is "2.5-3.2x on 4 threads on some pages, often none on simple pages (HiPC 2016)".
- Servo's design reuses SpiderMonkey via mozjs, so it is not a pure-Rust stack: the JS engine is C++.

### Gaps
- No 2025-2026 published benchmark of Servo's current (Layout 2020-derived) parallel layout or table layout speedups was found.
- I did not confirm whether multiprocess mode or sandboxing is on by default in Servo 0.5, because the crates.io and blog posts don't mention it.

## Q2. Stylo (Quantum CSS) and WebRender in Firefox

### Takeaway
Stylo, Servo's parallel Rust style system, has shipped in Firefox since 57 (Nov 2017). Mozilla's public numbers are marketing-grade: "up to 2/4/18x" with core count, and a 30% page-load speedup on Amazon's homepage. No controlled per-thread scaling study was found.

### Cited Findings
- Stylo uses Rayon work-stealing to parallelize style across all cores. Mozilla made two earlier attempts to parallelize style in C++, and both failed. It also includes existing single-thread style optimizations from other browsers. — [Mozilla Hacks 2017](https://hacks.mozilla.org/2017/08/inside-a-super-fast-css-engine-quantum-css-aka-stylo/)
- Gecko uses Servo's Rust parallel style system from Firefox 57 onward. — [Mozilla wiki diff](https://m.wiki.mozilla.org/Special:MobileDiff/1191550)
- The quoted claim of a "30% page load speedup for Amazon's homepage" bundles parallelism with other changes. — [mjtsai.com roundup, 2017-11-17](https://mjtsai.com/blog/2017/11/17/firefox-quantum)
- WebRender handles rasterization and display for both Servo and Firefox. — [Servo book](https://book.servo.org/design-documentation/architecture.html)

### Inferences
- Stylo is the strongest production evidence that parallel style in Rust is shippable at scale, but the speedup data is old (2017) and vendor-sourced.

### Gaps
- No independent measured Stylo thread-scaling data was found. Firefox's current WebRender rollout status and its 2019-2021 enablement dates were not re-verified this session.

## Q3. Ladybird: Rust adoption Feb-Oct 2026, Swift abandoned, alpha timeline

### Takeaway
Ladybird changed course sharply in 2026. In February it replaced the LibJS frontend with ~25k lines of Rust that produce byte-for-byte identical bytecode, translated with AI agents. By July-August 2026 much of LibWeb's style and layout, CSS parsing and painting had also moved to Rust. Swift was dropped. The first alpha is still planned for 2026.

### Cited Findings
- 23 Feb 2026 (Andreas Kling): LibJS's lexer, parser, AST and bytecode generator were ported to about 25,000 lines of Rust in about two weeks using Claude Code and Codex. Kling calls it "human-directed, not autonomous code generation" with "hundreds of small prompts" and adversarial multi-model review. ASTs and bytecode are identical to the C++ pipeline's output. test262 ran 52,898 tests and Ladybird's own regression suite 12,461, both with 0 regressions, and there were no performance regressions on tracked benchmarks. Lockstep browsing gave identical output. — [Ladybird: Adopting Rust](https://ladybird.org/posts/adopting-rust/)
- Why Swift was dropped: its C++ interop "never quite got there", and platform support outside Apple's ecosystem was limited. Rust had been rejected in 2024 as "not great at C++ style OOP", but the team now judged it the pragmatic choice. — [Ladybird: Adopting Rust](https://ladybird.org/posts/adopting-rust/)
- The Rust pipeline is on by default on all platforms, and `LIBJS_CPP=1` reverts to C++. The Swift code was removed. — [This Month in Ladybird, Feb 2026](https://ladybird.org/newsletter/2026-02-28/)
- Feb policy: porting is "a sidetrack that runs for a long time", C++ development continues, and contributors must coordinate before porting. — [Ladybird: Adopting Rust](https://ladybird.org/posts/adopting-rust/)
- July 2026: "much of LibWeb's style and layout pipeline" moved to Rust, with the explicit goals of safety and parallelism. That covers selector matching, computed style, cascade, animations, `calc()`, layout tree construction, and every formatting context from block through SVG. Style is stored as immutable shared values owned by Rust. CSS parsing, the DOM and painting had not moved yet. The JS interpreter inner loop was rewritten in "flap", an in-house typed language targeting x86-64 and AArch64. There is no JIT. — [This Month in Ladybird, Jul 2026](https://ladybird.org/newsletter/2026-07-31/)
- August 2026: CSS parsing moved to Rust using A/B "zero mismatches" gating before the C++ code was deleted. Computed style is fully Rust-owned, and painting (display list recording and hit testing) was ported. JPEG XL decoding moved to jxl-rs. Session history and navigation moved into the browser process as a site-isolation prerequisite. A sandboxed WasmCompiler service was added, and page processes lost fork permission. Cage-based pointer limits were added for JS values ("doesn't fully cage the JS heap"). WPT reached 2,088,677 subtests. On Linux, Speedometer 2 went from ~47 to ~64 and Speedometer 3 from ~2.5 to ~3.9. — [This Month in Ladybird, Aug 2026](https://ladybird.org/newsletter/2026-08-31)
- Alpha: "We've said the first alpha starts in 2026, and that's still the plan." The remaining work is mostly non-engine: crash reporting, signed builds, auto-update and first-run. — [This Month in Ladybird, Aug 2026](https://ladybird.org/newsletter/2026-08-31)

### Inferences
- Ladybird is now a hybrid C++/Rust engine whose style, layout and paint core is moving to Rust. That makes it a second data point, alongside Servo, for a Rust-first engine core. The equivalence-testing method (byte-identical output, A/B zero-mismatch gating) is a reusable migration pattern for HGBA-2.0.

### Gaps
- No September 2026 newsletter was found (possibly not yet published or indexed). The alpha date is still unannounced.

## Q4. Chromium: process model, RenderingNG, Rust adoption, memory-safety stats, MiraclePtr

### Takeaway
Chromium isolates sites into locked processes on desktop (full site-per-process). Android uses only partial isolation and requires about 2 GB of RAM. Around 70% of Chromium's high and critical security bugs are memory-safety bugs (912 bugs since 2015). Rust is entering through leaf libraries: fonts (Fontations/Skrifa, default for web fonts since Chrome 133) and PNG (RustyPng, still behind a feature flag in the sources found).

### Cited Findings
- Desktop uses full site-per-process. On Android, site-per-process is only a manual flag, and the default is partial isolation where only some sites get dedicated locked processes. About 2 GB of device RAM is needed for any level of site isolation on Android. — [Chromium docs: Process Model and Site Isolation](https://chromium.googlesource.com/chromium/src/+/main/docs/process_model_and_site_isolation.md)
- Android memory thresholds come from the field-trial params `strict_site_isolation_threshold_mb` and `partial_site_isolation_threshold_mb`, compared against physical RAM. — [components/site_isolation/features.cc](https://chromium.googlesource.com/chromium/src/+/cff8ad5/components/site_isolation/features.cc)
- "Around 70% of our high severity security bugs are memory unsafety problems", based on 912 high or critical bugs affecting Stable since 2015. The same page notes the Rule of 2 (process separation for untrustworthy data) sometimes blocks features. — [Chromium Memory Safety](https://www.chromium.org/Home/chromium-security/memory-safety/)
- Fontations/Skrifa (Rust) replaces the FreeType parts Skia uses. Chrome 128 (Aug 2024) used it for less common formats, and Chrome 133 (Feb 2025) for all web fonts on Linux, Android and ChromeOS. OS fonts are planned next. — [Chrome blog: Memory safety for web fonts, 2025-03-19](https://developer.chrome.com/blog/memory-safety-fonts)
- The Rust PNG decoder (SkPngRustCodec on the `png` crate) is gated by `enable_rust_png` and the `RustyPng` feature. Its default state was not confirmed. — [Skia rust_png README](https://skia.googlesource.com/skia/+/cbca9656f1d05cd058354666151a255bcc25113a/experimental/rust_png/README.md); [rusty_png_feature.h](https://chromium.googlesource.com/chromium/src/+/HEAD/skia/rusty_png_feature.h)
- RenderingNG pipeline stages: animate, style, layout, pre-paint, scroll, paint, commit, layerize, raster, activate, aggregate (Viz), draw. — [Chrome: RenderingNG architecture](https://developer.chrome.com/docs/chromium/renderingng-architecture) **[unverified-URL]**
- MiraclePtr (BackupRefPtr) protects raw pointers against UAF in the browser process. Google (Sept 2022) estimated it would make 57% of UAF bugs in privileged processes non-exploitable. — [Google Security Blog, "Use-after-freedom: MiraclePtr"](https://security.googleblog.com/2022/09/use-after-freedom-miracleptr.html) **[unverified-URL]**
- Chromium policy (Jan 2023) supports third-party Rust libraries called from C++. — [Google Security Blog, "Supporting the use of Rust in the Chromium project"](https://security.googleblog.com/2023/01/supporting-use-of-rust-in-chromium.html) **[unverified-URL]**

### Inferences
- Chromium's Rust strategy is "replace parsers and decoders of untrusted input" rather than rewriting the core engine. That contrasts with Ladybird's 2026 approach of porting the core.

### Gaps
- QR-code generator and other Rust components were not re-verified. The current RustyPng default, current Android threshold values and process-count limits were not found.

## Q5. Gecko: Fission, Rust share, Oxidation

### Takeaway
Rust has been in Firefox since version 48 (Oxidation). Community trackers put it at about 11.7% of all Firefox code, or about 4.7M lines, in early 2025. This is not an official Mozilla figure, and it includes vendored crates.

### Cited Findings
- firefox-lang-stats (early 2025) counted Rust at 11.7%, C at 13.8% and C++ at 26.7% of the codebase. A later thread cited about 4.7M lines of Rust. Much of the total is HTML and JS, and a substantial part of the Rust is vendored third_party code. — [LWN comment thread](https://lwn.net/Articles/1007781); [LWN](https://lwn.net/Articles/993608)
- Rust first shipped in Firefox 48 (media stack) under Project Oxidation. — [Neowin](https://www.neowin.net/news/mozilla-firefox-will-ship-with-its-first-pieces-of-rust-code-in-upcoming-release)
- Fission (site isolation) was enabled for all desktop users in Firefox 95 (Dec 2021). — [Mozilla Hacks: Introducing Firefox's new Site Isolation Security Architecture (2021)](https://hacks.mozilla.org/2021/05/introducing-firefox-new-site-isolation-security-architecture/) **[unverified-URL]**

### Gaps
- No official 2026 Mozilla Rust-share figure was found, and Fission's Android status was not checked.

## Q6. JavaScript engine options for a Rust-first engine

### Takeaway
The pure-Rust engines are spec-conformant interpreters with no production JIT. Boa reached ~94-95.6% test262 in 2025. Nova (data-oriented, NLnet-funded) claims ~80% and is explicitly non-compliant in places. Production-grade performance still means embedding C++ engines: SpiderMonkey via mozjs (as Servo does) or V8 via rusty_v8 (as Deno does). JIT is the main memory-safety liability. Microsoft's 2021 data put ~45% of V8 CVEs in the JIT, and the V8 sandbox (about 1% overhead, on by default on 64-bit) is V8's mitigation.

### Cited Findings
- Boa v0.21 (Oct 2025): test262 went from 89.92% to 94.12%, mostly from Temporal. — [Boa v0.21 release](https://opencollective.com/boa/updates/boa-release-v0-21)
- Boa v0.22: 95.60% with Temporal on by default (third-party mirror; date not confirmed). — [newreleases.io Boa v0.22](https://newreleases.io/project/github/boa-dev/boa/release/v0.22)
- Nova: Rust engine with data-oriented design. Values are Rust enums holding on-stack data or 32-bit heap handles (not NaN-boxed). It claims ~80% test262 (undated) and is "not compliant" for some RegExp behaviour. Interpreter performance is "not yet a high priority". NLnet NGI Zero Core funding started Dec 2024. — [docs.rs nova_vm 1.0.0](https://docs.rs/crate/nova_vm/latest); [NLnet Nova](https://nlnet.nl/project/Nova/); [trynova.dev mirror](https://pycoder42--f92f31dc3d1611f189e242b51c65c3df.web.val.run/https://trynova.dev/)
- Brimstone (Rust) is reported at 97% ECMAScript support (aggregator, unverified). — [feedbagel](https://feedbagel.com/post/brimstone-a-new-javascript-engine-written-in-rust-with-97-ecmascript-support)
- V8 sandbox: 64-bit only, enabled by default on supported configurations, overhead about 1% or less on Speedometer/JetStream. Any corruption outside the sandbox address space counts as a sandbox violation. It was designed against heap corruption, e.g. corrupted ArrayBuffer pointer or length fields. — [V8 blog: The V8 Sandbox](https://v8.dev/blog/sandbox); [V8 src/sandbox README](https://chromium.googlesource.com/v8/v8/%2B/refs/heads/12.2.206/src/sandbox/)
- Microsoft (2021): ~45% of V8 CVEs since 2019 were JIT-related, and more than half of in-the-wild Chrome exploits used JIT bugs. "Super Duper Secure Mode", renamed Enhanced Security Mode, disables the JIT (TurboFan/Sparkplug) and enables CET. Microsoft claimed the JIT is no longer crucial to Edge performance. — [BleepingComputer](https://www.bleepingcomputer.com/news/microsoft/microsoft-edge-just-got-a-super-duper-secure-mode-upgrade); [Slashdot/The Record](https://tech.slashdot.org/story/21/08/05/1845257/microsoft-announces-super-duper-secure-mode-for-edge)
- Ladybird's LibJS (Rust frontend) has no JIT, and its interpreter loop is generated by "flap". — [Ladybird Jul 2026](https://ladybird.org/newsletter/2026-07-31/)
- rusty_v8 (Deno's Rust bindings for V8) — [denoland/rusty_v8](https://github.com/denoland/rusty_v8) **[unverified-URL]**; Servo's SpiderMonkey bindings — [servo/mozjs](https://github.com/servo/mozjs) **[unverified-URL]**

### Inferences
- A realistic HGBA-2.0 choice is SpiderMonkey via mozjs or V8 via rusty_v8 for performance, with a JIT-less mode for hardened profiles, while tracking Boa or Nova as a pure-Rust option. Boa's conformance is near browser level, but its performance and Web-API integration (GC/DOM bindings) are unproven at browser scale.

### Gaps
- No 2026 Boa or Nova releases or benchmark comparisons against V8 or SpiderMonkey were found. No current (2024-2026) JIT-bug share figure was found to update Microsoft's 2021 number.

## Q7. RISC-V support

### Takeaway
V8 riscv64 is upstream, near parity with x64 and arm64, and green on V8 buildbots (Dec 2025). SpiderMonkey's riscv64 JIT and Wasm backend landed in 2023 and was enabled by default for Firefox 143.

### Cited Findings
- The V8 RISC-V backend was upstreamed in Feb 2021 (RIOS Lab, Futurewei, PLCT). As of Dec 2025 it is "close to parity" with x86_64 and ARM64 and runs on V8's buildbots, where it is among the greenest ports. — [RISE Project, 2025-12-09](https://riseproject.dev/2025/12/09/a-glimpse-into-v8-development-for-risc-v/); [RISC-V wiki: Language Runtimes](https://riscv.atlassian.net/wiki/spaces/HOME/pages/16154665/Language+Runtimes)
- The SpiderMonkey RISC-V64 JIT/Wasm backend, contributed and maintained by PLCT Lab, landed in the Firefox 110-111 cycle (Feb 2023). — [SpiderMonkey newsletter FF110-111](https://spidermonkey.dev/blog/2023/02/16/newsletter-firefox-110-111.html)
- The riscv64 JIT was enabled by default for Firefox 143 (bug 1982266). — [Bugzilla 1982266](https://bugzilla.mozilla.org/show_bug.cgi?id=1982266)
- Claimed JIT speedups on Kraken are up to 40x (openEuler) and about 49.2x on average on a SiFive Unmatched (ISCAS). These are vendor claims. — [openEuler blog](https://www.openeuler.org/en/blog/20230113-RISC/RISC); [ISCAS](https://iscas.cas.cn/xwdt2016/kyjz2016/202304/t20230403_6725585.html)

### Gaps
- No official Chromium (browser) riscv64 release or bot status was found. Servo or mozjs riscv64 support was not checked.

## Q8. Other memory-safe non-GC languages (Swift 6, Ada/SPARK) vs "Rust is the only production-proven language"

### Takeaway
Swift 6 offers compile-time data-race safety, but it can be bypassed (`@unchecked Sendable`), migration is opt-in per module, and Ladybird abandoned Swift over C++ interop and non-Apple support. No browser engine built in Ada/SPARK was found. The claim is defensible only in a narrow form: "Rust is the only memory-safe, non-GC language with production browser-engine components shipped at scale (Stylo, WebRender, Fontations, Servo)". It should not be framed as "the only production-proven memory-safe language" in general, since Ada/SPARK has long avionics and defense deployments.

### Cited Findings
- Swift 6 data-race safety is migrated module by module and enabled with `-strict-concurrency=complete`. — [swift.org: Enabling Complete Concurrency Checking](https://swift.org/documentation/concurrency/)
- Clean results are not proof of safety because of escape hatches like `@unchecked Sendable`. — [Swift Package Index: Ready for Swift 6](https://swiftpackageindex.com/ready-for-swift-6)
- A pre-Swift-6 update said there were "a number of bugs and holes in the Sendable checking model". — [Swift forums: Progress toward Swift 6 language mode (2023)](https://forums.swift.org/t/progress-toward-the-swift-6-language-mode/68315)
- Ladybird dropped Swift for C++ interop and cross-platform reasons. — [Ladybird: Adopting Rust](https://ladybird.org/posts/adopting-rust/)

### Inferences
- The only production browser-engine evidence for a non-GC memory-safe language is Rust: Firefox's Stylo and WebRender, Chrome's Fontations, Servo, and Ladybird's 2026 ports.

### Gaps
- No sources were gathered on Ada/SPARK browser or rendering-engine use or its deployment record. The search returned nothing for that part, so any Ada/SPARK statement in the spec needs separate sourcing.
