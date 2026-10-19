# Izcal JavaScript engine: fact-check (phase 8)

Checked 19 October 2026 against `reports/Izcal JavaScript engine.md` (draft 0.1) by a checker who did not write the spec or its notes. Primary sources were re-read directly: papers as full text, blogs and advisories as fetched pages, the Project Zero sheet as an export, ECMA-262 `spec.html` at `main`, test262.fyi `index.json` and `meta.json`, the TC39 proposals repository and notes, `WebAssembly/spec`, and the GitHub security-advisory API. Cloudflare blocked four Mozilla and Edge pages, so their Wayback copies were read. No personal data was sent in any request.

Verdicts: **Correct**; **Partly** (the substance holds but a number, scope or wording is off); **Wrong**; **Unverifiable** (the cited source does not contain the claim, and no other source was found). "Must-fix" marks a finding that a decision (JS-D1 to JS-D14) or a target depends on, or a plain factual error. Every other fix is recommended.

## 0. Links and fragments

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| All 119 distinct external URLs resolve | Correct (118); Partly (1) | All 119 were checked with `curl -L`. 118 returned HTTP 200. `doi.org/10.1145/3385412.3385973` returns 403 to scripts, but its handle resolves to `dl.acm.org/doi/10.1145/3385412.3385973` (doi.org handle API). Bugzilla pages return a bot wall to plain GET; the REST API confirms that bugs 1781425 and 1163268 exist and contain the quoted text | None. Optionally add the arXiv copy (2005.10554) next to the DOI |
| No soft 404s among the 2025–2026 and secondary links | Correct | The page titles of 20 at-risk links matched their citations: Ladybird newsletter March 2026, `cfallin.org` aegraph, "Introducing the JetStream 3 Benchmark Suite", GCVE CVE-2026-34987, P0 "Déjà vu-lnerability", Ladybird "adopts Rust", Nova "Garbage collection is contrarian", the IE9 Wayback page, Wingo 2025, Wasm 3.0, SpecTec, the LibJS "Remove JIT compiler" commit, PLDI 2017, rust#112788, static roots, P0 RCA index and The Register | — |
| Every `#fragment` exists (5 fragments) | Correct | Fragment ids checked in the fetched HTML: `webappapis.html#perform-a-microtask-checkpoint`, `#realms-and-their-counterparts`, `webidl#js-legacy-platform-objects`, `ecma262#sec-weakref-processing-model` and `#sec-math.random` | — |
| Repository links resolve | Partly | 11 of 13 targets exist: the six notes, HGBA-2, HGBA-3, Stage 1, the DevTools spec and `../m0/determinism_crossplatform.md`. `[SS]` (`Izcal security scope.md`) and `[CS]` (`Izcal coding standard.md`) are missing on this branch. They exist on `spec/security-scope` (draft 0.3) and `spec/coding-standard` (draft 0.3.1), and the header already says so | None now. Re-check after PRs #8 and #9 merge. The cited IDs (SEC-D5, UPD-R5, REC-R6, SEC-D12, TB-4, Q-CS-7, CS-D3's five exit criteria, CS-TY-10/13, CS-V8-9, CS-AS-2/4, CS-PF-4) were found in those branch files |

## 1. Summary paragraph and Section 1 (the problem)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| **Headline: "every in-the-wild JS-engine exploit from 2018 to September 2026 was memory corruption reached through an engine logic error"** | **Partly (overstated). Must-fix** | Re-counted from the [P0 sheet](https://docs.google.com/spreadsheets/d/1lkNJ0uQwbeC1ZTRrxdtuPLCIl7mlUreoKfSIgajnSyY/) ("All" tab, CSV export, 19 Oct 2026). Every JS-engine row is typed "Memory Corruption", so that half holds. The "logic error" half rests on root causes, and the [P0 RCA index](https://googleprojectzero.github.io/0days-in-the-wild/rca.html) has JS-engine RCAs only up to CVE-2022-4262, none later. The 2023–2026 rows (for example CVE-2024-0519 "Out-of-bounds memory access", CVE-2026-11645 "Out of bounds write in V8", CVE-2021-21148 "Heap buffer overflow in V8") have only advisory titles. [V8's sandbox post](https://v8.dev/blog/sandbox) says its bugs are "rarely 'classic' memory corruption", which is not "never". The same paragraph also says "Rust removes the minority of those that are classic memory bugs", which contradicts "every" | Reword: "Every in-the-wild JS-engine exploit with a published root cause (Project Zero's RCAs, 2019–2022) was memory corruption reached through an engine logic error, and V8 says its exploited bugs are 'rarely classic memory corruption'; the 2023–2026 rows have no published root cause." Keep the Rust sentence, now consistent. Also fix SE 1's "Implications" line |
| V8: 32 in-the-wild 0-days, 2020 to September 2026 | Correct | Re-count: 30 rows mention V8, plus CVE-2020-16009 ("Type confusion in TurboFan map deprecation") and CVE-2024-0519 (V8 per Chrome's advisory) = 32; the latest is CVE-2026-87491, patched 2026-09-08 | — |
| 59% of them type confusion | Correct | 19 of 32 rows say "Type confusion" (18 V8-named plus CVE-2020-16009) = 59.4% | — |
| 60% of Chrome's ITW exploits in 2021–2023 were V8 bugs | Correct | [V8 sandbox](https://v8.dev/blog/sandbox): "Of these, 60% were vulnerabilities in V8" | — |
| Scripts take 10–30% of page-load time on typical sites | Partly | [IE Blog 2010](https://web.archive.org/web/2010/http://blogs.msdn.com/b/ie/archive/2010/03/18/the-new-javascript-engine-in-internet-explorer-9.aspx): "typical sites and applications use 10% to 30% of page load time running script". It is a 2010 figure, stated in the present tense | "took 10–30% in Microsoft's 2010 measurement" (correct as written in 2.1) |
| Deopt Explorer needs 12 V8 logging flags | Wrong (count) | [Microsoft](https://devblogs.microsoft.com/typescript/introducing-deopt-explorer/): dexnode's equivalent command line lists 13 flags, "known to change from release to release" | "13 flags" or "a dozen flags" (Sections 1 and 3) |
| A heap snapshot needs about twice the heap and blocks the event loop | Correct | [Node.js](https://nodejs.org/api/v8.html): "requires memory about twice the size of the heap"; "synchronous operation which blocks the event loop" | — |
| Nashorn deprecated as "challenging to maintain" | Correct | [JEP 335](https://openjdk.org/jeps/335) | — |
| V8 needed a forked platform for determinism; predictable mode turns off concurrency | Correct | [flag-definitions.h](https://chromium.googlesource.com/v8/v8/+/refs/heads/main/src/flags/flag-definitions.h): `--predictable` negates `memory_reducer`, `concurrent_recompilation`, `concurrent_sparkplug` and the parallel compile tasks, and implies `single_threaded_gc` | — |
| Izcal-only renderers jitless; SEC-D5 banking container jitless | Correct | `spec/security-scope` draft 0.3: SEC-D5 "Decided (0.3)… (b)… (Izcal-only renderers are jitless regardless, Section 6.7)" | — |

## 2. History (Section 2.1 chronology, 2.2 mismatches)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| Mocha rewritten "to correct original design mistakes…" | Correct | [HOPL](https://www.cs.tufts.edu/~nr/cs257/archive/brendan-eich/js-hopl.pdf) §5 (the HP note read it in full) | — |
| SquirrelFish: polymorphic ICs keyed on structure IDs; regex JIT | Partly | The [SquirrelFish post](https://webkit.org/blog/189/announcing-squirrelfish/) covers only the register, direct-threaded bytecode and the tree-walking costs. ICs and the regex JIT are SquirrelFish Extreme, from a different post | Cite the SquirrelFish Extreme announcement for those features |
| V8's no-interpreter stance was reversed because baseline code took about a third of the heap | Correct | [V8](https://v8.dev/blog/launching-ignition-and-turbofan): "typically occupied almost one third of the overall JavaScript heap"; "nine supported platforms" | — |
| TraceMonkey "was actively getting in the way" | Correct | [Nethercote](https://blog.mozilla.org/nnethercote/2011/11/23/memshrink-progress-report-week-23/) (Wayback) | — |
| Chakra: background JIT; compile time can offset gains | Correct | IE Blog 2010 (Wayback) | — |
| Edge moved to Chromium in 2018 for compatibility and cadence; ChakraCore handed to the community | Partly | [Microsoft 2018](https://blogs.windows.com/windowsexperience/2018/12/06/microsoft-edge-making-the-web-better-through-more-open-source-collaboration/): "better web compatibility", "more frequent cadence". The post does not mention ChakraCore | Source the ChakraCore hand-over separately (ChakraCore repository README) |
| Opera: about 200 engineers on the core product | Unverifiable (press) | HP 6.1 quotes a CEO interview from press; already flagged in 11.2 | None beyond the existing flag |
| B3: LLVM took about three quarters of FTL compile time and "did not model patchpoints or speculation checks" | Partly | [WebKit B3](https://webkit.org/blog/5852/introducing-the-b3-jit-compiler/): "often three quarters" is correct, but "Although LLVM does support these capabilities, it treats them as experimental intrinsics, which limits optimization" | "modelled patchpoints only as experimental intrinsics, which limited optimization" (2.1, 2.2, 5) |
| CacheIR: one IR for ICs in every tier, against duplicated IC code | Correct | [de Mooij](https://jandemooij.nl/blog/cacheir/) | — |
| Riptide: Splay pauses under 3 ms; removing copying was "painful" | Correct | [WebKit Riptide](https://webkit.org/blog/7122/introducing-riptide-webkits-retreating-wavefront-concurrent-garbage-collector/) | — |
| SpiderMonkey Baseline Interpreter: Docs 901 → 676 ms | Correct | [Mozilla Hacks 2019](https://hacks.mozilla.org/2019/08/the-baseline-interpreter-a-faster-js-interpreter-in-firefox-70/) (Wayback): chart text "C++ Interpreter 901 ms, + Baseline Interpreter 676 ms" | — |
| Hermes: AOT bytecode, mmapped, no JIT | Correct | [Meta 2019](https://engineering.fb.com/2019/07/12/android/hermes/) | — |
| Warp: Docs −20%, Speedometer +10–12%, memory −8% | Correct | [Mozilla Hacks 2020](https://hacks.mozilla.org/2020/11/warp-improved-js-performance-in-firefox-83/) (Wayback) | — |
| Cranelift in Firefox: "the wrong shape", bit-rotted | Correct | Bug 1781425, comment 0 (REST API): "agreed between SM and CL to be the wrong shape. Our import of the code base is also old" | — |
| Maglev cut Speedometer energy 10% | Correct | [V8 Maglev](https://v8.dev/blog/maglev): energy "Speedometer -10%" | — |
| V8 sandbox: 1 TB reservation, ~1% cost, "not yet" a strong boundary, Rust would not help | Correct | [V8 sandbox](https://v8.dev/blog/sandbox): "one terabyte"; "around 1% or less"; issues remain "before it becomes a strong security boundary" | — |
| LibJS removed its JIT (2024) with the quoted reason | Correct | [Commit 1d29f90](https://github.com/LadybirdBrowser/ladybird/commit/1d29f9081fc243407e4902713d3c7fc0ee69650c), 2024-02-19 | — |
| AsmInt gave 1.37–1.69× | Partly | [Ladybird March 2026](https://ladybird.org/newsletter/2026-03-31): "Between AsmInt, the new regex engine, and other optimizations… Octane 1.37x… SunSpider 1.69x" | "AsmInt, a new regex engine and other work together gave 1.37–1.69×" (2.1, and the T0 row in 6.3) |
| Turboshaft: compile time halved; whole JS backend and Wasm pipeline migrated | Correct | [V8](https://v8.dev/blog/leaving-the-sea-of-nodes): "compile time got divided by 2"; Wasm "uses Turboshaft throughout its whole pipeline" | — |
| SpecTec adopted; source of Wasm 3.0 (17 September 2025) | Correct | [SpecTec](https://webassembly.org/news/2025-03-27-spectec/); [Wasm 3.0](https://webassembly.org/news/2025-09-17-wasm-3.0/): "the first version of the standard that has been produced with the new SpecTec tool chain" | — |
| 2026: Nova 1.0, Brimstone, Boa 0.21; "conformance reached by small teams" | Partly | nova_vm 1.0.0 on crates.io on 2026-03-15 (correct). Boa 0.21 was released 2025-10-21; 0.22 came on 2026-08-28. Brimstone has no release. On test262.fyi Nova passes 76.92% ([index.json](https://raw.githubusercontent.com/test262-fyi/data/gh-pages/index.json)), so "conformance reached" does not hold for Nova | "2025–26: Nova 1.0 (March 2026), Boa 0.21–0.22, Brimstone (unreleased)"; "Boa reached 95.6%; Nova is at 77%" |
| CVE-2026-34987 (Winch): upper 32 bits assumed clear; guest read up to ~4 GiB outside its memory | Partly | [GCVE](https://db.gcve.eu/vuln/CVE-2026-34987): Winch "on aarch64" (a non-default backend); the guest can access memory "up to 32KiB before… or ~4GiB after" | "Winch on aarch64 (non-default backend): guest read and write from 32 KiB before to ~4 GiB after its memory" (2.1, 5, 6.6) |
| 2.2: Octane-driven work made real sites slower; 44% of shipped JS unused; 50–70% of functions never ran | Partly | [JSMeter](https://www.usenix.org/legacy/event/webapps10/tech/full_papers/Ratanaworabhan.pdf): "as many as 50–70% are not executed during our use of the applications" | "up to 50–70% in the authors' sessions (2010)" |
| 2.2: Wasmtime's 7 GC-reference memory corruptions in Rust | Correct | GitHub security-advisory API: CVE-2021-39216, CVE-2021-39218, CVE-2022-23636, CVE-2022-24791, CVE-2022-31146, and two GC heap-corruption advisories of 2026-10-02 | — |
| 2.2: Pall's argument; Rust `become` still unstable | Partly / Correct | [lua-l msg00742](http://lua-users.org/lists/lua-l/2011-02/msg00742.html) argues that compilers handle interpreter loops badly; the "frames must match the JIT's" half is not Pall's. [rust#112788](https://github.com/rust-lang/rust/issues/112788) is open | Present "frames must match the JIT's" as the spec's own point |
| 2.2: ESMeta runs in TC39's CI | Correct | [esmeta-typecheck.yml](https://github.com/tc39/ecma262/blob/main/.github/workflows/esmeta-typecheck.yml) exists | — |

## 3. Today (Section 3)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| test262.fyi: 53,616 tests; JSC 98.80%, LibJS 97.72%, V8 97.69%, Boa 95.59% | Correct | [index.json](https://raw.githubusercontent.com/test262-fyi/data/gh-pages/index.json): total 53,616; jsc 52,971; libjs 52,393; v8 52,377; boa 51,252 | — |
| About half of exploited V8 bugs were in compilers | Correct | [V8 sandbox](https://v8.dev/blog/sandbox): "roughly half of the bugs… affected one of its compilers" | — |
| Named root causes: CVE-2020-6418, -16009, 2021-30632, -37975, -4102, 2022-1364, -4262 | Correct | [P0 RCA index](https://googleprojectzero.github.io/0days-in-the-wild/rca.html): each title matches the class given | — |
| CVE-2024-2887: Wasm type-count limit aliased an internal type; RWX Wasm page | Correct | [ZDI](https://www.zerodayinitiative.com/blog/2024/5/2/cve-2024-2887-a-pwn2own-winning-bug-in-google-chrome): "kV8MaxWasmTypes… aliasing one of these internal types"; "resides in an RWX… page" | — |
| Incomplete patches produced variants | Correct | [P0 Déjà vu-lnerability](https://projectzero.google/2021/02/deja-vu-lnerability.html) | — |
| VeRA verified Firefox's range analysis and found a bug latent six years | Correct | [VeRA](https://cseweb.ucsd.edu/~dstefan/pubs/brown:2020:vera.pdf): "a new, confirmed bug that has existed in the browser for six years" | — |
| Icarus verifies every IC stub a DSL can emit | Correct | [Icarus](https://www.cs.utexas.edu/~hovav/dist/icarus.pdf) | — |
| Arrival: 97.8% of in-scope AArch64 lowering **rules** reachable from Wasm, in 11 minutes, with ASL semantics | Partly | [Arrival](https://www.contrib.andrew.cmu.edu/~bparno/papers/arrival.pdf): "verifies 97.8% of the in-scope instruction lowering **paths**" in 11 minutes; semantics derived from ASL | "lowering paths" (3, and the JS-11 context) |
| VeriWasm 1.7 s median | Correct | [VeriWasm](https://cseweb.ucsd.edu/~dstefan/pubs/johnson:2021:veriwasm.pdf): "median time of 1.7" s, per module | Optional: "per module" |
| TurboTV: translation validation only, no bug count in the abstract | Partly | [TurboTV](https://prosys.kaist.ac.kr/publications/icse24.pdf): no V8 bug count; the abstract reports one new LLVM miscompilation | Optional: add that |
| Speedometer 3 governed by all three engine teams; Safari's 60% gain came largely from style, layout and DOM | **Partly. Must-fix** | [WebKit Speedometer 3.1](https://webkit.org/blog/15249/speedometer-3-1/): "~60% between Safari 17.0 and Safari 17.4", with large JavaScriptCore sections (megamorphic and call ICs, JSON, inlining, GC) next to DOM, style and layout. It gives no breakdown, and it does not state the governance | "Safari's ~60% came from work across JavaScriptCore, DOM, style, layout and rendering"; cite the Speedometer 3 announcement for governance. JS-21's "mostly not a JS-engine benchmark" should then rest on Izcal's own time accounting, not on this post |
| JetStream 3 (31 March 2026): First, Worst and Average scored separately; adds 12 Wasm workloads | Partly | [BrowserBench](https://browserbench.org/announcements/jetstream3): date and scoring correct; "now including twelve such workloads" is a total | "includes 12 Wasm workloads" |
| Snapshots cost 2× heap and minutes | Correct | [V8](https://v8.dev/blog/speeding-up-v8-heap-snapshots): a 500 MB snapshot took "over 30 minutes" | — |

## 4. First principles (Section 4)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| ECMA-262: about 14,480 numbered steps in 2,302 algorithm blocks | Correct | Re-counted in [spec.html](https://raw.githubusercontent.com/tc39/ecma262/main/spec.html) on 19 Oct 2026: 2,302 `<emu-alg>` and 14,480 numbered lines | — |
| 56 implementation-defined, 47 implementation-approximated and 50 host-defined points | Correct | Same file: 56 / 47 / 50 occurrences (occurrences, not distinct points) | Optional: "occurrences" |
| ESMeta in TC39's CI with 16 known exceptions | Correct | [esmeta-ignore.json](https://github.com/tc39/ecma262/blob/main/esmeta-ignore.json) has 16 entries | — |
| **"17 host hooks"** (4.2, the 6.0 diagram, 6.14) | **Wrong. Must-fix** | spec.html has **16** clauses with `type="host-defined abstract operation"`: HostEnsureCanAddPrivateElement, HostMakeJobCallback, HostCallJobCallback, HostEnqueueGenericJob, HostEnqueuePromiseJob, HostEnqueueTimeoutJob, HostEnqueueFinalizationRegistryCleanupJob, HostGetImportMetaProperties, HostFinalizeImportMeta, HostLoadImportedModule, HostGetSupportedImportAttributes, HostEnsureCanCompileStrings, HostHasSourceTextAvailable, HostResizeArrayBuffer, HostGrowSharedArrayBuffer and HostPromiseRejectionTracker. SP 1.6's 17th, **HostEventSet**, is an ordinary `type="abstract operation"` in the memory model ("Return a new Set containing all elements of EventSet(_execution_) that are not in SharedDataBlockEventSet(_execution_)"), not a host hook | "16 host hooks at `main`; Temporal adds HostSystemUTCEpochNanoseconds when its PR (#3966) merges". Fix SP 1.6 as well |
| HTML owns job scheduling; microtask checkpoint | Correct | [HTML](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint) | — |
| Wasm 3.0 (17 Sept 2025) adds GC, typed references, tail calls, exceptions, memory64, relaxed SIMD and a deterministic profile; written in SpecTec | Correct | [Wasm 3.0](https://webassembly.org/news/2025-09-17-wasm-3.0/): all listed, plus multiple memories, custom annotations and JS String Builtins. Status is the "live" standard; W3C REC status was not claimed by the spec and was not checked | — |
| ECMA-262 §9.9 weak-reference rules (atomic clearing, `[[KeptAlive]]`, host-scheduled cleanup) | Correct | [§9.9](https://tc39.es/ecma262/#sec-weakref-processing-model) | — |
| Register bytecode: ~47% fewer executed instructions for ~25% more bytecode | Correct | [VEE 2005](https://usenix.org/legacy/events/vee05/full_papers/p153-yunhe.pdf): "more than 47%… eliminated"; "roughly 25% larger" | — |
| Speculation: failure costs thousands of ns against ~1 ns of gain | Correct | [Pizlo](https://webkit.org/blog/10308/speculation-in-javascriptcore/): C = 2,499 ns (DFG) and 9,998 ns (FTL), "three orders of magnitude" | — |
| **"A heap sandbox is retrofitted… V8's took years and about 4 engineers over two years of GC work alone"** | **Wrong. Must-fix** (also 9.3) | [Wingo 2025](https://wingolog.org/archives/2025/11/13/the-last-couple-years-in-v8s-garbage-collector): "there have been about 4 FTE from Google over this period" for **all** of V8's GC (1,600 `src/heap` commits). The sandbox was "around 20% of the time", from what the author calls "some made-up numbers". So the sandbox's GC-side share was about 0.8 FTE, as an outsider's guess | 4.4: "V8's sandbox took years; one observer estimates about a fifth of V8's ~4-engineer GC effort over two years". 9.3: same, and note "the author's 'made-up numbers'" |
| Pointer compression cut V8's heap "up to 43%" | Partly | [V8](https://v8.dev/blog/pointer-compression): "reduces V8 heap size up to 43%… renderer process memory up to 20% on Desktop"; unboxed doubles lost | Write "up to 43%" and "up to 20%" (4.4, 5, 6.1, JS-D1) |
| NaN-boxing relies on 47-bit user addresses | Partly | [SM Value.h](https://github.com/mozilla/gecko-dev/blob/master/js/public/Value.h): "objects use only 47 bits", `JSVAL_TAG_SHIFT 47`. That is SpiderMonkey's case; JSC's encoding assumes 48 | "47- or 48-bit user addresses (SpiderMonkey, JSC)" |
| Constant blinding was bypassed; full blinding costs up to 80% | Correct | [NDSS 2015](https://www.ndss-symposium.org/ndss2015/ndss-2015-programme/devil-constants-bypassing-defenses-browser-jit-engines/) | — |
| ReDoS is common; linear-time JS lookarounds exist | Correct | [Davis et al.](https://davisjam.github.io/files/publications/DavisCoghlanServantLee-EcosystemREDOS-ESECFSE18.pdf); [arXiv 2311.17620](https://arxiv.org/abs/2311.17620) | — |
| Arrival quote "lacks the assurance of verification" | Correct | Arrival §1 | — |

## 5. Lessons (Section 5)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| **Deegen: generated interpreter 28–31% faster than LuaJIT's hand assembly; baseline at 19.1 M bytecodes/s, citing the 2023 post** | **Wrong (28–31%). Must-fix** | [Xu 2022-11-22](https://sillycross.github.io/2022/11/22/2022-11-22/): "28% faster than LuaJIT interpreter"; "31" is from "31 out of the 34 benchmarks". [Xu 2023-05-12](https://sillycross.github.io/2023/05/12/2023-05-12/): "19.1 million Lua bytecodes per second", and no 28% figure | "28% faster (geometric mean; faster on 31 of 34 benchmarks)", citing the 2022 post for that number (Section 5 and the T0 row in 6.3) |
| CPython 3.14 tail-call gain was mostly a compiler regression | Correct | [Elhage](https://blog.nelhage.com/post/cpython-tail-call/) (read by the EX note; not re-fetched beyond HTTP 200) | — |
| V8 2019 `CheckBounds` hardening was bypassed | Correct | [doar-e](https://doar-e.github.io/blog/2019/05/09/circumventing-chromes-hardening-of-typer-bugs/) | — |
| JScript: four exploited variants of one "not tracked by the GC" cause | Correct | [P0](https://projectzero.google/2021/02/deja-vu-lnerability.html): "the fourth vulnerability… exploited in this bug class in 2 years" | — |
| Servo: rooting safety rested on a lint | Correct | [Mozilla Research 2014](https://research.mozilla.org/2014/08/26/javascript-servos-only-garbage-collector/) | — |
| **Nova: ~800 bind/unbind sites** (also 6.5 and the Section 10 risk) | **Unverifiable. Must-fix** | The cited [Nova post](https://trynova.dev/blog/garbage-collection-is-contrarian) has no count, only a "soup of… bind(nogc) and… unbind() calls"; its FOSDEM 2025 talk has no 800 either. The variance claim is correct: covariant handles "do not work", and the contravariant approach is "currently unsound/incomplete, requiring runtime checks" | Drop "~800" or cite a real count (for example, count the call sites in a tagged Nova release). The Section 10 trigger (2,000† hand-written sites) is set relative to it, so restate its basis |
| gc-arena: "mutation xor collection", needs a stackless interpreter | Partly | The [gc-arena](https://github.com/kyren/gc-arena) README has "Mutation xor collection"; the stackless point comes from the Nova post's 11 Jan 2026 update | Cite Nova for the stackless point |
| Chromium issue 1144662: conservative scanning gave an address oracle; WontFix | Correct | [The Register](https://www.theregister.com/2021/02/26/chrome_aslr_bypass) (press, already flagged) | — |
| Riptide and Oilpan non-moving with conservative stacks; V8 had not shipped it after a decade | Correct | Wingo 2025: "I don't think any of this is shipping yet" (commentary, already flagged) | — |
| LXR: short STW pauses beat concurrent evacuation; pauses are not latency | Correct | [LXR](https://www.steveblackburn.org/pubs/papers/lxr-pldi-2022.pdf): "low pauses do not translate into low application latency" | — |
| Replay.io: finalizers broke replay; the recorder leaks finalizable objects | Correct | [Replay](https://www.replay.io/blog/effective-determinism): "We currently deal with these cases by simply leaking the associated objects" | — |
| JARDIS: time travel at <2% overhead | Correct | [FSE 2016](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/09/TTNode.pdf): "under 2%" (record mode; memory overhead was 27–71%) | Optional: add the memory figure |
| Node CVE-2017-11499: snapshots froze the hash seed | Correct | [Red Hat 1475327](https://bugzilla.redhat.com/show_bug.cgi?id=1475327) | — |
| Edge SDSM and Chrome's policy: JIT off lost Wasm | Partly | The [SDSM post](https://microsoftedge.github.io/edgevr/posts/Super-Duper-Secure-Mode/) never mentions WebAssembly. [Microsoft Learn](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-security-browse-safer) says Wasm "is now supported in enhanced security mode" through an interpreter that "might not yield the expected level of performance" | Cite Microsoft Learn: "Wasm was first unsupported, then ran on a slower interpreter" (5, 6.8) |
| Ladybird: Rust rejected in 2024 for GC and inheritance; adopted in 2026 with equivalence gating | Correct | [Ladybird](https://ladybird.org/posts/adopting-rust/) | — |

## 6. Design (Section 6)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| **6.1/JS-D1: ×8-scaled decompression "both AArch64 and x86-64 express in one addressing mode"** | **Partly. Must-fix** | Checked with the system assembler (`clang -arch arm64`). `ldr w0, [x1, w2, uxtw #3]` is rejected: "expected 'uxtw' or 'sxtw' with optional shift of #0 or #2". AArch64 register-offset loads scale only by the access size, and they take no displacement. On AArch64, decompression is one `ADD Xd, Xbase, Wm, UXTW #n` followed by a load with an immediate, which is the same count as V8's unscaled scheme. x86-64 folds it into one `[base + idx*s + disp]` operand. Also, with the low-bit tag, the stored slot is `offset<<1 \| 1`, so the scale is ×4 with a −4 displacement, not ×8 | "On x86-64 one memory operand; on AArch64 one extended-register ADD before the load, as for V8's byte offsets". Keep it as a first-principles claim measured at the JS-M0 spike |
| 16 GB cage from 31-bit offsets in 8-byte units | Correct | 2^31 × 8 B = 16 GiB (arithmetic) | — |
| V8 static roots ~+1% on Speedometer 2 | Correct | [V8](https://v8.dev/blog/static-roots): "about 1%… on an M1 CPU" | — |
| Wasm GC struct field types (i8/i16 packed, i32, i64, f32, f64) | Correct | [GC MVP](https://github.com/WebAssembly/gc/blob/main/proposals/gc/MVP.md) | — |
| Off-thread parse and compile: ~40% less main-thread time | Correct | [V8](https://v8.dev/blog/cost-of-javascript-2019): "reduced… by an average of 40%" | — |
| Copy-and-patch compiled 4.9–6.5× faster than Liftoff | Correct | [arXiv 2011.13127](https://arxiv.org/abs/2011.13127): "generates code 4.9×–6.5× faster than Liftoff" | — |
| Truffle reached 0.83× V8 | Correct (link weak) | Paper text: "our speedup relative to the V8 JavaScript VM is 0.83". The linked [PLDI event page](https://pldi17.sigplan.org/event/pldi-2017-papers-practical-partial-evaluation-for-high-performance-dynamic-language-runtimes) carries no abstract | Link the paper PDF |
| JSC tier-up thresholds scale with function size; exponential back-off; OSR exit preferred | Correct | Pizlo: thresholds "scale by the square root of the size"; "each recompilation doubles the thresholds" | — |
| V8 needed Maglev; TurboFan's frontend moving onto Maglev's graph builder | Correct | Leaving the Sea of Nodes: the frontend "we're replacing by Maglev" | — |
| Cranelift aegraph: ~2% better code for 7–8% more compile time | Correct (HTTP and title checked; the figure is from the EX note) | [Fallin 2026](https://cfallin.org/blog/2026/04/09/aegraph/) | — |
| Wasmtime CVE-2023-26489 is a lowering error | Correct | MITRE: "address-mode computation mistakenly would calculate a 35-bit effective address" | — |
| Range-analysis class example "CVE-2025-4919" | Partly | MITRE: Firefox, "Out-of-bounds access when optimizing linear sums"; a Pwn2Own Berlin 2025 entry, so SpiderMonkey, not in the wild | "SpiderMonkey CVE-2025-4919 (Pwn2Own 2025)" |
| Alive2 is bounded translation validation | Correct | [Alive2](https://users.cs.utah.edu/~regehr/alive2-pldi21.pdf), title | — |
| Generational ZGC used ¼ of the heap for 4× throughput | Correct | [JEP 439](https://openjdk.org/jeps/439): Cassandra, against non-generational ZGC | Optional: add the context |
| Immix +7–25% | Correct | [Immix](https://www.steveblackburn.org/pubs/papers/immix-pldi-2008.pdf): "7 to 25% on average across 20 benchmarks" | — |
| Field-logging barrier 1.6%; read barriers 5.4% against 0.9% for write barriers | Correct | LXR §5.3; [ISMM 2012](https://www.steveblackburn.org/pubs/papers/barrier-ismm-2012.pdf): "as low as 5.4% and 0.9%" | Optional: "as low as" |
| Background sweeping cut Oilpan main-thread sweep time ~42% | Correct | [V8](https://v8.dev/blog/high-performance-cpp-gc): "25%-50% (42% on average)" | — |
| V8 idle-time GC: Gmail idle heap −45% | Correct | [V8](https://v8.dev/blog/trash-talk) | — |
| LXR beat G1 by 4% and Shenandoah by 43% | Correct | LXR abstract | — |
| Concurrent copying "throughput fell up to 7× under allocation pressure" (the evidence for rejecting JS-D3 (c)) | Partly | LXR Table 1 is one workload (lusearch, 1.3× heap): "Shenandoah takes a 7× hit on throughput compared to G1" | "7× lower than G1 on lusearch (LXR Table 1)" |
| Cai et al. LBO method (time, cycles, energy) | Correct | [arXiv 2112.07880](https://arxiv.org/abs/2112.07880) | — |
| Conservative scanning costs only 2–3% in a mark-region heap | Correct | [OOPSLA 2014](https://www.steveblackburn.org/pubs/papers/consrc-oopsla-2014.pdf): "within 2 to 3% of their exact counterparts" | — |
| RustBelt states per-library verification conditions | Correct | [RustBelt](https://plv.mpi-sws.org/rustbelt/popl18/paper.pdf) | — |
| Wasm in-place interpreter: side table ~30% of code size against 2–4× for rewriting interpreters | Partly | [Titzer](https://arxiv.org/abs/2205.01183): "about 30% additional space"; rewriting interpreters "can consume up to 4×", others about the same | "up to 4×" |
| Baseline compilers 3–10× faster than interpreters at run time | Correct | [arXiv 2305.13241](https://arxiv.org/abs/2305.13241): "3× to 10× are common" | — |
| Liftoff fixed 30-second startups; dynamic tiering's reason; speculative inlining ~30% on Sheets; V8 Wasm has no OSR | Correct | [Liftoff](https://v8.dev/blog/liftoff); [dynamic tiering](https://v8.dev/blog/wasm-dynamic-tiering); [Wasm GC porting](https://v8.dev/blog/wasm-gc-porting); [V8 docs](https://v8.dev/docs/wasm-compilation-pipeline) | — |
| WasmCert-Coq covers the Wasm 2.0 subset | Correct | [WasmCert-Coq](https://github.com/WasmCert/WasmCert-Coq) README (SP 7.5) | — |
| Pwn2Own 2024 chain wrote shellcode into an RWX Wasm page | Correct | ZDI, as in Section 3 | — |
| V8 CFI plan "reports no regressions in preliminary tests for this direction" (landing pads, shadow stacks) | Partly | [V8 CFI](https://v8.dev/blog/control-flow-integrity): the no-regression result was for validating JIT output, not for landing pads or shadow stacks | Move the citation to JS-D7, where it supports the validator; drop it as evidence for landing pads |
| GTIG: a 2024 WebKit chain targeted only Intel Macs, likely for lack of a PAC bypass | Correct | [GTIG](https://cloud.google.com/blog/topics/threat-intelligence/2024-zero-day-trends) | — |
| V8's Spectre quote | Correct | [V8](https://v8.dev/blog/spectre), exact | — |
| Apple EU: 90% WPT and 80% test262 when JIT is unavailable | Correct | [Apple](https://developer.apple.com/support/alternative-browser-engines/) | — |
| Edge Balanced mode chose by engagement score | Partly | Microsoft Learn says "unfamiliar sites"; "engagement score" is from the Edge VR post (SE 4.3) | Cite the Edge VR post or write "site familiarity" |
| Chrome's heavy-ad intervention is frame-level | Correct | [Chrome](https://developer.chrome.com/blog/heavy-ad-interventions) | — |
| `Math.random` must give each realm a distinct sequence | Correct | [§21.3.2.28](https://tc39.es/ecma262/#sec-math.random) | — |
| **6.9 and JS-D12: NaN bit patterns and relaxed SIMD are "deterministic per ISA" and differ across ISAs** | **Partly. Must-fix** | **The NaN half is correct.** An instruction with no NaN input that produces one returns `0xFFF8000000000000` (sign set, Intel's "QNaN floating-point indefinite") on x86-64 and `0x7FF8000000000000` (Arm's default NaN, sign clear) on AArch64. This was confirmed by running `0.0/0.0` natively on arm64 and as an x86-64 binary under Rosetta, with float32 `0xFFC00000` against `0x7FC00000`. The [Wasm design rationale](https://raw.githubusercontent.com/WebAssembly/design/main/Rationale.md) states the same, and adds that propagation differs when several inputs are NaN. **"Per ISA" is not enough on its own, for two reasons.** (1) Relaxed-SIMD results on x86-64 depend on CPU features, such as whether FMA is used for `relaxed_madd`, unless the engine fixes one lowering per ISA. (2) A T2 constant fold computed in Rust yields the positive NaN on every host, unlike T0's hardware result on x86-64, so tiers could disagree on one ISA, against JS-51. Wasm 3.0 also defines a **deterministic profile** that fixes NaNs and relaxed-SIMD results; that is option (a) | State the two requirements: one relaxed-SIMD lowering per ISA, independent of CPU features; and constant folding and runtime helpers that reproduce the ISA's NaN rules in every tier. Cite Intel SDM Vol. 1 §4.8.3.7 and Table 4-7, the Arm ARM (FPCR.DN, default NaN) and the Wasm rationale. Note that option (a) is Wasm 3.0's deterministic profile. The added cost of (a) is unmeasured, so mark it as such |
| V8 predictable mode switches off concurrent compilation, background GC and the memory reducer | Correct | `flag-definitions.h`, lines 4261–4277 at `main` | — |
| JS Self-Profiling cross-origin limits | Correct (HTTP 200; not re-read) | [WICG](https://wicg.github.io/js-self-profiling/) | — |
| Snapshot: context creation 40 ms → <2 ms; code caching −20–40% | Correct | [V8 snapshots](https://v8.dev/blog/custom-startup-snapshots); [code caching](https://v8.dev/blog/improved-code-caching) | — |
| AWS SnapStart uniqueness guidance | Correct | [AWS](https://docs.aws.amazon.com/lambda/latest/dg/snapstart-uniqueness.html) | — |
| **Watt et al. "already caught a broken ARMv8 compilation scheme"** | **Wrong. Must-fix** | arXiv 2005.10554 (the DOI's paper): the **model** was broken: "in violation of the design intention, does not support a compilation scheme to ARMv8 which is used in practice" | "…which found that the JS memory model failed to support the ARMv8 compilation scheme used in practice, and repaired it" |
| Shared structs at TC39 Stage 2 | Correct | [proposals README](https://github.com/tc39/proposals/blob/main/README.md): "Structs: Fixed Layout Objects…" under Stage 2 | — |
| Strings were 70% of Facebook's heap (MemLab) | Correct (vendor; HTTP 200; figure from GC 11.1) | [Meta](https://engineering.fb.com/2022/09/12/open-source/memlab/) | — |
| **6.13: "V8 shipped the opposite default (backtracking first, linear after 50,000 backtracks)"** | **Wrong. Must-fix** | [V8](https://v8.dev/blog/non-backtracking-regexp): the fallback is "available behind the feature flags" (default N = 50,000) and "remains experimental because Irregexp is orders of magnitude faster". `flag-definitions.h` still declares `enable_experimental_regexp_engine_on_excessive_backtracks` as `DEFINE_EXPERIMENTAL_FEATURE` | "V8's default is backtracking only; a linear fallback after 50,000 backtracks exists behind an experimental flag, because its linear engine is far slower". The regex alternative row (6.17) still stands |
| ECMA-402 does not require CLDR; it requires IANA tz and BCP 47 | Correct (SP 3.2; HTTP 200) | [ECMA-402](https://tc39.es/ecma402/) | — |
| Decorators moved back to Stage 2.7 in May 2026; ShadowRealm at 2.7 | Correct | [notes, 19 May 2026](https://github.com/tc39/notes/blob/HEAD/meetings/2026-05/may-19.md) "Decorators for Stage 2.7"; proposals README lists both under Stage 2.7 | — |
| Temporal at Stage 4, merge PR open | Correct | [notes, 11 March 2026](https://github.com/tc39/notes/blob/HEAD/meetings/2026-03/march-11.md): "Temporal advances to stage 4"; `gh pr view 3966` reports state OPEN; finished-proposals lists it for 2027 | — |
| Dynamic Code Brand Checks, `import defer` and source phase imports at Stage 3 | Correct | proposals README, Stage 3 | — |

## 7. M-1 compliance (Section 7)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| Temporal: 6,715 test262 tests | Correct | [meta.json](https://raw.githubusercontent.com/test262-fyi/data/gh-pages/meta.json) `features.Temporal` = 6,715 | — |
| 258 core `.wast` scripts | Correct | `gh api repos/WebAssembly/spec/git/trees/main?recursive=1`: 258 `test/core/**.wast`, tree not truncated | — |
| ANC-1 denominator about 14,480 steps | Correct | As in Section 4 | — |
| ESMeta, SpecTec and engine262 as oracles | Not a factual claim | Owner classification (JS-D9) | — |

## 8. Targets (Section 8), including the † audit

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| JS-20: Sparkplug about 1.45× and TurboFan about 4.35× Ignition on JetStream 2 | Correct | [V8 Maglev](https://v8.dev/blog/maglev): "Sparkplug improves performance… compared to Ignition (+45%)"; "TurboFan helps V8 run the suite 4.35x as fast". 1.45 / 4.35 ≈ 0.33 supports the 0.3×† Stage 1 estimate, which the spec labels an estimate | — |
| JS-5: test262 covered 91.6% of statements (2021) | Correct | [JEST](https://arxiv.org/abs/2102.07498): 22,440 of 24,495 ES11 algorithm statements | Optional: "of ES11 algorithm statements" |
| JS-21 rests on "Speedometer is mostly not a JS-engine benchmark" | Partly | See Section 3 | See Section 3's must-fix |
| JS-24: "with its JIT disabled (enterprise policy or Lockdown Mode)" | Correct in substance | SE 4.3–4.4 (Edge policy; Chrome's `DefaultJavaScriptJitSetting`) | — |
| **† audit: no invented number presented as measured** | Correct, with three exceptions | Every threshold set by the spec in Sections 6.17, 8, 9.4, 10 and 11 carries †. The unmarked numbers are spec- or policy-derived (100%, 0, "identical", 1,000 of 1,000 from DET-1, 258 `.wast`). The three exceptions follow in the next rows | — |
| † exception 1: "300–350 engineer-years†" from 50–58† engineers over 5–7† years | Partly | 50 × 5 = 250 and 58 × 7 = 406; the stated range is narrower than its inputs and gives no reason (ramp-up?) | Show the arithmetic or write "about 250–400†" |
| † exception 2: 4.4 and 9.3 present Wingo's "made-up numbers" as data | Wrong | See Section 4's must-fix | As there |
| † exception 3: "Nova's 800 sites" is a number with no source, used to set the 2,000† risk trigger | Unverifiable | See Section 5's must-fix | As there |

## 9. Milestones and staffing (Section 9)

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| Opera kept about 200 engineers on its core product | Unverifiable (press) | HP 6.1, already flagged | — |
| V8's sandbox took about four engineers over two years, about 20% of V8's GC effort | **Wrong. Must-fix** | See Section 4 | See Section 4 |
| "LibJS, Boa and Brimstone reached 95–98% of test262 with small teams ([SP 4.3])" | Partly | SP 4.3 has LibJS 97.72% and Boa 95.59%. Brimstone is not on test262.fyi; its README self-reports ">97% of test262 language tests" (HP 8.6) | "LibJS and Boa reach 96–98% on test262.fyi; Brimstone self-reports over 97% of the language tests (HP 8.6)" |
| Stage 1 design 8.3's 23–30.5 engineers did not include an engine | Not checked | Design critic's scope | — |

## 10. Risks and Section 11

| Claim | Verdict | Evidence | Required fix |
|---|---|---|---|
| Wasmtime advisories rose to 51 in one year | Correct | GitHub API: 2021: 4, 2022: 8, 2023: 4, 2024: 4, 2025: 4, 2026: 51 (to 19 Oct) | "51 in 2026 to date" |
| AI-scale auditing found hundreds of engine bugs in 2026 | Not re-checked | SE 7.4, carried from the security scope's fact-checked note 8.5–8.8 | — |
| 11.2: "x86-64 and AArch64 produce different default NaN bit patterns" (unchecked in the session) | Correct | See the 6.9 row: confirmed by test and by the Wasm rationale | Move the item from "unchecked" to checked, with the two added requirements |
| 11.2: unchecked dates: ES2026 GA approval; Wasm 3.0 W3C status | Unverifiable (not checked here either) | — | Keep flagged |

## Summary

Counted over the rows above (Section 0's link rows included):

| Verdict | Count |
|---|---|
| Correct | 106 |
| Partly | 29 |
| Wrong | 8 |
| Unverifiable | 5 |
| Not checked or not a factual claim | 3 |
| **Total** | **151** |

(Rows with a split verdict are counted by their first word.)

Links: 119 external URLs checked, all resolve (one DOI is bot-blocked but valid); all 5 fragments exist; 11 of 13 repository targets exist, and the two missing ones are the in-review documents the header names.

### Must-fix list

1. **Headline "every exploit was a logic error"** (summary, SE 1 implications). Root causes are public only for the 2019–2022 RCAs; 2023–2026 rows have advisory titles only, and "every" contradicts the same paragraph's "minority of classic memory bugs". Restate as "every exploit with a published root cause".
2. **"17 host hooks" → 16** (4.2, the 6.0 diagram, 6.14, SP 1.6). HostEventSet is an ordinary abstract operation; Temporal's HostSystemUTCEpochNanoseconds makes 17 only after PR #3966 merges.
3. **V8 sandbox staffing** (4.4, 9.3). The ~4 FTE is all of V8's GC work, and the sandbox's ~20% share is the author's self-described "made-up numbers". Restate, and drop it as a measured input to staffing.
4. **Deegen "28–31%" → 28%** (5, 6.3); 31 is a benchmark count, and the figure belongs to the 2022 post.
5. **Watt et al.** (6.12): the JS memory model was what was broken, not the ARMv8 compilation scheme.
6. **V8 regex default** (6.13): V8 ships backtracking only; the 50,000-backtrack linear fallback is an experimental flag.
7. **Nova "~800 sites"** (5, 6.5, Section 10): not in the cited source. Remove it or measure it, and rebase the 2,000† trigger.
8. **JS-D12 and 6.9 NaN and relaxed SIMD.** The default-NaN difference is confirmed. Per-ISA determinism also needs one relaxed-SIMD lowering per ISA, independent of CPU features (FMA), and constant folding that reproduces hardware NaNs in every tier; otherwise JS-50 and JS-51 fail on one ISA. Say that option (a) is Wasm 3.0's deterministic profile.
9. **JS-D1 decompression "one addressing mode on both ISAs"** (6.1, 11.2). False for AArch64 (assembler-checked): it needs an extended-register ADD before the load, and the tag bit makes the scale ×4 with −4. Restate; the cost is still V8-equivalent and still measured at JS-M0.
10. **Speedometer 3.1 "Safari's 60% came largely from style, layout and DOM"** (3, JS-21). The post shows large JSC work and no breakdown. Restate, and base JS-21's "not a JS-engine benchmark" on Izcal's own time accounting.
11. **Chronology row "Nova 1.0, Brimstone, Boa 0.21 (2026)… conformance reached"** (2.1). Boa 0.21 was released in 2025, Brimstone has no release, and Nova is at 77% on test262.fyi.

Recommended (not blocking): Deopt Explorer 13 flags; Winch is aarch64-only and non-default, with read and write from −32 KiB to +4 GiB; AsmInt shares its speedup with other work; Arrival "paths", not "rules"; LXR's 7× is one workload; JetStream 3 "includes" 12 Wasm workloads; Edge SDSM's Wasm loss re-sourced to Microsoft Learn; V8 CFI result moved to JS-D7; B3 "experimental intrinsics"; "up to 43%"; 2010 date on the 10–30% script share; CVE-2025-4919 labelled SpiderMonkey and Pwn2Own; SquirrelFish Extreme citation; Brimstone's figure from HP 8.6; 250–400† engineer-years arithmetic; Truffle paper link; ChakraCore hand-over source; "47- or 48-bit" addresses; Titzer "up to 4×"; JSMeter "up to".
