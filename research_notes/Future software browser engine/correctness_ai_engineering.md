# Correctness-at-Scale and AI-Assisted Engineering for a New Browser Engine (state as of Oct 2026)

Method note: the researcher read sources live on 2026-10-14 (web search plus direct fetches of the Ladybird posts, the Ladybird newsletters and the Rust Foundation article). Items marked "(background, not re-fetched)" come from long-standing primary URLs the researcher knows but did not re-open in this session. The report writer should treat those as lower confidence for exact figures.
Status labels: **[SHIPPED]** = in production or a released product; **[PROTOTYPE]** = working system that is not in a shipped product; **[RESEARCH]** = paper or academic artifact.

## Formal verification in production (Verus, Kani, EverParse, Everest/HACL*, F*, SPARK, seL4, browser-specific)

### Takeaway
Formal verification has shipped at scale for small, high-value, well-specified components: parsers of binary formats (EverParse), crypto (HACL*), and microkernels (seL4). It has not shipped for anything as large and loosely specified as a browser engine. The 2024–2026 Rust standard-library effort shows the current pattern at scale: automatic, shallow harnesses cover many functions, and deep contracts cover only a few. For a new browser, the realistic targets are the parsers of untrusted input (image, font and media containers, IPC message formats, URL), crypto, and sandbox/IPC brokers. Layout and the DOM can only be verified at research level today.

### Cited Findings
**Kani / Rust standard library verification (AWS + Rust Foundation)**
- **[SHIPPED in CI, ongoing]** The contest launched in November 2024. About 34,000 functions are in scope across core, alloc and std. More than 450 PRs have come from at least 21 external contributors across four institutions — [Rust Foundation, 1 Sep 2026](https://rustfoundation.org/?p=120341)
- Manual work in the first year produced 725 Kani harnesses (694 backed by formal contracts) and more than 50 VeriFast proofs. Manual contract growth "plateaued around October 2025", which led AWS to build **Autoharness**. Autoharness generates harnesses at the MIR level: 16,748 harnesses (4,645 for unsafe functions, 1,126 for safe abstractions over unsafe code). 11,970 of them were verified against Kani's supported classes of undefined behavior, and 9,600 generic functions were skipped. Only **989 functions** are verified against full formal contracts — [Rust Foundation](https://rustfoundation.org/?p=120341)
- Tools in CI: Kani, ESBMC, VeriFast and Flux. Verus, Creusot, KRust and RAPx are "under review" — [Rust Foundation](https://rustfoundation.org/?p=120341)
- **Outcome:** no previously unknown memory-safety vulnerabilities were found. The effort did surface specification and documentation defects: an incorrect SIMD shift result, missing `unsafe` annotations, wrong SAFETY comments, and a panic-documentation error. All were fixed upstream. A paper was presented at NFM 2026 — [Rust Foundation](https://rustfoundation.org/?p=120341); secondary summary of the "null result" at [youngju.dev, Jul 2026](https://www.youngju.dev/blog/2026-07-16-rust-std-verification-null-result.en)
- An earlier work-in-progress paper (Oct 2025) reported low contract coverage: 361 of 9,078 unsafe functions or safe abstractions in `core` (about 4%), and about 1.4% in `std` — [arXiv 2510.01072](https://arxiv.org/html/2510.01072v2)
- Bounties range from US$5k to US$25k per challenge — [AWS Open Source Blog, Nov 2024](https://aws.amazon.com/blogs/opensource/verify-the-safety-of-the-rust-standard-library/); [devclass](https://devclass.com/2024/11/21/aws-will-pay-devs-to-verify-rust-standard-library-because-of-7500-unsafe-functions-and-enormity-of-task/)

**Verus (SMT-based verified Rust)**
- **[RESEARCH → PROTOTYPE]** Atmosphere, a verified microkernel written in Rust and Verus, appeared at SOSP 2025 ("Atmosphere: Practical Verified Kernels with Rust and Verus"). Amazon gave Anton Burtsev (Utah) a $50K award to develop it — [KVerus arXiv 2605.03822 (cites Atmosphere SOSP'25)](https://arxiv.org/pdf/2605.03822); [Utah CS](https://www.cs.utah.edu/author/corinna-healey/); earlier prototype paper [par.nsf.gov](https://par.nsf.gov/servlets/purl/10549642)
- **[PROTOTYPE]** Asterinas `vostd` is a Verus verification of the Asterinas OSTD framework. It focuses on memory management and synchronization and is labelled "ongoing". The related CortenMM paper won a SOSP 2025 Best Paper award — [github.com/asterinas/vostd](https://github.com/asterinas/vostd)
- **[RESEARCH]** KVerus (arXiv, May 2026) targets LLM-generated Verus proofs at scale. This is evidence that proof writing is now a target for AI automation — [arXiv 2605.03822](https://arxiv.org/pdf/2605.03822)
- The researcher found no source showing Verus-verified code in a shipped Microsoft product as of Oct 2026. The user's "OSDI 2024" attribution for Verus could not be confirmed (background: the main Verus systems paper is generally cited as SOSP 2024, not re-fetched).

**EverParse / Project Everest / HACL* / F\***
- **[SHIPPED]** (background, not re-fetched) EverParse generates C parsers for binary message formats from F*/Low* specifications, with proofs of memory safety, functional correctness and non-malleability. Microsoft reports that it is deployed in the Hyper-V virtual switch, where every network packet entering Azure passes through EverParse-generated code — [MSR blog: EverParse](https://www.microsoft.com/en-us/research/blog/everparse-hardening-critical-attack-surfaces-with-formally-proven-message-parsers/)
- **[SHIPPED]** (background) HACL* verified crypto (F* → C) is in Firefox/NSS (starting with Firefox 57 in 2017), in the Linux kernel (Curve25519), and in CPython's `hashlib` (SHA-2/SHA-3/Blake2 since 3.12). The project page lists the deployments — [hacl-star.github.io](https://hacl-star.github.io/); [Mozilla Security Blog, Sep 2017](https://blog.mozilla.org/security/2017/09/13/verified-cryptography-firefox-57/)

**seL4**
- **[SHIPPED]** (background, dated 2009/2014) seL4 has a functional-correctness proof of about 8,700 lines of C. The proof is on the order of 200k lines of Isabelle and took roughly 20+ person-years for the original effort. This is the canonical proof-to-code ratio of about 20:1+ — [sel4.systems](https://sel4.systems/About/); [Klein et al., TOCS 2014](https://trustworthy.systems/publications/nicta_full_text/7371.pdf)

**SPARK/Ada**
- The researcher found no fresh, primary source this session on the GNAT-LLVM → WebAssembly status. See Gaps.

**Browser-specific verification**
- **[RESEARCH]** Cassius (Panchekha et al.) formalizes a large part of CSS block/inline layout as SMT constraints. **VizAssert** (PLDI 2018) uses Cassius with Z3 to check "Visual Logic" layout properties, such as a button staying visible, across ranges of screen sizes and user settings. It works on whole pages, which limits scale — [MSR: Verifying Web Page Layouts](https://www.microsoft.com/en-us/research/?p=610539); [PLDI 2018 slides](https://homes.cs.washington.edu/~mernst/pubs/verify-layout-pldi2018-slides.pdf); [Panchekha PhD thesis, UW](https://digital.lib.washington.edu/researchworks/handle/1773/44148)
- **[RESEARCH]** Troika (OOPSLA 2019) makes layout verification modular. It splits pages into components because "existing layout verification tools do not scale to large pages" — [OOPSLA 2019 abstract](https://homes.cs.washington.edu/~mernst/pubs/verify-layout-modular-oopsla2019-abstract.html)
- The researcher found no verified HTML5 tokenizer or tree builder in any shipping engine.

### Inferences
- Proof effort is still roughly 5–20x the code size for deep functional correctness. Even AWS's well-funded campaign reached full contracts for only about 3% of in-scope std functions (989 of about 34,000) in about 2 years. A browser engine of several million lines cannot be fully verified. Tiered verification is realistic:
  - **Tier 1, full proofs (EverParse/F*, Verus):** binary parsers (IPC message serialization, image/font/media container headers), crypto, the sandbox broker policy.
  - **Tier 2, bounded model checking (Kani, ESBMC):** every `unsafe` block and FFI boundary.
  - **Tier 3, everything else:** fuzzing and differential testing.
- The std campaign found no memory-safety bugs, but it did find specification and documentation errors. Verification of already-reviewed Rust mostly hardens contracts rather than finding exploitable bugs. Its value is higher at C++/Rust FFI seams.
- Layout verification (Cassius) is a strong oracle-generation idea: a formal CSS semantics could serve as a differential reference for a new layout engine rather than as a proof target.

### Gaps
- No primary 2026 data was found on the SPARK/Ada GNAT-LLVM WebAssembly target. Its maturity is unknown.
- No reliable recent source gives proof-to-code ratios for Verus projects (Atmosphere/vostd). The Verus/Atmosphere papers should be checked directly.
- The researcher did not confirm whether Microsoft ships any Verus-verified component in Windows/Azure as of 2026.
- The researcher found no verified HTML-parsing work (only CSS layout).

## Deterministic simulation testing (DST) and applicability to a multi-process browser

### Takeaway
DST is proven in distributed databases: FoundationDB, TigerBeetle, and Antithesis customers. It has strong commercial momentum (a $105M Series A in Dec 2025). A multi-process browser is structurally similar to a small distributed system: processes plus IPC plus network plus timers. That makes DST unusually applicable if the engine is designed for it from day one, with all I/O, time, randomness and thread scheduling behind injectable interfaces. Retrofitting it is very hard.

### Cited Findings
- **[SHIPPED]** (background) FoundationDB runs the whole cluster (many simulated processes, network, disks) in a single-threaded deterministic simulator with fault injection. A failing seed replays exactly — [FoundationDB testing docs](https://apple.github.io/foundationdb/testing.html)
- **[SHIPPED]** (background) TigerBeetle's VOPR runs the replicated database in a deterministic simulator with injected network, storage and clock faults, and time acceleration — [TigerBeetle docs](https://docs.tigerbeetle.com/); [github.com/tigerbeetle/tigerbeetle](https://github.com/tigerbeetle/tigerbeetle)
- **[SHIPPED, commercial]** Antithesis (a deterministic hypervisor that runs unmodified software deterministically) raised a $105M Series A led by Jane Street on 3 Dec 2025. Jane Street is also a customer. The company claims customers in finance, AI, blockchain and data infrastructure — [PR Newswire, 3 Dec 2025](https://tools.prnewswire.com/en-us/live/20813/release/20251203EN37581). Earlier rounds: $47M seed (Feb 2024) and $30M (Feb 2025) per [Sacra](https://sacra.com/c/antithesis/) (secondary).
- **[SHIPPED, OSS]** (background) The Rust DST libraries are **madsim**, which replaces tokio/std time, net and rand with deterministic versions and is used by RisingWave, and **turmoil** from the Tokio project, which simulates hosts and network in-process — [madsim](https://github.com/madsim-rs/madsim); [turmoil](https://github.com/tokio-rs/turmoil)
- **[SHIPPED]** (background) Record/replay for browsers already exists: Mozilla's `rr` debugger (Linux) is used heavily for Firefox debugging — [rr-project.org](https://rr-project.org/)

### Inferences
- **Browser mapping:**
  - **IPC between processes (browser/renderer/GPU/network):** route through one message-transport abstraction that can run in a single-process simulated mode with a seeded scheduler. This is the turmoil model, with renderer processes as simulated hosts.
  - **Network:** replay HTTP archives via a fake network stack with fault injection (resets, slow bytes, reordering of resource completion). This is the likely source of many load-order races.
  - **Event loop / tasks / timers:** HTML's event loop is already specified as task queues. A deterministic scheduler can explore task-source interleavings that the spec allows, which surfaces races between parser, script, fetch and rendering steps.
  - **GPU/compositor threads, audio and media:** harder. Use Antithesis-style whole-VM determinism or record/replay (`rr`).
- The precondition is architectural: no direct `std::time`, threads, `rand`, or sockets in engine code. The Rust ecosystem makes this enforceable through crate features, as madsim does.

### Gaps
- The researcher found no published case of DST applied to a full browser engine, Servo, Ladybird, Chromium or Gecko.
- No measured bug-yield numbers were found for Antithesis or TigerBeetle in this session.

## Fuzzing and differential testing for browsers; WPT as an executable spec

### Takeaway
Fuzzing at Google scale and the shared WPT/test262 suites are the strongest correctness assets a new engine inherits for free. WPT now gives about 2.08M subtests as an oracle (Ladybird's own count). Interop 2026 continues the yearly cross-engine targets. Differential execution against existing engines, or against an engine's own previous implementation, is the cheapest high-precision oracle. Ladybird used exactly this to gate its AI-assisted Rust ports.

### Cited Findings
- **[SHIPPED]** (background) OSS-Fuzz/ClusterFuzz: the OSS-Fuzz site reports more than 10,000 vulnerabilities and 36,000 bugs fixed across more than 1,000 projects (figure as of Aug 2023) — [OSS-Fuzz](https://google.github.io/oss-fuzz/)
- **[SHIPPED]** (background, Nov 2024) Google's LLM-generated fuzz targets added coverage to 272 C/C++ projects and found 26 new vulnerabilities, including OpenSSL CVE-2024-9143 — [Google Security Blog, Nov 2024](https://security.googleblog.com/2024/11/leveling-up-fuzzing-finding-more.html)
- **[SHIPPED]** (background) Fuzzilli is Project Zero's coverage-guided JS engine fuzzer built on a typed IL, used against V8, JSC, SpiderMonkey and others. Domato is the DOM/CSS grammar-based fuzzer — [Fuzzilli](https://github.com/googleprojectzero/fuzzilli); [Domato](https://github.com/googleprojectzero/domato)
- **[SHIPPED]** WPT as an oracle at scale: Ladybird's WPT subtest pass count was 2,067,263 → 2,075,546 in May 2026 and 2,078,912 → 2,079,020 in July 2026 — [Ladybird May 2026](https://buttondown.com/ladybird/archive/this-month-in-ladybird-may-2026/); [Ladybird July 2026](https://ladybird.org/newsletter/2026-07-31/)
- test262 has at least 52,898 tests, the count Ladybird used for its LibJS gate — [ladybird.org, Feb 2026](https://ladybird.org/posts/adopting-rust/)
- **Interop 2026** is the fifth year of the cross-vendor effort. Selected WPT tests run continuously, with pass rates on the Interop 2026 dashboard. Reported areas include:
  - anchor positioning, container style queries, dialogs/popovers (`closedby`, `:open`), `contrast-color()`, custom highlights and WebVTT
  - advanced `attr()`, IndexedDB `getAllRecords()`, WebTransport and Wasm JS Promise Integration (said to make up 20% of the score)

  Sources: [web.dev, Interop 2026](https://web.dev/blog/interop-2026?hl=en); [WebKit](https://webkit.org/?p=17320). One summary claims 20 focus areas (15 new and 5 carryover), but that summary comes from a scraped mirror. Dashboard: [wpt.fyi/interop-2026](https://wpt.fyi/interop-2026).
- Interop 2025 baseline: Firefox went from 46 to 99 over the cycle (per Mozilla, via search summary) — [web.dev Interop 2026 post](https://web.dev/blog/interop-2026?hl=en)
- **Differential corpus testing:** Ladybird checked its new Rust HTML parser for DOM parity with the C++ parser on a corpus of about 7,000 websites — [Ladybird May 2026](https://buttondown.com/ladybird/archive/this-month-in-ladybird-may-2026/)

### Inferences
- A new engine should treat WPT and test262 as the acceptance spec from day one, track pass counts in CI, and add three differential oracles:
  1. **Old vs new implementation:** byte-for-byte output or DOM-tree parity during rewrites.
  2. **Engine vs reference engines:** layout boxes, DOM serialization and JS results compared against Chromium/Gecko/WebKit on generated inputs (Domato/Fuzzilli-style).
  3. **Engine vs a formal model:** ESMeta for JS, Cassius-style for CSS.
- LLM-written fuzz harnesses (Google 2024) are a low-risk, measured win: generated tests can be thrown away, and found bugs are real.

### Gaps
- Live Interop 2026 scores per engine (Oct 2026) were not retrieved.
- The total WPT test and subtest count for the whole suite (as opposed to Ladybird's pass count) was not confirmed.
- No current (2026) OSS-Fuzz totals were retrieved.

## AI/LLM-assisted development in 2026: evidence, defect rates and limits

### Takeaway
The strongest measured evidence for AI-assisted engineering comes from translations gated by a pre-existing oracle. Ladybird's LibJS frontend port (about 25k lines of Rust in about 2 weeks) matched the C++ output byte for byte on 52,898 test262 tests and 12,461 regression tests. Its HTML-parser port was checked for DOM parity on about 7,000 sites. Corporate "X% of code is AI" figures are self-reported and unverifiable. The best controlled study (METR 2025) found experienced developers were 19% *slower*, and its 2026 follow-up is inconclusive. Large-scale AI C-to-Rust translation (DARPA TRACTOR, Microsoft's 2030 aspiration) is still research.

### Cited Findings
**Ladybird (most relevant precedent)**
- **[SHIPPED in Ladybird pre-alpha]** On 23 Feb 2026, Ladybird ported the LibJS lexer, parser, AST and bytecode generator from C++ to Rust using Claude Code and Codex:
  - Size and speed: about 25,000 lines of Rust in about two weeks, versus an estimated "several months" by hand.
  - Process: human-directed through "hundreds of small prompts", with Kling choosing what to port, in what order and in what style. After translation, he ran "multiple adversarial review passes using different models".

  Source: [ladybird.org/posts/adopting-rust](https://ladybird.org/posts/adopting-rust/)
- **Equivalence gating:**
  - Every Rust AST is identical to the C++ one, and the bytecode is identical.
  - test262: 52,898 tests, 0 regressions. Ladybird regression tests: 12,461 tests, 0 regressions. No regressions on the JS benchmarks the project tracks.
  - A "lockstep mode" ran both pipelines during real browsing and compared their output.

  Source: [ladybird.org](https://ladybird.org/posts/adopting-rust/)
- Caveat stated by Kling: the Rust code is deliberately **non-idiomatic** and mirrors the C++ (including register allocation patterns) so the outputs match. Cleanup is deferred until the C++ path is retired — [ladybird.org](https://ladybird.org/posts/adopting-rust/). The Feb 2026 newsletter says the Rust pipeline is on by default, with `LIBJS_CPP=1` as a fallback — [Ladybird Feb 2026](https://ladybird.org/newsletter/2026-02-28/)
- **May 2026:** the HTML tokenizer, tree builder and speculative preload scanner are fully in Rust. The new parser is about 10% faster than the C++ one and passed DOM-parity checks on about 7,000 websites. The URL parser and URLPattern also moved to Rust. This newsletter does not mention AI use — [Ladybird May 2026](https://buttondown.com/ladybird/archive/this-month-in-ladybird-may-2026/)
- **July 2026:** much of LibWeb's style and layout moved to Rust: selector matching, computed style, cascade, animations, `calc()`, layout tree construction and every formatting context through SVG. Parts still call back into C++, and CSS parsing, the DOM and painting were not ported. Style values are now "immutable, shared values that Rust owns" as groundwork for parallel restyle. The newsletter mentions no AI use and no verification method — [Ladybird July 2026](https://ladybird.org/newsletter/2026-07-31/)
- An unverified secondary claim says Rust became mandatory and the `ENABLE_RUST` flag was removed by April 2026 — [byteiota](https://byteiota.com/ladybird-browser-goes-full-rust-ua-spoofing-chrome-2026/). Ladybird's own pages do not confirm it.

**Other large AI-assisted migrations and claims**
- **[RESEARCH]** DARPA TRACTOR aims for highly automated C→Rust translation "with the same quality and style that a skilled Rust developer would employ". MIT Lincoln Lab does test and evaluation — [DARPA TRACTOR](https://www.darpa.mil/program/translating-all-c-to-rust). One performer team (UW–Madison, Berkeley, Edinburgh, UIUC, about $5M) uses "ForCLift", a verified-lifting approach that combines formal methods with LLMs — [UW–Madison CS, Jul 2025](https://www.cs.wisc.edu/2025/07/15/translating-legacy-code-for-a-safer-future/). The PM, Dan Wallach, said LLM output is "not entirely error-free" — [Dark Reading](https://www.darkreading.com/application-security/darpa-aims-to-ditch-c-code-move-to-rust). **No 2026 TRACTOR results were found.**
- **[RESEARCH, not a plan]** In Dec 2025, Galen Hunt (Microsoft) posted a goal of removing C/C++ from Microsoft by 2030 using AI, with "1 engineer, 1 month, 1 million lines of code". He later clarified that "Windows is NOT being rewritten in Rust with AI" and that it is a research project — [The Register, 24 Dec 2025](https://www.theregister.com/2025/12/24/microsoft_rust_codebase_migration/); [TechRadar](https://www.techradar.com/computing/cloud-computing/microsoft-engineer-clarifies-speculation-around-plans-to-eliminate-c-c-languages-by-2030)
- **[SELF-REPORTED]** Google says about 75% of its new code is AI-generated and then approved by engineers (April 2026). The figure was 25% in 2024 and 50% in fall 2025. Google also cited a migration done "six times faster". Semafor notes these figures are "pretty impossible to fact-check" — [Semafor, 24 Apr 2026](https://www.semafor.com/article/04/24/2026/google-ceo-says-75-of-companys-new-code-is-ai-generated)
- **Controlled evidence:** METR's 2025 RCT had 16 experienced OSS developers on their own repositories, mostly using Cursor with Claude 3.5/3.7 Sonnet. They were **19% slower** with AI, yet believed they were about 20% faster — [scienceblog summary](https://scienceblog.com/t-a-randomized-trial-by-metr-found-that-experienced-developers-completed-real-coding-tasks-19-slower-when-allowed-to-use-ai-tools-yet-afterwards-they-estimated-on-average-that-ai-had-made-them-20-fast/); primary at [metr.org](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/). Secondary sources say the 2026 follow-up showed "some evidence of faster work", but selection effects make the size unreliable, and METR now labels the 2025 result historical — [codercops blog](https://blog.codercops.com/blog/ai-coding-productivity-2x-not-10x-metr-2026) (secondary; primary not retrieved).
- **Memory-safe language outcome data (background):** Android memory-safety vulnerabilities fell from 76% (2019) to 24% (2024) of all vulnerabilities as new code shifted to memory-safe languages — [Google Security Blog, Sep 2024](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html)
- **AI as bug finder:** Google's Big Sleep agent found an exploitable SQLite stack buffer underflow before release (Oct/Nov 2024) — [Project Zero](https://googleprojectzero.blogspot.com/2024/10/from-naptime-to-big-sleep.html)

### Inferences
- The pattern that works is **oracle-gated, human-steered, incremental translation**. It needs:
  - a working reference implementation;
  - massive conformance suites (test262/WPT);
  - byte-for-byte or tree-equivalence checks and lockstep execution;
  - small prompts plus cross-model adversarial review.

  Ladybird shows this can move about 25k LOC in weeks with zero measured regressions. The price is non-idiomatic code, which leaves cleanup debt.
- For a *new* engine with no reference implementation, the equivalent oracles are the other engines (differential rendering) plus WPT/test262. Spec-driven AI generation against WPT is plausible, but the researcher found no measured public case of an agent building a conformant browser subsystem from spec alone.
- Self-reported AI code shares (Google's 75%) say nothing about defect rates. The researcher found no public measured defect-rate comparison for AI-written browser code.

### Gaps
- No primary data was found on defect or bug-escape rates of AI-written code in large systems (2026).
- The researcher could not confirm whether Ladybird's May–Aug 2026 ports (HTML parser, style/layout) used AI agents or byte-for-byte gating. The newsletters do not say. The August 2026 newsletter was not read.
- No public agent-builds-engine-from-WPT/test262 result with measured pass rates was found.
- No Microsoft-measured outcomes for AI C→Rust translation were found.

## Executable specifications and spec-to-code

### Takeaway
WHATWG, TC39 and CSSWG specifications are written as step-by-step algorithms, which makes them unusually amenable both to mechanical extraction and to LLM translation. ECMAScript has a mature mechanized-spec toolchain (ESMeta). CSS has research-grade formal semantics (Cassius). HTML/DOM has no equivalent machine-readable semantics.

### Cited Findings
- **[RESEARCH → used upstream]** (background) ESMeta, from KAIST's JISET/JEST line of work, mechanically extracts ECMA-262's algorithm steps into an executable IR. It uses that IR to run test262, generate conformance tests, type-check the spec, and find bugs in both the spec and engines. Spec bugs it found were reported to TC39 — [github.com/es-meta/esmeta](https://github.com/es-meta/esmeta)
- **[RESEARCH]** Cassius gives a formal, SMT-encoded semantics of a large fragment of CSS layout — [MSR](https://www.microsoft.com/en-us/research/?p=610539); [Panchekha thesis](https://digital.lib.washington.edu/researchworks/handle/1773/44148)
- Spec-adherent coding style: Ladybird's codebase mirrors spec algorithm steps (cited implicitly by its spec-driven approach). Its AI ports kept structural parity with the existing C++ rather than re-deriving code from the specs — [ladybird.org](https://ladybird.org/posts/adopting-rust/)

### Inferences
- A practical "executable spec" pipeline for a new engine:
  1. Generate code skeletons with step-number comments from WHATWG/TC39 algorithm text.
  2. Run ESMeta as a reference interpreter and differential oracle for the JS semantics.
  3. Use a Cassius-like solver model as a layout oracle on small generated cases.
  4. Use WPT/test262 as acceptance gates.
- LLMs are well matched to step 1 because the specs are already pseudo-code. Correctness must still come from steps 2–4, not from the model.

### Gaps
- No machine-readable formal semantics of the WHATWG HTML parser or DOM was found in this session.
- No measured spec-to-code generation results (LLM or mechanical) for WHATWG specs were found.
