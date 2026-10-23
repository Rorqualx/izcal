# Izcal JavaScript engine: frame

Written 19 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Izcal needs an engine that runs ECMAScript and WebAssembly for every web page, the shell's own UI, DevTools and extensions. HGBA-2 2.3.0 (M-1 part 1) excludes V8 and every other engine library, so Izcal builds its own. The owner's direction (19 October 2026): **no shortcuts for time or complexity; the best engine from first principles, "like nothing before".** Cost is recorded as a planning fact, never as a reason for a lesser design.

Who has the problem:
- **Web users**: pages must be correct (test262, WPT), fast (Speedometer 3, JetStream, real-page p95 latency) and safe. JS engines have been the largest source of exploited browser bugs (V8: 21 of 37 Chrome in-the-wild 0-days 2022–Sept 2026, security scope 0.3).
- **Web developers**: debugging, profiling, memory and performance visibility (DevTools spec).
- **Izcal's own engineers and agents**: the engine must be testable deterministically (DST, HGBA-3 B1), verifiable, and maintainable by a small team with AI assistance.

## What Izcal's specs already decide or constrain (read these)

- HGBA-2 (`reports/HGBA 2 browser architecture spec.md`, 2.3.0): principles P1–P6 in precedence order (security > compatibility > robustness > performance > hardware > evidence); M-1 (original, spec-anchored, first principles; no engine libraries); I-1/I-2 (site isolation, zero ambient authority); Section 4.2 (page-facing Wasm runs inside the JS engine because Wasm GC, JSPI and WebIDL need shared GC objects; internal components in a separate Wasm host) — the V8-specific parts of 4.2, D2, S-4b/c/e and Q-1 are superseded and to be rewritten from this spec.
- HGBA-3 (`reports/Future software browser engine.md`): B1 deterministic substrate and simulation (DST), B3 script thread/event loop, B4 data plane, B8 semantic plane, B9 agent principal, B10 AI off by default; DET-1/DET-2.
- Stage 1 design (`reports/HGBA Stage 1 architecture design.md`): process map (renderer per site), crate graph G1–G9 (G6 currently names `v8`: to change), 3.3 determinism (the V8 predictable-platform problem disappears with an own engine, but determinism becomes our requirement), 5.1–5.3 script thread, DOM in a GC heap, WebIDL bindings generator (`hg-webidl-codegen`).
- Security scope 0.3 (PR #8, `../security-scope/reports/Izcal security scope.md`): threat model; V8-era items (SEC-1 patch latency, SEC-D5 JIT tiers, SEC-D12) to be redone for an own engine.
- Coding standard 0.3 (PR #9): rules for Rust code, tiers, `unsafe` inventory, Miri, determinism (CS-AS), the CS-D3 TokenCell re-entrancy work (done against V8's cppgc; to be redone against our own GC).
- DevTools spec 0.3: inspection protocol (IIP), recorded sessions, replay.

## What success looks like (measurable)

To be turned into targets in the spec. Candidate dimensions:
- **Correctness**: test262 pass rate (all non-intl-402 features at the pinned edition, then Intl), WPT pass rates for JS-facing APIs, Wasm spec test suite.
- **Security**: memory-safety bug classes eliminated by construction; JIT correctness argued/verified; sandboxing of generated code; zero in-the-wild exploit classes that V8 suffered by construction where possible.
- **Performance**: Speedometer 3 and JetStream 2/3 relative to the best shipping engine on the reference devices; startup; memory per page; energy; GC pause p99.
- **Determinism**: same inputs + seed → same execution under DST, including GC timing and JIT tiering decisions.
- **Visibility**: what the engine can expose to DevTools/agents that no engine does.
