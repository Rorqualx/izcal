# CS-D2 proxy: cost of integer overflow checks in release builds of browser-renderer Rust crates

Date: 2026-10-18. Session: browser-61. Rig: revire (Huey the Revire). Decision under test: CS-D2 in
`reports/Izcal coding standard.md` §9.3. Rule (a) keep overflow checks on in tier R release builds if the
cost is ≤ 1% on Speedometer 3 and P-1 at M4, else (b) off, but on in a `checked` profile.
**This file measures a proxy now (existing crates on a renderer's hot path), not the M4 measurement.**

## Conclusion

The proxy is **over the 1% threshold** and supports **(b)**. With overflow checks on, the
user-space instruction count goes up by a geometric mean of **+3.87% across the five crates** (+5.32% across all 17
benchmarks; +4.50% for the four renderer crates without `url`). Wall time goes up by **+3.09%** (crate geomean).
The crates' own benchmarks agree: html5ever's criterion bench is **+8.26%** and url's bench is **+3.43%**.
The cost is uneven. Tight byte-level loops (the cssparser tokenizer, the html5ever tokenizer and tree builder) pay
+5–10%. Stylo stylesheet parsing and Stylist (cascade-data) building execute +2–6% more instructions but show
almost no change in cycles or wall time (+0.3% wall geomean), so on this CPU the extra instructions mostly fit
into otherwise idle issue slots. Selector matching shows no cost at all (−0.46% instructions, +0.18% wall), a
codegen effect. No crate-level aggregate falls under 1% on instructions except selectors, and the overall
aggregates are 3–5× the threshold. Confidence: **high** that checks cost more than 1% on these hot paths (the
instruction counts are deterministic to <0.3%, the wall time agrees, and the crates' own benches agree).
Confidence is **moderate** that the M4 measurement will also exceed 1%. Speedometer 3 and P-1 time also includes JS
execution, layout, painting, GPU work and idle time, which dilutes a renderer-parser cost. Izcal's own code and a
newer CPU may also differ. Even so, a 3–8% cost in the HTML and CSS parsers would need about 4–8× dilution to
fall under 1% end to end, so the M4 run should still be done as the rule says. The proxy gives no reason to expect (a).

## Method

* **Crates and why.**
  * `stylo` (servo/stylo, `style` + `selectors`): Izcal's chosen CSS engine. It has no benchmarks in the repo (its
    tests and benches live in Servo), so I wrote small realistic workloads: `Stylesheet::from_str` on real
    stylesheets, then building a `Stylist` (`append_stylesheet` + `flush`, which builds the cascade data,
    selector maps and invalidation maps).
  * `selectors`: matching. I parsed every non-pseudo-element selector out of Bootstrap with stylo's servo
    `SelectorImpl` (2,837 selectors) and matched each one against every element (7,347) of a real Bootstrap docs
    page with `selectors::matching::matches_selector`. The page was parsed with html5ever into an arena DOM that
    implements `selectors::Element`. This is naive all-pairs matching: no rule hash and no Bloom filter.
  * `cssparser`: a full token walk of the stylesheets, recursing into every block. Its own `#[bench]`s need
    nightly `test` behind a `bench` feature and cover only 2 micro-cases, so I did not use them.
  * `html5ever`: its own criterion bench (the tokenizer on `data/bench/*.html`), plus the same six inputs in the
    proxy driver with the same 1024-char chunking, plus full tree building (`parse_document` → `RcDom`) of two
    real pages.
  * `url` (optional): its own `bencher` bench, plus resolving all 1,317 `href`/`src` values of the Wikipedia page
    against the page URL, plus the own-bench strings in a fixed loop.
* **One driver crate** (`csd2-proxy`, source below) depends on all five crates by path at the recorded commits.
  `[patch.crates-io]` makes stylo's transitive `cssparser`, `url`, `idna`, `web_atoms`, `markup5ever`,
  `html5ever`, `selectors` and `servo_arc` also resolve to the cloned checkouts (checked with `cargo tree`).
  The same `Cargo.lock` was used for both builds.
* **Two builds** differ only in `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`. Both use the default `release`
  profile (opt-level 3, no LTO, 16 CGUs), the same toolchain and commits, and separate target dirs. The
  crates' own benches were built with `cargo bench --no-run` as well, with
  `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true CARGO_PROFILE_BENCH_OVERFLOW_CHECKS=true` for the "on" build.
  * Proof that the checks are compiled in: `jo` instructions 57 → 245, `jno` 46 → 91, and the binary grows
    7,373,904 → 7,448,144 bytes (+1.0%). The `core::panicking::panic_const::panic_const_{add,sub,mul,neg,shl,shr}_overflow`
    symbols are present only in the "on" binary.
* **Primary metric: instructions:u** via `sudo perf stat -x, -e instructions:u,cycles:u`. Sudo was needed
  because revire has `perf_event_paranoid=4`; the sysctl was not changed. Valgrind is not installed, so
  gungraun/callgrind was not used. For each benchmark and build:
  `(count(N iterations) − count(0 iterations)) / N`, which subtracts process start, input loading and setup.
  3 reps per build, interleaved off/on then on/off, pinned with `taskset -c 4`. Rep-to-rep spread is ≤0.0011%
  for the large workloads and ≤0.26% for the sub-µs tokenizer cases.
* **Secondary: wall time.** 3 interleaved processes per build × 15 samples = **45 samples per build**. Each
  sample runs the workload `batch` times (≥ ~20 ms per sample) after a warm-up; reported as median ns/call and IQR.
  The crates' own benches ran in 3 interleaved rounds (off,on / on,off / off,on); criterion used 2 s warm-up and 5 s
  measurement, pinned to CPU 4.
* The CPU frequency governor was `schedutil` with turbo on (left as found). This adds noise to wall time only.

## Versions

| item | value |
|---|---|
| rig | revire, Intel Xeon E5-2695 v2 @ 2.40GHz (Ivy Bridge-EP, 12c/24t), Linux 7.0.0-38-generic, perf 7.0.14 |
| rustc | 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, host x86_64-unknown-linux-gnu (rustup, minimal profile, in `~/izcal-measure/cs-d2/rustup`) |
| cargo | 1.99.1 (5f94df478 2026-08-27) |
| stylo (servo/stylo) | 6091d92b1d6f06938572bfb49d27bb5272718d46 (2026-10-15); stylo 0.22.0, selectors 0.41.0 |
| cssparser (servo/rust-cssparser) | 9f3ceddf398f5ce4ab76fdad1b42ccb6c578a926 (2026-10-08); 0.38.0 |
| html5ever (servo/html5ever) | 7760920edff08e6dfd4b62affee39d8084e9dc45 (2026-09-14); 0.40.1, web_atoms 0.3.0 |
| url (servo/rust-url) | 00a6ce58d02f4e0d43c5ca0702c0bedb8b1ebf3a (2026-07-31); url 2.5.8, idna 1.1.0 |
| proxy binary sha256 | off `8c868c3abe1cd0a2f989b4c99cdc9e0f004b7d843fc0b33dc540ebbf2e3c8b24`, on `9a1ef5aebf9d6906946e1f3c6680e823662bf03d715630afb535fae0e566dfc7` |

Inputs (fetched 2026-10-18 into `~/izcal-measure/cs-d2/inputs`):

| file | source | bytes | sha256 |
|---|---|---|---|
| bootstrap-5.3.3.css | https://cdnjs.cloudflare.com/ajax/libs/bootstrap/5.3.3/css/bootstrap.css | 281,046 | 18a105d7cb38e01e5ed0ca255c092992a2e211b39594a7fa57262bfc6fc4ea9c |
| bulma-1.0.2.css | https://cdnjs.cloudflare.com/ajax/libs/bulma/1.0.2/css/bulma.css | 763,189 | 113149035b8b7543113ca93d6f8c75a7ff4a73a1c0877d5e50c710aa1fe5f404 |
| wikipedia-web-browser.html | https://en.wikipedia.org/wiki/Web_browser | 415,336 | eacf017cf37c3c107daa429b9933019e149d08edf1e5c7378f1965aa2d505d69 |
| bootstrap-docs-navbar.html | https://getbootstrap.com/docs/5.3/components/navbar/ | 358,858 | 5bd08ce8efc9c6e9e512250bb867e6755eb02fc6fd9257f0a07a8d049415a869 |

## Exact commands

```bash
# setup on revire
mkdir -p ~/izcal-measure/cs-d2 && cd ~/izcal-measure/cs-d2
export RUSTUP_HOME=$PWD/rustup CARGO_HOME=$PWD/cargo PATH=$PWD/cargo/bin:$PATH
wget -q https://sh.rustup.rs -O rustup-init.sh && sh rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable
git clone https://github.com/servo/stylo; git clone https://github.com/servo/rust-cssparser cssparser
git clone https://github.com/servo/html5ever; git clone https://github.com/servo/rust-url url
# (inputs fetched with wget from the URLs above; proxy/ is in this folder)

# proxy builds
cd proxy
cargo build --release --target-dir ../target-off
CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true cargo build --release --target-dir ../target-on

# own benches
cd ../html5ever && cargo bench -p html5ever --bench html5ever --no-run --target-dir ../tgt-h5-off
CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true CARGO_PROFILE_BENCH_OVERFLOW_CHECKS=true \
  cargo bench -p html5ever --bench html5ever --no-run --target-dir ../tgt-h5-on
cd ../url && cargo bench -p url --bench parse_url --no-run --target-dir ../tgt-url-off
CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true CARGO_PROFILE_BENCH_OVERFLOW_CHECKS=true \
  cargo bench -p url --bench parse_url --no-run --target-dir ../tgt-url-on

# per benchmark W, build B in {off,on}, 3 interleaved reps (orchestrated by run_measure.py from the Mac)
rig check revire --kind exclusive --session browser-61          # before each benchmark; must exit 0
sudo perf stat -x, -e instructions:u,cycles:u -- taskset -c 4 target-$B/release/csd2-proxy inputs $W $N 0 1
sudo perf stat -x, -e instructions:u,cycles:u -- taskset -c 4 target-$B/release/csd2-proxy inputs $W 0 0 1
taskset -c 4 target-$B/release/csd2-proxy inputs $W 15 $WARM $BATCH      # wall time, 3 processes per build
# own benches (run_own.sh), 3 interleaved rounds, rig check before each run
taskset -c 4 tgt-h5-$B/release/deps/html5ever-* --bench --noplot --warm-up-time 2 --measurement-time 5 --discard-baseline
taskset -c 4 tgt-url-$B/release/deps/parse_url-* --bench
```

Scripts and sources in this folder: `proxy/` (Cargo.toml, Cargo.lock, src/main.rs),
`run_measure.py`, `run_own.sh`, `analyze.py`; raw data in `raw/`
(`results.json`, 17 MB, holds every perf count and wall sample and is kept outside the repository on revire at `~/izcal-measure/cs-d2/raw/results.json`, sha256 `5c0841e52c8d09f5c475205f262f57b435253df4c43f9293d73f1526bc8c0fb5`; run `analyze.py <path>` on it; also `run.log`, `own.log`).

## Raw numbers: proxy driver (instructions primary, wall secondary)

Δ% = on/off − 1 (positive = the checked build is slower or executes more). Instruction counts are the median of
3 reps, per workload call.

| benchmark | instr/iter off | instr/iter on | instr Δ% | instr spread (max-min)/med off, on | cycles Δ% | wall median off (µs) | wall median on (µs) | wall Δ% | wall IQR off / on (µs) |
|---|---|---|---|---|---|---|---|---|---|
| cssparser_tokenize_bootstrap | 12,715,332 | 13,633,035 | +7.22 | 0.0000%, 0.0000% | +3.90 | 3,207.67 | 3,369.45 | +5.04 | 6.43 / 12.82 |
| cssparser_tokenize_bulma | 33,525,890 | 36,155,031 | +7.84 | 0.0000%, 0.0001% | +7.18 | 7,788.52 | 8,336.77 | +7.04 | 26.19 / 280.54 |
| stylo_parse_bootstrap | 43,035,884 | 43,922,618 | +2.06 | 0.0000%, 0.0000% | -1.82 | 13,026.45 | 12,840.19 | -1.43 | 306.59 / 42.91 |
| stylo_parse_bulma | 110,081,265 | 113,202,265 | +2.84 | 0.0001%, 0.0001% | +0.56 | 28,151.98 | 28,483.29 | +1.18 | 405.06 / 283.99 |
| stylo_stylist_bootstrap | 5,370,018 | 5,685,994 | +5.88 | 0.0011%, 0.0006% | +0.73 | 1,647.09 | 1,663.87 | +1.02 | 4.07 / 9.18 |
| stylo_stylist_bulma | 8,534,934 | 8,944,996 | +4.80 | 0.0005%, 0.0007% | +0.16 | 2,646.87 | 2,654.98 | +0.31 | 4.54 / 8.38 |
| selectors_match_bootstrap_navbar | 4,073,682,938 | 4,054,777,604 | -0.46 | 0.0000%, 0.0000% | -0.30 | 834,418.01 | 835,936.29 | +0.18 | 582.15 / 1,171.27 |
| html5ever_treebuild_wikipedia | 65,369,681 | 70,488,309 | +7.83 | 0.0000%, 0.0000% | +5.59 | 14,617.35 | 15,517.69 | +6.16 | 1,310.67 / 166.60 |
| html5ever_treebuild_navbar | 92,191,237 | 98,634,332 | +6.99 | 0.0000%, 0.0000% | +9.24 | 19,607.82 | 21,044.08 | +7.32 | 514.87 / 97.07 |
| html5ever_tokenize_lipsum | 38,888 | 41,778 | +7.43 | 0.0304%, 0.0168% | -0.00 | 5.01 | 5.00 | -0.05 | 0.04 / 0.01 |
| html5ever_tokenize_lipsum_zh | 14,081 | 14,797 | +5.08 | 0.1888%, 0.1638% | +0.81 | 1.55 | 1.56 | +0.64 | 0.03 / 0.02 |
| html5ever_tokenize_medium_fragment | 423,715 | 465,974 | +9.97 | 0.0380%, 0.0053% | +9.42 | 79.53 | 88.47 | +11.24 | 1.53 / 5.06 |
| html5ever_tokenize_small_fragment | 41,292 | 45,170 | +9.39 | 0.1137%, 0.0532% | +8.11 | 7.18 | 7.92 | +10.34 | 0.32 / 0.03 |
| html5ever_tokenize_tiny_fragment | 7,533 | 7,724 | +2.54 | 0.0529%, 0.2620% | +1.95 | 0.74 | 0.82 | +11.47 | 0.00 / 0.00 |
| html5ever_tokenize_strong | 317,804 | 346,170 | +8.93 | 0.0156%, 0.0019% | +12.17 | 51.64 | 57.57 | +11.48 | 2.27 / 0.10 |
| url_join_wikipedia | 4,644,497 | 4,684,809 | +0.87 | 0.0003%, 0.0002% | +3.74 | 893.69 | 909.88 | +1.81 | 19.84 / 3.47 |
| url_parse_own_bench_strings | 27,152,027 | 27,678,024 | +1.94 | 0.0000%, 0.0000% | +1.76 | 5,514.39 | 5,629.93 | +2.10 | 60.41 / 54.86 |

| crate | n | instr geomean Δ% | cycles geomean Δ% | wall geomean Δ% |
|---|---|---|---|---|
| cssparser | 2 | +7.53 | +5.53 | +6.04 |
| stylo | 4 | +3.88 | -0.10 | +0.26 |
| selectors | 1 | -0.46 | -0.30 | +0.18 |
| html5ever | 8 | +7.25 | +5.83 | +7.23 |
| url | 2 | +1.40 | +2.75 | +1.95 |
| overall (geomean of crate geomeans) | 5 | +3.87 | +2.71 | +3.09 |
| overall (geomean of all benchmarks) | 17 | +5.32 | +3.64 | +4.37 |
| renderer-only (crate geomean, excl. url) | 4 | +4.50 | +2.70 | +3.38 |

## Raw numbers: the crates' own benchmarks (wall time)

criterion point estimate (html5ever) or bencher ns/iter (url) per round; Δ% compares the medians of 3 rounds.

| crate | bench | off ns (3 rounds) | on ns (3 rounds) | Δ% (median) |
|---|---|---|---|---|
| html5ever | lipsum.html | 4173.7, 4170.6, 4179.2 | 4306.2, 4323.5, 4299.3 | +3.17 |
| html5ever | lipsum-zh.html | 2727.1, 2724.9, 2728.7 | 2773.3, 2788.8, 2786.9 | +2.19 |
| html5ever | medium-fragment.html | 66182, 68984, 66205 | 76111, 75136, 76487 | +14.96 |
| html5ever | small-fragment.html | 5844.9, 6237, 5765 | 6798.8, 6700.8, 6860.7 | +16.32 |
| html5ever | tiny-fragment.html | 625.67, 640.47, 626.36 | 710.84, 672.29, 672.38 | +7.35 |
| html5ever | strong.html | 44326, 42687, 44324 | 47130, 47142, 47145 | +6.36 |
| url | fragment | 533, 532, 531 | 561, 541, 631 | +5.45 |
| url | hyphen | 502, 501, 502 | 522, 543, 568 | +8.17 |
| url | leading_digit | 515, 516, 515 | 549, 549, 549 | +6.60 |
| url | long | 482, 481, 481 | 492, 492, 492 | +2.29 |
| url | plain | 343, 343, 341 | 352, 353, 352 | +2.62 |
| url | port | 503, 504, 504 | 506, 507, 506 | +0.40 |
| url | punycode_ltr | 1320, 1323, 1324 | 1288, 1287, 1287 | -2.72 |
| url | punycode_mixed | 938, 961, 942 | 961, 986, 961 | +2.02 |
| url | punycode_rtl | 1310, 1311, 1310 | 1314, 1317, 1316 | +0.46 |
| url | short | 383, 383, 383 | 393, 393, 392 | +2.61 |
| url | unicode_ltr | 1431, 1436, 1458 | 1493, 1487, 1504 | +3.97 |
| url | unicode_mixed | 1091, 1087, 1080 | 1216, 1207, 1245 | +11.87 |
| url | unicode_rtl | 1388, 1387, 1391 | 1487, 1472, 1500 | +7.13 |
| url | url_to_file_path | 387, 403, 395 | 389, 388, 388 | -1.77 |
| html5ever | **geomean** (6) | | | **+8.26** |
| url | **geomean** (14) | | | **+3.43** |

## Headline aggregates

| aggregate | instructions | cycles | wall |
|---|---|---|---|
| overall, geomean of 5 crate geomeans | **+3.87%** | +2.71% | **+3.09%** |
| overall, geomean of all 17 proxy benchmarks | +5.32% | +3.64% | +4.37% |
| renderer crates only (stylo, selectors, cssparser, html5ever) | +4.50% | +2.70% | +3.38% |
| html5ever own criterion bench (6) | n/a | n/a | +8.26% |
| url own bencher bench (14) | n/a | n/a | +3.43% |
| 1% threshold (CS-D2) | over | over | over |

## rig check evidence

Booking: `booked b-4c0108 exclusive cpu revire owner=browser-61 13:42-16:42 ... "CS-D2 overflow-check cost proxy"`,
released at 14:07 (`released b-4c0108 ...`). Every check exited 0.

Before each proxy benchmark:
- cssparser_tokenize_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.14/cpu (3.29 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:10-06:00` (exit 0)
- cssparser_tokenize_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.12/cpu (2.78 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:22-06:00` (exit 0)
- stylo_parse_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.10/cpu (2.39 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:36-06:00` (exit 0)
- stylo_parse_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.09/cpu (2.17 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:49-06:00` (exit 0)
- stylo_stylist_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.08/cpu (1.91 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:03-06:00` (exit 0)
- stylo_stylist_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.07/cpu (1.77 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:14-06:00` (exit 0)
- selectors_match_bootstrap_navbar: `rig check OK: revire kind=exclusive session=browser-61 | load 0.07/cpu (1.60 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:27-06:00` (exit 0)
- html5ever_treebuild_wikipedia: `rig check OK: revire kind=exclusive session=browser-61 | load 0.05/cpu (1.11 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:06-06:00` (exit 0)
- html5ever_treebuild_navbar: `rig check OK: revire kind=exclusive session=browser-61 | load 0.05/cpu (1.09 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:20-06:00` (exit 0)
- html5ever_tokenize_lipsum: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.92 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:36-06:00` (exit 0)
- html5ever_tokenize_lipsum_zh: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.93 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:48-06:00` (exit 0)
- html5ever_tokenize_medium_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.86 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:59-06:00` (exit 0)
- html5ever_tokenize_small_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.89 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:11-06:00` (exit 0)
- html5ever_tokenize_tiny_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.91 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:23-06:00` (exit 0)
- html5ever_tokenize_strong: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.93 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:36-06:00` (exit 0)
- url_join_wikipedia: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.94 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:46-06:00` (exit 0)
- url_parse_own_bench_strings: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.95 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:58-06:00` (exit 0)

Before each own-bench run:
- rig check (round 1 off): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.96 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:58:14-06:00 exit=0
- rig check (round 1 on): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (1.00 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:59:50-06:00 exit=0
- rig check (round 2 on): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (1.02 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T14:01:27-06:00 exit=0
- rig check (round 2 off): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (1.00 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T14:02:53-06:00 exit=0
- rig check (round 3 off): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (1.04 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T14:04:23-06:00 exit=0
- rig check (round 3 on): rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (1.01 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T14:05:50-06:00 exit=0

## Caveats

1. **std is precompiled without overflow checks.** The rustup `std`/`core`/`alloc` are built in release mode
   with checks off. Arithmetic inside std (Vec growth, iterators that are not inlined, hashbrown inside std's
   HashMap, string formatting) stays unchecked in both builds, but generic or inlined std code
   instantiated in these crates does get checked. A `-Zbuild-std` build would add some cost on top, so this
   proxy is, if anything, a slight **under**estimate for "checks on everywhere".
2. **Proxy crates, not Izcal's own code.** These are mature Servo crates, tuned under overflow-checks-off
   release builds. They use `wrapping_*` in hashing and `usize` index arithmetic in parsers. Izcal's own tier R
   code, written under the coding standard, may use more explicit `checked_*` or `wrapping_*` and more
   bounds-checked indexing, which changes the marginal cost of global checks in either direction. Tier R will
   also contain layout, painting and the DOM, which are not measured here.
3. **Not Speedometer 3 or P-1.** These are hot-loop microbenchmarks. End-to-end page metrics include JS
   execution, GC, layout, raster, compositing and idle time, which dilute the parser cost (see the conclusion).
4. **Old CPU.** The Ivy Bridge-EP (2013) has a narrower out-of-order core than current x86 or Apple M-series
   chips. Instruction counts carry over well; wall and cycle costs on newer, wider cores may be somewhat
   lower. The stylo results already show that cycles can stay flat while instructions rise.
5. **Selector-matching harness is naive.** It runs all-pairs `matches_selector` with no rule hash or Bloom
   filter and a hand-rolled arena `Element`, so it over-weights the matching inner loop compared with
   Stylo-in-Servo. Its result (−0.46% instructions, +0.18% wall) is a codegen artifact of a loop dominated by
   pointer chasing and atom compares, and should not be read as "checks make matching faster".
6. **Stylo workloads stop short of the cascade.** Computing styles per element (`TElement` + cascade) needs a
   full DOM binding, which was out of scope. Parsing and cascade-data building are covered; property
   computation is not.
7. **Wall-time noise.** The governor (schedutil) and turbo were left as found. Some workloads have a wide IQR
   in one build (e.g. stylo_parse_bootstrap off 307 µs IQR). The tiny-fragment proxy wall time (+11.5%) is
   much larger than its +2.5% instruction delta, because the workload is sub-µs. Instructions are the
   primary metric for this reason.
8. **Failures and deviations.** The first orchestration attempt failed before measuring anything (output dir
   missing) and was rerun. Callgrind/gungraun was not used because Valgrind is not installed on revire. perf
   was run via passwordless sudo instead of changing `perf_event_paranoid`. cssparser's own `#[bench]`s
   (nightly-only) and stylo (no in-repo benches) were covered by the custom workloads instead.
