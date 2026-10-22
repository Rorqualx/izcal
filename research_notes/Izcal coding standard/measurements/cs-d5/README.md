# CS-D5 measurement: gungraun instruction counts on GitHub-hosted runners

*18 October 2026. Measures the claim behind CS-D5 (a) and CS-PF-4: `gungraun` (Valgrind Callgrind) instruction counts on GitHub-hosted Linux runners are deterministic enough for a per-PR gate that fails on more than a 1% increase, and the `perf-gate` job fits its 300 s A.5 budget (CS-T9). The spec was not edited.*

## Conclusion

The data supports **(a)**, with one finding the spec should absorb. **Same code, same commit:** 90 jobs on 6 CPU models (5 x86-64 models and 1 Arm Neoverse model) over 3 workflow runs 41 minutes apart. Every benchmark that hashes deterministically gave exactly the same value for **every** event gungraun reports (Ir and all 13 cache and derived events). The spread was **0.0000%**, with one distinct value per benchmark and commit. **Same-job base against head:** two identical copies built in different directories gave **0 nonzero differences out of 420 base-vs-head comparisons** and 420 head-rerun comparisons. So with the CS-PF-4 conditions in place, the false-positive rate at 1% was **0%**. The one source of noise was the control case that CS-PF-4 already bans: a `HashMap` with `std`'s `RandomState`. It varied by up to 0.20% across jobs and 0.16% between base and head in the same job, which is below 1% but never zero. **The finding:** adding unrelated code to the benchmark crate (run 3) moved the Ir of an unchanged kernel, `html_tokenize`, by **+0.22% on x86-64 and −0.45% on arm64**. I1 misses moved by up to 22% and estimated cycles by up to 0.63%. The likely cause is a codegen change (inlining or codegen-unit layout) in the monomorphized tokenizer. This is not runner noise: it is deterministic and the same on every host. But a 1% Ir gate will sometimes see a real but unintended shift of this size in a PR that does not touch the kernel. So the gate should use **Ir only**, not cache or estimated-cycle events. The 1% threshold leaves about 2x headroom over the shift seen here. **Budget:** the gate's steps (setup, both builds, both bench runs) took a median of **49–51 s, p90 54–56 s, max 60 s** against 300 s. Building the benchmark crates is the term that will grow with Izcal. Nothing here points to (b) or (c). Hardware counters on a dedicated machine would add noise that Valgrind does not have, and SEC-27 forbids that machine on PRs anyway.

## Headline numbers

| Measure | x86-64 (`ubuntu-latest`) | arm64 (`ubuntu-24.04-arm`) |
|---|---|---|
| Jobs (3 runs × 15) | 45, all succeeded | 45, all succeeded |
| Distinct hosts' CPU models | AMD EPYC 7763 (24), 9V74 (7), 9V45 (9); Intel Xeon 8573C (3), 8370C (2) | Arm part 0xd49 (Neoverse N2), 45 |
| Cross-job Ir spread, 4 kernels + fixed-hash map, within a commit | 0.0000% (1 distinct value each) | 0.0000% (1 distinct value each) |
| Cross-job spread, all 14 events, within a commit | 0.0000% for every event | 0.0000% for every event |
| Same-job base vs head, Ir, deterministic benches | 0 of 195 nonzero (max 0.00000%) | 0 of 195 nonzero |
| Same-job base vs head, Ir, all benches incl. `RandomState` control | 15 of 210 nonzero (all the control), max 0.091%, 0 over 1% | 15 of 210 nonzero (all the control), max 0.161%, 0 over 1% |
| `RandomState` control, cross-job Ir spread | 0.20% (60 distinct values in 60 runs) | 0.18% (60 of 60 distinct) |
| Unchanged kernel moved by an unrelated change in the same crate (run 3) | `html_tokenize` Ir +0.222% | `html_tokenize` Ir −0.451% |
| Gate steps wall time (setup + build base + build head + bench base + bench head + checkout + set-up) | median 51 s, p90 56 s, max 60 s | median 49 s, p90 54 s, max 60 s |
| Whole measurement job (adds two extra bench runs and upload) | median 62 s, p90 69 s, max 72 s | median 61 s, p90 67 s, max 71 s |
| A.5 `perf-gate` budget | 300 s | 300 s |

## Method

- **Benchmark crate** (`measure/cs-d5/bench`, standalone, `Cargo.lock` committed, run with `--locked`). Dependencies are pinned: `cssparser =0.38.0`, `url =2.5.8`, `html5ever =0.40.1`, `gungraun =0.20.0`. The kernels are single-threaded and their inputs come from a fixed-seed LCG, built in gungraun `setup` functions so the setup is not counted:
  - `css_tokenize`: `cssparser` tokenizer over a generated 2000-rule stylesheet (selectors, `@media`, `calc()`, `url()`, comments), descending into every nested block. About 28.7M Ir.
  - `selector_match`: a hand-written right-to-left matcher (tag, id, class bitset; child and descendant combinators) of 200 selectors against a 2000-element tree. About 17.7M Ir.
  - `url_parse`: `Url::join` of 2000 absolute, relative, scheme-relative and IDNA/IPv6/percent-encoded strings against a base URL. About 11.2M Ir.
  - `html_tokenize`: the `html5ever` tokenizer with a counting sink over a generated document of 1000 blocks (entities, comments, `<script>`, implied end tags). About 67M Ir.
  - Run 3 only: `hashmap_random` (5000 string keys in `std::collections::HashMap` with `RandomState`, the control case for CS-PF-4's seeded-hashing rule) and `hashmap_fixed` (the same work with `BuildHasherDefault<DefaultHasher>`). About 5.8M Ir.
- **Toolchain and tools:** Rust 1.99.1 installed by `rustup` in each job, so the runner image's Rust version cannot vary. Valgrind comes from the image's apt (3.22.0 on both architectures). The prebuilt `gungraun-runner` 0.20.0 is checked against SHA-256 values that match GitHub's asset digests. gungraun defaults are kept: ASLR off, environment cleared, cache simulation on with a **fixed** geometry (`--I1=32768,8,64 --D1=32768,8,64 --LL=8388608,16,64`, from `gungraun-runner` `callgrind/args.rs`). So cache events model a fixed machine, not the host.
- **Per job (CS-PF-4's "same job" pattern):** copy the crate to `$RUNNER_TEMP/ws/base` and `$RUNNER_TEMP/ws/head`, two identical sources at different paths. Build both with one shared `CARGO_TARGET_DIR`, so the dependencies build once and the bench crate builds once per copy, as in a gate. Then: `base` runs with `--save-baseline=base`; `head` runs with `--baseline=base` (the gate comparison); `head2` reruns the same head binary against base (pure rerun noise); `cachesim` runs once more (redundant, because cache simulation was already on by default). Each phase's `summary.json` files are kept.
- **Runs:** matrix `os: [ubuntu-latest, ubuntu-24.04-arm] × rep: 1..15`, giving 30 jobs per run and 3 runs: two at 19:47 and 19:49 UTC, one at 20:28 UTC (after a deliberate 35-minute gap). That is 90 jobs, 45 per architecture. Runs 1 and 2 are the same benchmark code (run 2 is an empty commit). Run 3 adds the two hashing benchmarks.
- **Metrics:** the spread is (max−min)/median over jobs, per benchmark and event. The same-job difference is (head−base)/base from gungraun's own `old`/`new` values in `summary.json`. Wall times come from the GitHub jobs API (step `started_at`/`completed_at`), with sub-step stamps from `ci.sh`.

## Run URLs

| Run | Commit | Started (UTC) | Jobs | Result |
|---|---|---|---|---|
| https://github.com/Rorqualx/izcal/actions/runs/39160348068 | 481c0bf | 19:47:16 | 30 | success |
| https://github.com/Rorqualx/izcal/actions/runs/39160530092 | 5d1fd8c (empty commit) | 19:49:41 | 30 | success |
| https://github.com/Rorqualx/izcal/actions/runs/39163421026 | fafda4f (+ hashing benches) | 20:28:00 | 30 | success |

Branch `measure/cs-d5-gungraun` has been deleted. The run pages and artifacts (7-day retention) stay on GitHub until they expire.

## Results

### Cross-job values, within each commit

For every one of the 4 kernels, on each architecture and in each run, every job and every phase (base, head, head2, cachesim) reported the same value for every event. "n" counts benchmark executions (jobs × 4 phases).

| Benchmark | Arch | Runs 1–2 Ir (n=120) | Run 3 Ir (n=60) | Spread within a commit, all 14 events |
|---|---|---|---|---|
| css_tokenize | x86-64 | 28752958 | 28752958 | 0.0000% |
| css_tokenize | arm64 | 28356736 | 28356736 | 0.0000% |
| selector_match | x86-64 | 17729135 | 17729135 | 0.0000% |
| selector_match | arm64 | 16172600 | 16172600 | 0.0000% |
| url_parse | x86-64 | 11181077 | 11181077 | 0.0000% |
| url_parse | arm64 | 10817984 | 10817984 | 0.0000% |
| html_tokenize | x86-64 | 67106292 | **67255292 (+0.222%)** | 0.0000% |
| html_tokenize | arm64 | 65254049 | **64960049 (−0.451%)** | 0.0000% |
| hashmap_fixed | x86-64 | — | 5785920 | 0.0000% |
| hashmap_fixed | arm64 | — | 4945028 | 0.0000% |
| hashmap_random | x86-64 | — | 5785176 … 5793789, 60 distinct values in 60 runs | Ir 0.20%; D1mw 2.2%, RamHits 1.4% |
| hashmap_random | arm64 | — | 5028686 … 5036642, 60 of 60 distinct | Ir 0.18%; D1mw 2.2%, DLmw 1.3% |

The five x86-64 CPU models (AMD Zen 3, Zen 4, Zen 5; Intel Ice Lake, Emerald Rapids) gave the same counts, including the cache events. Two things make this hold, and the gate depends on both: Callgrind's cache model has a fixed geometry, and Valgrind's virtual CPU reports the same feature set whatever the host. glibc's `memcpy` and `strlen` therefore pick the same implementation (ifunc) on every host.

Between commits, with the kernel source unchanged (run 3 added two functions to `src/lib.rs` and a second benchmark group), the other events moved as follows:

| Benchmark | Arch | Ir | I1mr | LLhits | EstimatedCycles |
|---|---|---|---|---|---|
| html_tokenize | x86-64 | +0.222% | −21.8% | −19.0% | +0.282% |
| html_tokenize | arm64 | −0.451% | −6.0% | −5.1% | −0.626% |
| url_parse | x86-64 | 0 | +11.3% | +10.3% | +0.276% |
| url_parse | arm64 | 0 | +7.6% | +6.8% | +0.158% |
| css_tokenize | both | 0 | 0 to +1.8% | ≤0.02% | ≤0.0002% |
| selector_match | both | 0 | 0 | 0 | 0 |

Cause (likely, not proven): `html_tokenize` instantiates `html5ever::Tokenizer<Counter>` inside the edited crate, so the generic tokenizer code is compiled as part of `cs-d5-bench`, and adding items there can change how the crate is split into codegen units and what gets inlined. Supporting evidence on macOS/aarch64 (not the CI target): the bench binaries of 481c0bf and fafda4f differ in their html5ever-related symbols. Two monomorphized helpers that are separate symbols in the old build, `drop_glue::<Tendril<UTF8>>` and a `smallsort` instantiation over `(html5ever::tokenizer::states::State, u64)`, are absent in the new build, which means they were inlined or merged. `url_parse` keeps the same Ir but its I-cache events move, which is consistent with a code-layout change.

### Same-job base vs head (false-positive rate at 1%)

| Comparison | Arch | Comparisons | Nonzero | Max abs diff | Over 1% |
|---|---|---|---|---|---|
| head vs base, deterministic benches (Ir) | x86-64 | 195 | 0 | 0.00000% | 0 |
| head vs base, deterministic benches (Ir) | arm64 | 195 | 0 | 0.00000% | 0 |
| head vs base, `hashmap_random` (Ir) | x86-64 | 15 | 15 | 0.091% | 0 |
| head vs base, `hashmap_random` (Ir) | arm64 | 15 | 15 | 0.161% | 0 |
| head2 (rerun) vs base, deterministic benches (Ir) | both | 390 | 0 | 0.00000% | 0 |
| head2 vs base, `hashmap_random` (Ir) | x86-64 / arm64 | 15 / 15 | 15 / 15 | 0.103% / 0.159% | 0 |
| head vs base, EstimatedCycles, deterministic benches | both | 390 | 0 | 0.00000% | 0 |

Different source paths for base and head (`ws/base` vs `ws/head`, the same length) did not change any count.

### Wall time against the A.5 budget (300 s)

From the GitHub jobs API, 45 jobs per architecture, in seconds:

| Step | x86-64 median / p90 / max | arm64 median / p90 / max |
|---|---|---|
| Set up job + checkout | 2 / 4 / 5 | 2 / 2 / 4 |
| Setup total | 20 / 26 / 33 | 22 / 26 / 32 |
|  – `apt-get install valgrind` (incl. `apt-get update`) | 11.8 / 16.1 / 24.9 | 16.2 / 20.3 / 21.6 |
|  – `rustup toolchain install 1.99.1` | 7.9 / 8.8 / 10.2 | 5.3 / 5.8 / 10.8 |
|  – `gungraun-runner` download + verify | 0.3 / 0.5 / 0.7 | 0.3 / 0.4 / 0.8 |
| Build base (cold, all dependencies, release + debuginfo=1) | 20 / 21 / 22 | 15 / 17 / 17 |
| Build head (bench crate only) | 2 / 3 / 3 | 2 / 3 / 3 |
| Bench base (4–6 benchmarks) | 4 / 5 / 5 | 4 / 5 / 5 |
| Bench head | 3 / 5 / 5 | 4 / 5 / 5 |
| **Gate steps total** | **51 / 56 / 60** | **49 / 54 / 60** |
| Extra steps (head2, cachesim, upload) | ~9 | ~9 |
| Whole job | 62 / 69 / 72 | 61 / 67 / 71 |

Per-benchmark time under Callgrind (`duration_ns` in `summary.json`, head phase, median): `html_tokenize` 1.5–1.65 s, `css_tokenize` 0.8–0.85 s, `selector_match` and `url_parse` 0.55–0.6 s, hash maps 0.45 s. A linear fit gives about 17 ns of wall time per counted instruction plus about 0.35 s of fixed cost per benchmark. The gate runs each benchmark twice, once for base and once for head.

Queue wait, from the job's `created_at` to `started_at`, had a median of 2–5 s and a p90 of 63–66 s. The high p90 comes from 30 jobs being queued at once against the account's concurrency limit. A single per-PR gate job does not hit that limit.

### Failures

There were none: 90 of 90 jobs succeeded and every step passed. Two local issues came up, neither of them a gungraun problem. The pre-commit AI review blocked the first commit with two false "won't compile" findings based on the cssparser 0.37 API. The crate compiles against the pinned 0.38.0, and the commit records `Hooks-Skipped: review` with that reason. The first `git push` ran after that failed commit, so it pushed a branch identical to `main` with no workflow on it. No run was triggered.

## Caveats

- **Kernel size.** These kernels are 5–67M Ir, and their dependency builds are small (cold build 15–20 s). Izcal's benchmarked crates (style, layout, display list, IPC codecs) will have larger dependency graphs. Building them twice, as base and head, is the term that will dominate the 300 s budget. This run measured neither a warm `target/` cache nor a large workspace. Budget headroom: about 245 s at p90, enough for roughly 60–100 kernels of this size at two runs each. The actual numbers depend on the build.
- **Codegen sensitivity is real and does not come from the runner.** A 0.2–0.45% Ir move in an unchanged kernel came from an unrelated edit in the same crate. Larger moves are possible for kernels dominated by generic code instantiated in the edited crate. Gating on cache events or `EstimatedCycles` would fail PRs often: I1mr moved 22% and LLhits 19% with no kernel change. Gate on Ir only.
- **The hashing rule is load-bearing.** `RandomState` made every run different, by up to 0.20% across jobs and 0.16% within a job. That is under 1% for this kernel, but the size of the effect depends on the workload; a probe-heavy map in a cascade could exceed it. CS-PF-4's simulator-key requirement and gungraun's ASLR-off default are both needed. ASLR was not tested with ASLR on.
- **Image and Valgrind version.** All 90 jobs ran image `ubuntu24 20261012.334.1` / `ubuntu24-arm64 20261012.149.1`, Valgrind 3.22.0 and kernel 6.17.0-1022-azure. A runner image update that changes Valgrind, glibc or the `ubuntu-latest` alias (for example to 26.04) can change absolute counts. Measuring base and head in the same job makes that harmless for the gate, but not for any stored long-term baseline. The same goes for the Rust toolchain, which was pinned here.
- **One Arm CPU model.** All arm64 jobs ran on the same part, 0xd49, so cross-model behaviour on Arm was not tested. x86-64 covered 5 models from two vendors.
- **Sample size and time span.** 3 runs over 41 minutes on one day. All 90 jobs ran on a single image version, so this measures host variation, not image-rollout variation.
- **Detection was not exercised.** No PR with a real regression was injected. Ir is an exact count, so a 1% code change shows up as 1% by construction. The real limit is the codegen shift described above.
- **Not a full CS-PF-4 implementation.** The run had no `Perf:` override, no path filter and no V8-free crate check. Kernels are stand-ins: `selectors` was replaced by a hand-written matcher.

## Workflow file (`.github/workflows/measure-cs-d5.yml`)

It ran on the branch only, followed `checks.yml`'s conventions (SHA-pinned actions, `permissions: contents: read`, `persist-credentials: false`, no cache), and passed `actionlint` and `zizmor --min-severity medium`, both locally and in the pre-commit and pre-push hooks.

```yaml
# Temporary measurement for coding-standard decision CS-D5: are gungraun
# (Valgrind) instruction counts stable enough on GitHub-hosted runners for a
# 1% per-PR gate, and does the job fit its 300 s A.5 budget? Runs only on
# pushes to the measurement branch; the branch is deleted afterwards.

name: measure-cs-d5

on:
  push:
    branches: [measure/cs-d5-gungraun]

permissions:
  contents: read

jobs:
  bench:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, ubuntu-24.04-arm]
        rep: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    steps:
      # Pinned to full commit hashes, as in checks.yml.
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0
        with:
          persist-credentials: false

      - name: Setup (valgrind, pinned toolchain, gungraun-runner)
        run: bash measure/cs-d5/ci.sh setup

      - name: Build base
        run: bash measure/cs-d5/ci.sh build base

      - name: Build head
        run: bash measure/cs-d5/ci.sh build head

      - name: Bench base (save baseline)
        run: bash measure/cs-d5/ci.sh bench base base --save-baseline=base

      - name: Bench head (compare with base)
        run: bash measure/cs-d5/ci.sh bench head head --baseline=base

      - name: Bench head again (same binary)
        run: bash measure/cs-d5/ci.sh bench head2 head --baseline=base

      - name: Bench head with cache simulation
        run: bash measure/cs-d5/ci.sh bench cachesim head --callgrind-args=--cache-sim=yes

      - name: Upload results
        if: always()
        uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          name: results-${{ matrix.os }}-${{ matrix.rep }}
          path: ${{ runner.temp }}/results
          retention-days: 7
```

## Step script (`measure/cs-d5/ci.sh`)

```bash
#!/usr/bin/env bash
# CS-D5 measurement steps, one subcommand per workflow step so GitHub's step
# timings give the setup, build and run costs separately.
#
#   ci.sh setup                 valgrind, pinned toolchain, gungraun-runner
#   ci.sh build <base|head>     copy the crate to $WS/<copy>, cargo bench --no-run
#   ci.sh bench <phase> <copy> [gungraun args...]
#                               run the benchmarks, keep summary.json files
set -euo pipefail

TOOLCHAIN=1.99.1
RUNNER_VERSION=0.20.0
SRC=$(cd "$(dirname "${BASH_SOURCE[0]}")/bench" && pwd)
WS=${RUNNER_TEMP:?}/ws
OUT=${RUNNER_TEMP:?}/results
# One target directory for both copies: the dependencies build once, the
# benchmark crate builds once per copy (different paths), as in a real gate.
export CARGO_TARGET_DIR=$WS/target
mkdir -p "$WS" "$OUT"

stamp() { echo "$1 $(date +%s.%N)" >> "$OUT/stamps.txt"; }

case "$1" in
  setup)
    stamp setup-start
    sudo apt-get update -qq
    sudo apt-get install -y -qq valgrind > /dev/null
    stamp valgrind-done
    rustup toolchain install "$TOOLCHAIN" --profile minimal --no-self-update
    rustup default "$TOOLCHAIN"
    stamp toolchain-done
    case "$(uname -m)" in
      x86_64) triple=x86_64-unknown-linux-gnu; sha=b3b39d4243e43597512a1f79016a1f38cbf90e5869c0c2535fe4097c69ed91aa ;;
      aarch64) triple=aarch64-unknown-linux-gnu; sha=30ddf6737e486d1d3c11746eb603d8a228afeb1614dcf84ebe9ee1442f59d2f3 ;;
      *) echo "unsupported arch $(uname -m)" >&2; exit 1 ;;
    esac
    name=gungraun-runner-v$RUNNER_VERSION-$triple
    tmp=$(mktemp -d)
    curl -fsSL -o "$tmp/r.tar.gz" "https://github.com/gungraun/gungraun/releases/download/v$RUNNER_VERSION/$name.tar.gz"
    echo "$sha  $tmp/r.tar.gz" | sha256sum -c --quiet -
    tar -xzf "$tmp/r.tar.gz" -C "$tmp" "$name/gungraun-runner"
    mkdir -p "$RUNNER_TEMP/bin"
    install -m 0755 "$tmp/$name/gungraun-runner" "$RUNNER_TEMP/bin/gungraun-runner"
    echo "$RUNNER_TEMP/bin" >> "${GITHUB_PATH:?}"
    stamp setup-done
    {
      echo "image: ${ImageOS:-?} ${ImageVersion:-?}"
      echo "runner: ${RUNNER_NAME:-?} ${RUNNER_ARCH:-?}"
      echo "kernel: $(uname -r)"
      echo "cpu: $(grep -m1 -E 'model name|CPU part' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//')"
      lscpu | grep -E '^(Model name|Vendor ID|CPU\(s\)|Flags|L1d|L2|L3)' | sed 's/^/lscpu: /' | cut -c1-400
      echo "nproc: $(nproc)"
      echo "valgrind: $(valgrind --version)"
      echo "rustc: $(rustc --version)"
      echo "glibc: $(ldd --version | head -1)"
    } > "$OUT/meta.txt"
    cat "$OUT/meta.txt"
    ;;
  build)
    copy=$2
    rm -rf "${WS:?}/$copy"
    mkdir -p "$WS/$copy"
    cp -R "$SRC/." "$WS/$copy/"
    stamp "build-$copy-start"
    (cd "$WS/$copy" && cargo bench --locked --no-run)
    stamp "build-$copy-done"
    ;;
  bench)
    phase=$2 copy=$3
    shift 3
    stamp "bench-$phase-start"
    (cd "$WS/$copy" && cargo bench --locked --bench kernels -- --save-summary=pretty-json "$@") 2>&1 | tee "$OUT/$phase.log"
    stamp "bench-$phase-done"
    mkdir -p "$OUT/$phase"
    (cd "$CARGO_TARGET_DIR/gungraun" && find . -name summary.json -exec cp --parents {} "$OUT/$phase/" \;)
    ;;
  *) echo "usage: ci.sh setup | build COPY | bench PHASE COPY [ARGS]" >&2; exit 2 ;;
esac
```

## Benchmark crate

`Cargo.toml`:

```toml
[package]
name = "cs-d5-bench"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
cssparser = "=0.38.0"
url = "=2.5.8"
html5ever = "=0.40.1"

[dev-dependencies]
gungraun = "=0.20.0"

[[bench]]
name = "kernels"
harness = false

[profile.bench]
debug = 1
```

`benches/kernels.rs` (run 3 version; runs 1–2 had only the `kernels` group):

```rust
//! gungraun benchmarks for the CS-D5 measurement. Inputs are built in
//! `setup` functions, so only the kernel is counted.

use std::hint::black_box;

use cs_d5_bench as k;
use gungraun::prelude::*;

#[library_benchmark]
#[bench::stylesheet(setup = k::make_css, args = (2000))]
fn css_tokenize(css: String) -> u64 {
    black_box(k::css_tokenize(black_box(&css)))
}

#[library_benchmark]
#[bench::dom2000_sel200(setup = k::make_match_input)]
fn selector_match(input: (Vec<k::Elem>, Vec<k::Selector>)) -> u64 {
    black_box(k::selector_match(black_box(&input.0), black_box(&input.1)))
}

#[library_benchmark]
#[bench::urls(setup = k::make_urls, args = (2000))]
fn url_parse(urls: Vec<String>) -> u64 {
    black_box(k::url_parse(black_box(&urls)))
}

#[library_benchmark]
#[bench::document(setup = k::make_html, args = (1000))]
fn html_tokenize(html: String) -> u64 {
    black_box(k::html_tokenize(black_box(&html)))
}

#[library_benchmark]
#[bench::keys5000(setup = k::make_keys, args = (5000))]
fn hashmap_random(keys: Vec<String>) -> u64 {
    black_box(k::hashmap_random(black_box(&keys)))
}

#[library_benchmark]
#[bench::keys5000(setup = k::make_keys, args = (5000))]
fn hashmap_fixed(keys: Vec<String>) -> u64 {
    black_box(k::hashmap_fixed(black_box(&keys)))
}

library_benchmark_group!(name = hashing, benchmarks = [hashmap_random, hashmap_fixed]);

library_benchmark_group!(
    name = kernels,
    benchmarks = [css_tokenize, selector_match, url_parse, html_tokenize]
);

main!(library_benchmark_groups = kernels, hashing);
```

`src/lib.rs` (run 3 version; runs 1–2 lacked the hashing functions at the end):

```rust
//! CS-D5 measurement: CPU-bound kernels like the ones the Izcal perf gate
//! would cover (CSS tokenizing, selector matching, URL parsing, HTML
//! tokenizing). Inputs are generated from a fixed-seed LCG, so every run sees
//! the same bytes. No `HashMap` with a random seed, no threads.

use std::cell::Cell;

/// Fixed-seed linear congruential generator (deterministic inputs).
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const TAGS: [&str; 10] = [
    "div", "span", "p", "a", "li", "ul", "section", "img", "button", "td",
];
const CLASSES: [&str; 12] = [
    "nav", "item", "active", "btn", "primary", "card", "header", "footer", "row", "col", "hidden",
    "title",
];

// ---------------------------------------------------------------- CSS input

pub fn make_css(rules: usize) -> String {
    let mut r = Lcg::new(0x5eed_c55);
    let mut s = String::new();
    for i in 0..rules {
        let tag = TAGS[r.below(10) as usize];
        let c1 = CLASSES[r.below(12) as usize];
        let c2 = CLASSES[r.below(12) as usize];
        match i % 4 {
            0 => s.push_str(&format!("{tag}.{c1} > .{c2}:hover, #id{i} {{")),
            1 => s.push_str(&format!(
                "@media (min-width: {}px) {{ .{c1} {tag} {{",
                300 + r.below(900)
            )),
            2 => s.push_str(&format!("{tag}[data-x=\"{c2}\"] ~ .{c1}::before {{")),
            _ => s.push_str(&format!(".{c1}.{c2} {tag}:nth-child(2n+{}) {{", r.below(5))),
        }
        s.push_str(&format!(
            " color: rgb({}, {}, {}); margin: {}px {}em; width: calc(100% - {}px);",
            r.below(256),
            r.below(256),
            r.below(256),
            r.below(40),
            r.below(4),
            r.below(200)
        ));
        s.push_str(" background: url(\"img/bg.png\") no-repeat; /* comment */ font: 12px/1.5 \"Helvetica Neue\", sans-serif; }");
        if i % 4 == 1 {
            s.push_str(" }");
        }
        s.push('\n');
    }
    s
}

fn css_walk(p: &mut cssparser::Parser<'_>, depth: u32) -> u64 {
    let mut n = 0u64;
    loop {
        let block = match p.next_including_whitespace_and_comments() {
            Err(_) => break,
            Ok(tok) => {
                n += 1;
                matches!(
                    tok,
                    cssparser::Token::Function(_)
                        | cssparser::Token::ParenthesisBlock
                        | cssparser::Token::SquareBracketBlock
                        | cssparser::Token::CurlyBracketBlock
                )
            }
        };
        if block {
            let r: Result<u64, cssparser::ParseError<()>> =
                p.parse_nested_block(|inner| Ok(css_walk(inner, depth + 1)));
            n += r.unwrap_or(0);
        }
    }
    n
}

/// Tokenizes a stylesheet with `cssparser`, descending into every block.
pub fn css_tokenize(css: &str) -> u64 {
    let mut p = cssparser::Parser::new(css);
    css_walk(&mut p, 0)
}

// ---------------------------------------------------------- selector matching

pub struct Elem {
    pub parent: Option<u32>,
    pub tag: u8,
    pub id: u32,
    pub classes: u16, // bitset over CLASSES
}

pub struct Compound {
    pub tag: Option<u8>,
    pub id: Option<u32>,
    pub classes: u16,
}

pub enum Comb {
    Descendant,
    Child,
}

/// Rightmost compound first, then (combinator, compound) pairs leftwards.
pub struct Selector {
    pub key: Compound,
    pub rest: Vec<(Comb, Compound)>,
}

pub fn make_dom(n: usize) -> Vec<Elem> {
    let mut r = Lcg::new(0xd0d0);
    let mut v: Vec<Elem> = Vec::with_capacity(n);
    for i in 0..n {
        let parent = if i == 0 {
            None
        } else {
            Some(r.below(i as u64) as u32)
        };
        let mut classes = 0u16;
        for _ in 0..r.below(4) {
            classes |= 1 << r.below(12);
        }
        v.push(Elem {
            parent,
            tag: r.below(10) as u8,
            id: i as u32,
            classes,
        });
    }
    v
}

fn rand_compound(r: &mut Lcg) -> Compound {
    let tag = if r.below(2) == 0 {
        Some(r.below(10) as u8)
    } else {
        None
    };
    let mut classes = 0u16;
    for _ in 0..r.below(3) {
        classes |= 1 << r.below(12);
    }
    let id = if r.below(20) == 0 {
        Some(r.below(2000) as u32)
    } else {
        None
    };
    Compound { tag, id, classes }
}

pub fn make_selectors(n: usize) -> Vec<Selector> {
    let mut r = Lcg::new(0x5e1e);
    (0..n)
        .map(|_| {
            let key = rand_compound(&mut r);
            let rest = (0..r.below(4))
                .map(|_| {
                    let c = if r.below(3) == 0 {
                        Comb::Child
                    } else {
                        Comb::Descendant
                    };
                    (c, rand_compound(&mut r))
                })
                .collect();
            Selector { key, rest }
        })
        .collect()
}

fn compound_matches(c: &Compound, e: &Elem) -> bool {
    c.tag.is_none_or(|t| t == e.tag)
        && c.id.is_none_or(|i| i == e.id)
        && (e.classes & c.classes) == c.classes
}

fn rest_matches(dom: &[Elem], rest: &[(Comb, Compound)], mut cur: u32) -> bool {
    let Some(((comb, comp), tail)) = rest.split_first() else {
        return true;
    };
    match comb {
        Comb::Child => match dom[cur as usize].parent {
            Some(p) if compound_matches(comp, &dom[p as usize]) => rest_matches(dom, tail, p),
            _ => false,
        },
        Comb::Descendant => {
            while let Some(p) = dom[cur as usize].parent {
                if compound_matches(comp, &dom[p as usize]) && rest_matches(dom, tail, p) {
                    return true;
                }
                cur = p;
            }
            false
        }
    }
}

/// Right-to-left matching of every selector against every element.
pub fn selector_match(dom: &[Elem], sels: &[Selector]) -> u64 {
    let mut n = 0u64;
    for (i, e) in dom.iter().enumerate() {
        for s in sels {
            if compound_matches(&s.key, e) && rest_matches(dom, &s.rest, i as u32) {
                n += 1;
            }
        }
    }
    n
}

// ------------------------------------------------------------------- URLs

pub fn make_urls(n: usize) -> Vec<String> {
    let mut r = Lcg::new(0x0e1);
    let hosts = [
        "example.com",
        "xn--bcher-kva.example",
        "sub.domain.test",
        "127.0.0.1",
        "[::1]",
    ];
    (0..n)
        .map(|i| match i % 4 {
            0 => format!(
                "https://{}/path/{}/../to/./file{}.html?q={}&x=%20y#frag",
                hosts[r.below(5) as usize],
                r.below(1000),
                i,
                r.below(99999)
            ),
            1 => format!("../rel/{}/img.png?v={}", r.below(100), i),
            2 => format!(
                "HTTP://User:Pass@{}:8080/A%2fB/c d/ü?ä=ö",
                hosts[r.below(5) as usize]
            ),
            _ => format!(
                "//cdn.{}/static/{}.js",
                hosts[r.below(3) as usize],
                r.below(5000)
            ),
        })
        .collect()
}

/// Parses each string against a base URL (absolute, relative, scheme-relative).
pub fn url_parse(urls: &[String]) -> u64 {
    let base = url::Url::parse("https://www.example.org/dir/sub/page.html").unwrap();
    let mut n = 0u64;
    for u in urls {
        if let Ok(p) = base.join(u) {
            n += p.as_str().len() as u64;
        }
    }
    n
}

// ------------------------------------------------------------------- HTML

pub fn make_html(blocks: usize) -> String {
    let mut r = Lcg::new(0x47e1);
    let mut s = String::from("<!DOCTYPE html><html><head><title>t</title></head><body>\n");
    for i in 0..blocks {
        let c = CLASSES[r.below(12) as usize];
        s.push_str(&format!(
            "<div class=\"{c} row\" id=\"b{i}\" data-n='{}'><p>Some text &amp; an entity &lt;{}&gt; \
             with <a href=\"/x/{}?a=1&b=2\">a link</a> and <img src=x.png alt=\"\" /> &nbsp;</p>\
             <!-- c{i} --><ul><li>one<li>two</ul><script>var x = \"</div>\" < {};</script></div>\n",
            r.below(100),
            r.below(1000),
            r.below(1000),
            r.below(10)
        ));
    }
    s.push_str("</body></html>");
    s
}

struct Counter {
    n: Cell<u64>,
}

impl html5ever::tokenizer::TokenSink for Counter {
    type Handle = ();
    fn process_token(
        &self,
        token: html5ever::tokenizer::Token,
        _line: u64,
    ) -> html5ever::tokenizer::TokenSinkResult<()> {
        use html5ever::tokenizer::Token;
        let add = match token {
            Token::TagToken(t) => 1 + t.attrs.len() as u64,
            Token::CharacterTokens(b) => b.len() as u64,
            _ => 1,
        };
        self.n.set(self.n.get() + add);
        html5ever::tokenizer::TokenSinkResult::Continue
    }
}

/// Tokenizes an HTML document with the `html5ever` tokenizer.
pub fn html_tokenize(html: &str) -> u64 {
    use html5ever::tendril::StrTendril;
    use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts};
    let input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(html));
    let tok = Tokenizer::new(Counter { n: Cell::new(0) }, TokenizerOpts::default());
    let _ = tok.feed(&input);
    tok.end();
    tok.sink.n.get()
}

/// Setup for the selector benchmark: 2000 elements, 200 selectors.
pub fn make_match_input() -> (Vec<Elem>, Vec<Selector>) {
    (make_dom(2000), make_selectors(200))
}

#[cfg(test)]
mod tests {
    #[test]
    fn kernels_do_work() {
        let (d, s) = super::make_match_input();
        let r = [
            super::css_tokenize(&super::make_css(2000)),
            super::selector_match(&d, &s),
            super::url_parse(&super::make_urls(2000)),
            super::html_tokenize(&super::make_html(1000)),
        ];
        println!("{r:?}");
        assert!(r.iter().all(|&n| n > 0));
    }
}

/// Inserts and looks up `n` string keys in a `HashMap` with `std`'s
/// randomly seeded `RandomState`: the control case for CS-PF-4's rule that
/// benchmarked hashing uses a fixed key.
pub fn hashmap_random(keys: &[String]) -> u64 {
    let mut m = std::collections::HashMap::new();
    hash_work(&mut m, keys)
}

/// The same work with a fixed-key hasher.
pub fn hashmap_fixed(keys: &[String]) -> u64 {
    let mut m: std::collections::HashMap<
        &str,
        u64,
        std::hash::BuildHasherDefault<std::hash::DefaultHasher>,
    > = std::collections::HashMap::default();
    hash_work(&mut m, keys)
}

fn hash_work<'a, S: std::hash::BuildHasher>(
    m: &mut std::collections::HashMap<&'a str, u64, S>,
    keys: &'a [String],
) -> u64 {
    for (i, k) in keys.iter().enumerate() {
        m.insert(k.as_str(), i as u64);
    }
    keys.iter().filter_map(|k| m.get(k.as_str())).sum()
}

pub fn make_keys(n: usize) -> Vec<String> {
    let mut r = Lcg::new(0x4a54);
    (0..n).map(|i| format!("key-{}-{}", i, r.next())).collect()
}
```

## Appendix: raw Ir per job (head phase)

| run | os | rep | cpu | css_tokenize | hashmap_fixed | hashmap_random | html_tokenize | selector_match | url_parse |
|---|---|---|---|---|---|---|---|---|---|
| 39160348068 | ubuntu-24.04-arm | 1 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 2 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 3 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 4 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 5 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 6 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 7 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 8 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 9 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 10 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 11 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 12 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 13 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 14 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-24.04-arm | 15 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160348068 | ubuntu-latest | 1 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 2 | INTEL(R) XEON(R) PLATINUM 8573C | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 3 | INTEL(R) XEON(R) PLATINUM 8573C | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 4 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 5 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 6 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 7 | AMD EPYC 9V45 96-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 8 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 9 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 10 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 11 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 12 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 13 | AMD EPYC 9V45 96-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 14 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160348068 | ubuntu-latest | 15 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-24.04-arm | 1 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 2 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 3 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 4 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 5 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 6 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 7 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 8 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 9 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 10 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 11 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 12 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 13 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 14 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-24.04-arm | 15 | 0xd49 | 28356736 | - | - | 65254049 | 16172600 | 10817984 |
| 39160530092 | ubuntu-latest | 1 | AMD EPYC 9V45 96-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 2 | Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 3 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 4 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 5 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 6 | AMD EPYC 9V45 96-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 7 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 8 | INTEL(R) XEON(R) PLATINUM 8573C | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 9 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 10 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 11 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 12 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 13 | AMD EPYC 7763 64-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 14 | AMD EPYC 9V45 96-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39160530092 | ubuntu-latest | 15 | AMD EPYC 9V74 80-Core Processor | 28752958 | - | - | 67106292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-24.04-arm | 1 | 0xd49 | 28356736 | 4945028 | 5034552 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 2 | 0xd49 | 28356736 | 4945028 | 5030389 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 3 | 0xd49 | 28356736 | 4945028 | 5036642 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 4 | 0xd49 | 28356736 | 4945028 | 5028889 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 5 | 0xd49 | 28356736 | 4945028 | 5034937 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 6 | 0xd49 | 28356736 | 4945028 | 5031420 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 7 | 0xd49 | 28356736 | 4945028 | 5034377 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 8 | 0xd49 | 28356736 | 4945028 | 5035635 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 9 | 0xd49 | 28356736 | 4945028 | 5033030 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 10 | 0xd49 | 28356736 | 4945028 | 5033175 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 11 | 0xd49 | 28356736 | 4945028 | 5028686 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 12 | 0xd49 | 28356736 | 4945028 | 5033486 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 13 | 0xd49 | 28356736 | 4945028 | 5030919 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 14 | 0xd49 | 28356736 | 4945028 | 5031226 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-24.04-arm | 15 | 0xd49 | 28356736 | 4945028 | 5034015 | 64960049 | 16172600 | 10817984 |
| 39163421026 | ubuntu-latest | 1 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5788221 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 2 | Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz | 28752958 | 5785920 | 5785176 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 3 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5788514 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 4 | AMD EPYC 9V45 96-Core Processor | 28752958 | 5785920 | 5793789 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 5 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5787753 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 6 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5791512 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 7 | AMD EPYC 9V45 96-Core Processor | 28752958 | 5785920 | 5788654 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 8 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5786426 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 9 | AMD EPYC 9V45 96-Core Processor | 28752958 | 5785920 | 5788633 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 10 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5790717 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 11 | AMD EPYC 9V45 96-Core Processor | 28752958 | 5785920 | 5789561 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 12 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5787863 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 13 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5789355 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 14 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5788302 | 67255292 | 17729135 | 11181077 |
| 39163421026 | ubuntu-latest | 15 | AMD EPYC 7763 64-Core Processor | 28752958 | 5785920 | 5789499 | 67255292 | 17729135 | 11181077 |


Raw artifacts (summary.json per benchmark per phase, logs, host metadata, stamps) and the GitHub jobs API dumps (25 MB) are kept outside the repository on revire at `~/izcal-measure/cs-d5/cs-d5-raw.tgz` (sha256 `cfdea9d51a7cb5bfc8a9782c6356df766601f2215d10e036804725fda5d3301c`; it unpacks to `data/` and `jobs/`). The analysis script is `analyze_cs_d5.py` in this folder and its full output is `analysis.txt`; the benchmark crate and workflow are in `source/`.
