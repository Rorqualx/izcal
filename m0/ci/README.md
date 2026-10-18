# CI

The M0 draft `v8-build.yml` that used to be here was replaced on 17 October 2026 by the fork's own workflow, `../rusty_v8/.github/workflows/izcal-v8-build.yml`, which lives in the fork so it can trigger. Compared with the draft, it:

- checks out the triggering ref (tag builds used to compile the `hgba` tip);
- relies on the `third_party/fadec/src` submodule now in the fork (x64 debug sandbox builds failed without it);
- sets `CLANG_PATH` as well as `LIBCLANG_PATH`, and uses Ubuntu 24.04 as on huey;
- builds a real Windows debug V8 (`GN_ARGS=is_debug=true` plus the `libcmtd` RUSTFLAGS) instead of the "verify" variant;
- fixes the cache paths and keys the cache on submodule commits;
- sizes build jobs from each runner's cores and memory;
- records the fork commit, toolchain and submodules with every archive, and checksums every file.

It has not yet run on GitHub: the fork has no remote.

**Windows jobs verified locally (17 October 2026).** On the Windows laptop, a local runner (`m0/ci/render.py` and `m0/ci/runner.sh`) rendered the workflow's own `run:` steps for the two Windows matrix entries and ran them in order, with `GITHUB_ENV` handling. Both jobs passed end to end at fork commit 895b6e7 plus the staged changes:

| Job | V8 build | Fork tests | Archive (gzipped) |
|---|---|---|---|
| release | 1,050 s at 19 jobs | 5 of 5 | 40.8 MB |
| debug (`GN_ARGS=is_debug=true`, `libcmtd`) | 1,764 s at 11 jobs | 5 of 5 | 87.9 MB |

The run found and fixed four workflow bugs:

- job sizing read 0 MB on a session where CIM fails, which silently dropped to 1 job;
- each `SHA256SUMS` file listed itself, which would have failed the publish check;
- the SDK fallback was not idempotent;
- the `GITHUB_ENV` write could carry a byte-order mark.

It also showed that `test_single_threaded_default_platform` hits the S-4f teardown crash too; it now keeps the platform alive on Windows, like `test_predictable_platform`. The SDK debugging-tools fallback ran for real, because the laptop lacks that feature. macOS and Linux jobs were not run.

Before the first push, disable the upstream workflows (`ci.yml`, `release.yml`, `update-v8.yml`) in the fork's Actions settings.
