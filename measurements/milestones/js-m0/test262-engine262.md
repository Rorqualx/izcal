# JS-M0: hg-test262 validated on engine262, the full suite

Measured 31 October 2026 on the Mac (Huey the Sixth). This is the validation of `hg-test262`, Izcal's test262 runner (`crates/hg-test262`), at JS-M0. Izcal has no parser yet, so ORC-1's oracle is checked against an existing engine, engine262. engine262 was used strictly as a black-box, test-only oracle: it ran under Node through its documented CLI, was never linked or vendored, and its source was not read (HGBA-2 M-1 part 3). At JS-M1, Izcal's engine plugs into the same `Engine` adapter trait.

## What ran

| | |
|---|---|
| test262 | `91c70759363c46557b694792bf34a2551a7af5ba` (main, 29 Oct 2026; the revision test262.fyi ran on 31 Oct 2026); archive SHA-256 `9869ee47ade3ed1f4d08c06e1b9a7c7e84391394575e2267064335917decb903` |
| engine262 | `d553e7b52501fbc47eae981c8d0fa7a5c26d0171` (main, 23 Oct 2026). The npm package `@engine262/engine262@0.0.1-d553e7b…` is what its CI publishes for that commit; tarball SHA-256 `1966b1f909d522741c56c394973c4c5c4d70f4e7cb70b74b511cea8fe76c5018` |
| Pins | `tools/test-data.toml`; fetched and checked by `tools/fetch-test-data.sh` into `~/.cache/izcal/test-data` (outside the repository) |
| Host | Node v26.13.0; engine262 flags `--no-inspect --features=immutable-arraybuffer,decorators,cleanup-some,regexp-buffer-boundaries,export-defer` (every feature its `--help` lists except `decorators.no-bugfix.1`); Node heap capped at 2 GiB (`--max-old-space-size=2048`) |
| Machine | Apple M6, 12 CPUs, 32 GB, macOS 27.0.1. Heavy booking b-9e650f. The machine was shared: another session held a light booking, and the load average was about 23. The elapsed time below is not a benchmark |
| Command | `hg-test262 run --engine engine262 --no-exclusions --jobs 8 --timeout 120 --ledger full.ledger`, run under `rig run local --kind heavy` |
| Time | 13:18–16:34 MDT, 11,756 s |
| Scope | all 53,751 test files (`_FIXTURE` files are not tests), 103,264 runs (both strictness modes unless flags restrict). Run with no exclusion list, so nothing was skipped except tests needing host capabilities the CLI lacks (below). The engine's exclusion list is applied afterwards, in a restatement |

The front matter of every one of the 53,751 files parses (`hg-test262 frontmatter`: 0 refused). The flag counts agree with an independent count of the files: 32 `raw`, 843 `module`, 5,624 `async`, 7 `CanBlockIsTrue`, 2 `CanBlockIsFalse`, 4,754 `negative`.

## Results: per directory, every test run

A file passes when every run it has passes. Runs: 94,012 pass, 8,930 fail (51 of them timeouts, 43 host crashes), 322 excluded.

| Directory | Files | Pass | Fail (timeout) | Excluded | Pass / ran | Pass / all |
|---|---:|---:|---:|---:|---:|---:|
| annexB | 1086 | 161 | 890 (0) | 35 | 15.32% | 14.83% |
| built-ins | 23852 | 23191 | 542 (20) | 119 | 97.72% | 97.23% |
| harness | 116 | 116 | 0 (0) | 0 | 100.00% | 100.00% |
| intl402 | 3382 | 96 | 3286 (0) | 0 | 2.84% | 2.84% |
| language | 23828 | 23778 | 50 (0) | 0 | 99.79% | 99.79% |
| staging | 1487 | 1398 | 82 (12) | 7 | 94.46% | 94.01% |
| **all** | 53751 | **48740** | 4850 (32) | 161 | 90.95% | **90.68%** |

**Correction after the run.** One adapter bug was found in this run and fixed. A SyntaxError thrown by GlobalDeclarationInstantiation (`let undefined;`) happens before any statement runs, so the adapter labelled it a parse error. test262 expects it in the runtime phase. Only `language/global-code/decl-lex-restricted-global.js` (2 runs) was affected. After the fix, a rerun of 3,312 negative-test-heavy files (`language/global-code`, `let`, `const`, class element syntax, literals, `module-code`, `dynamic-import`, `eval-code`) changed exactly those 2 runs, from fail to pass, and nothing else (`hg-test262 compare`: 0 regressions). The corrected totals are **48,741 files pass (90.68%)**, and `language` is 23,779 / 23,828 (99.79%).

Excluded at run time, as host gaps (`HostCapabilities`), never as failures: 161 files. The CLI's `$262` has no `IsHTMLDDA` and no `agent`, and there is no `Atomics`, so CanBlockIsTrue cannot apply.

## Restated under engine262's exclusion list

`crates/hg-test262/exclusions/engine262.txt` lists what engine262 does not implement. Each entry was probed through the CLI:

- `typeof Intl` is `"undefined"`, so `intl402/` and `staging/intl402/` are skipped (INTERPRETING.md, "Implementations without ECMA-402").
- There is no `SharedArrayBuffer` and no `Atomics` global.
- Annex B is mostly absent: no `escape` or `substr`, no HTML-like comments, no block-function hoisting.
- There are no proper tail calls: 34 of the 35 `tail-call-optimization` runs exhaust Node's stack.
- The host resolves no `<module source>` specifier.

```text
hg-test262 summary full.ledger --exclusions crates/hg-test262/exclusions/engine262.txt
```

The restatement below was taken before the last two entries (tail calls, module source) were added:

| Directory | Files | Pass | Fail (timeout) | Excluded | Pass / ran |
|---|---:|---:|---:|---:|---:|
| built-ins | 23852 | 23191 | 58 (20) | 603 | 99.75% |
| harness | 116 | 116 | 0 (0) | 0 | 100.00% |
| language | 23828 | 23778 | 48 (0) | 2 | 99.80% |
| staging | 1487 | 1398 | 77 (12) | 12 | 94.78% |
| annexB, intl402 | 4468 | — | — | 4468 | — |
| **all** | 53751 | 48483 | 183 (32) | 5085 | 99.62% |

## Against independent published figures

**engine262's own CI (the closest figure).** The `test` workflow at the same engine262 commit (run 39942017139, 23 Oct 2026) prints its test262 summary in the job log, which is CI output, not source. Its last line reads `Total 52628, 47470 passed, 5157 skipped, 0 failed`. That run used test262 `419d3e0a` (3 Sep 2026), which is 23 commits and 254 changed files behind our pin.

| | Total | Passed | Not run | Failed | Pass / total |
|---|---:|---:|---:|---:|---:|
| engine262 CI (test262 `419d3e0a`) | 52,628 | 47,470 | 5,157 skipped | 0 | **90.20%** |
| hg-test262, same areas left out (intl402, annexB, SharedArrayBuffer, Atomics) | 53,751 | 48,483 | 5,085 excluded | 183 | **90.20%** |
| hg-test262, everything run | 53,751 | 48,741 | 161 (host) | 4,849 | 90.68% |

The two pass rates agree to the second decimal, and the not-run counts are within 72 (5,085 against 5,157). The remaining differences:

- **Different suites.** Our pin is 23 commits newer, with 254 files changed and 1,123 more tests in total. The CI log does not say whether its "Total" counts files or runs.
- **What is skipped.** engine262's runner keeps its own list of tests not to run; we have not read that list (it is in its source tree). We infer that it covers roughly the same areas from the black-box probes and from the matching counts. That is an inference, not a reading.
- **Our 183 failures.** On the CI side these must be among the skipped tests, since it reports 0 failures. They are 32 timeouts and 9 heap exhaustions (resource limits, below), the 2 phase-fix runs (fixed), and engine gaps: tail calls, `<module source>`, `import-bytes`, a few RegExp identity escapes, DataView/TypedArray edge cases, the `arguments`/`caller` restrictions, and 77 `staging/sm` files (SpiderMonkey-imported staging tests, which engine262's own runner may not run).

**test262.fyi.** engine262 is no longer in test262.fyi's current data (`index.json`, revision `91c7075`). Its `history.json` last lists engine262 on 24 June 2025: 38,308 of 49,169 (77.9%), engine262 `0aaa3ff`. That is 16 months older on both sides, and test262.fyi counts Intl and Annex B as failures. Our "everything run" figure is the comparable one: 90.68% today against 77.9% then. No per-test data for engine262 is published at any revision close to ours, so no per-test diff was possible against either source. The comparison is aggregate only.

## Not conformance: resource limits

- **Timeouts (51 runs, 32 files), at 120 s:**
  - `RegExp/property-escapes/generated` (11);
  - `RegExp/CharacterClassEscapes` negative cases;
  - Temporal `argument-string-limits`;
  - `decodeURI` and `decodeURIComponent` `A2.5_T1`;
  - `staging/sm` Date `dst-offset-caching`.

  engine262 is a spec-literal interpreter, and the machine was loaded.
- **Heap exhaustion (9 runs):** the `CharacterClassEscapes` positive cases and two `staging/sm` files hit the 2 GiB Node heap cap. Rerunning one alone confirmed Node's "Last few GCs" fatal-heap report.

## How the adapter drives engine262 (observed behaviour it relies on)

These are recorded in the `engine262` module docs:

- Scripts run through `$262.evalScript` inside a closure. The constructor name is therefore read from the thrown value itself, and a test that replaces `print` or `$262` cannot hide its result.
- A `print` marker after the strict prefix separates evaluation errors from parse errors. With no marker, a stack with a caret line and no frame of the script is a parse error; otherwise the error is from instantiation.
- A script that names `./` files runs directly as the CLI's file, because evalScript code has no referrer for `import()`.
- A module's harness runs as a global script. The module is then imported from a scratch copy that sits beside copies of its `./` dependencies, and each copy is instrumented with the marker. A marker printed by any module means the error is a runtime one.
- No test at the pinned revision declares a top-level `print` binding, which would put the marker in its TDZ (checked).

## CI

The workflow is unchanged. `cargo test -p hg-test262` (the parser and classification unit tests, plus the medium test of 16 synthetic files through a fake engine) already runs in the `test` job on all three OSes. The full engine262 run took 3.3 h on 8 workers on a loaded Mac and needs network fetches and Node, so it does not fit the 15–20 min PR jobs. It belongs in a scheduled job, with the ledger kept as an artifact and each run compared with the previous ledger (`hg-test262 compare`, which exits 1 on a regression). That job is for the owner to add with the JS-M1 CI tiers.

## Reproduce

```text
tools/fetch-test-data.sh                       # downloads, checks SHA-256, unpacks
cargo build --release -p hg-test262
target/release/hg-test262 run --engine engine262 --no-exclusions --jobs 8 --ledger full.ledger
target/release/hg-test262 summary full.ledger --exclusions crates/hg-test262/exclusions/engine262.txt --features
```
