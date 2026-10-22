# Q-CS-3: which proptest configuration gives a fixed, printed seed per run

Measured 19 October 2026 for CS-TE-10 ("its seed comes from the run's test seed").

## Question

Which proptest configuration makes a run use one fixed seed, prints that seed, and reproduces the same inputs across invocations and machines?

## Method

`probe/` is a throwaway crate on proptest **1.12.0** (the current release). `tests/seed.rs` has three properties over the same strategy, `(any::<u64>(), "[a-z]{0,8}")`, and each one appends every generated input to `$QCS3_OUT.<test>`:

- `default_config` uses the default `Config`;
- `fixed_in_config` sets `Config { rng_seed: RngSeed::Fixed(0x1234_5678), .. }` in code;
- `failing` fails on purpose when `QCS3_FAIL` is set, to show what a failure prints.

`probe/run.sh` runs the suite six times (`--test-threads=1`): twice with no seed, twice with `PROPTEST_RNG_SEED=42`, once with seed 43, and once with seed 42 and `PROPTEST_RNG_ALGORITHM=xs`. It then hashes each input file (256 cases per property). The same script ran on two machines:

| Machine | OS and arch | Toolchain |
|---|---|---|
| `local` | macOS 27.0, arm64 | cargo 1.99.1 |
| `hueyj` | Linux, x86_64 | cargo 1.98.1 |

The failure logs come from `failing` with `QCS3_FAIL=1`, run with a random seed, with seed 42 twice from a clean state, and once with `PROPTEST_VERBOSE=1` after a failure had been persisted. Absolute paths are replaced by `<probe>`.

## Results

Hash prefixes of the 256 generated inputs (`logs/macos-arm64-summary.txt`, `logs/linux-x86_64-summary.txt`):

| Configuration | Run 1 vs run 2 | macOS arm64 | Linux x86_64 |
|---|---|---|---|
| Default (no env) | **differ** | `817587d6db930741`, `94afaaa3388be4bf` | `cb9775bb3c23b65b`, `8d332bb2086dc092` |
| `PROPTEST_RNG_SEED=42` | identical | `66b98f4f90cbe3e7` | `66b98f4f90cbe3e7` |
| `PROPTEST_RNG_SEED=43` | — | `d354bc1e29b61bb4` | `d354bc1e29b61bb4` |
| Seed 42, `PROPTEST_RNG_ALGORITHM=xs` | — | `9024ec8c1090fd6f` | `9024ec8c1090fd6f` |
| In-code `RngSeed::Fixed`, no env | identical | `8b0ab50bb572bfdb` | `8b0ab50bb572bfdb` |
| In-code `RngSeed::Fixed`, `PROPTEST_RNG_SEED=42` | identical | `66b98f4f90cbe3e7` (the env seed's stream) | same |

What this shows:

1. **The default is a random seed** (`RngSeed::Random`, algorithm ChaCha; `config.rs` and `rng.rs`), so two runs differ.
2. **`PROPTEST_RNG_SEED=<u64>` fixes the run.** Inputs are identical across invocations and across macOS arm64 and Linux x86_64 with different rustc versions, for both RNG algorithms. Pinning `PROPTEST_RNG_ALGORITHM=cc` (ChaCha, the default) guards against a default change.
3. **The environment overrides the config written in code.** The `proptest!` macro passes every config through `contextualize_config` (`sugar.rs`), which applies `PROPTEST_*` variables on top. So a test's `RngSeed::Fixed` silently gives way to the run's seed. A `TestRunner::new(config)` built by hand skips that step and ignores the env.
4. **Every property in a run gets the same stream.** All three properties produced identical inputs under one seed. Properties with the same strategy test the same values; per-property streams would need a seed derived per test, which proptest does not do.
5. **proptest never prints the run's seed**, on success or failure, even with `PROPTEST_VERBOSE=1` or `2` (verbose prints each input). On failure it prints the minimal input and a per-case `cc <hex>` line, and saves that line to `<source>.proptest-regressions` (failure persistence). For an integration test in `tests/`, it first warns that `SourceParallel` "failed to find lib.rs or main.rs" and then saves next to the test file.
6. **With seed 42, a failure reproduces exactly:** the same `cc 32e566a4…` line, the same minimal input, and `successes: 7` in two clean runs (`logs/failure-env-seed-42*.log`). With a random seed it does not.
7. **Persisted cases run before the seeded stream.** After a failure was persisted, the next seed-42 run replayed the stored case first and failed at once (`successes: 0`, `logs/failure-replay-from-persistence.log`). A regression file therefore changes what a fixed-seed run executes, which is deterministic only if that file is checked in.

## Answer

No proptest setting both fixes and prints the seed. The configuration that meets CS-TE-10 is:

- **The runner owns the seed.** The test runner (`hg-wpt`/`cargo xtask test`, or the T0 job) takes the run's test seed, sets `PROPTEST_RNG_SEED=<seed>` and `PROPTEST_RNG_ALGORITHM=cc`, and **prints the seed itself** in the job log and on failure.
- **Tests do not set `rng_seed` in code.** It is ignored under `proptest!` anyway, and misleads a reader. Tests use `proptest!`, never a hand-built `TestRunner`, so the env applies.
- **Failure persistence stays on.** `*.proptest-regressions` files are checked in, so replayed cases are part of the deterministic input. The `cc` line is the replay handle a bug report quotes.
