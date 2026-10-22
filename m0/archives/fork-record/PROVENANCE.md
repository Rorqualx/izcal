# The rusty_v8 fork: provenance record

The Rorqualx/rusty_v8-izcal fork (branch `hgba`, final head `aabd96b`,
V8 15.2.124.1) was **deleted from GitHub on 22 October 2026**, after the
owner decided the fork was no longer needed (HGBA-2 2.3.0 made it history
on 19 October 2026; the corpus builds its own engine, D-S10).

**What is preserved here:** the build configuration (`.gn`, `BUILD.gn`,
`.cargo/config.toml` — the deterministic-build parameters), the fork's
README (the predictable-platform and cppgc-API design notes), and the five
workflows (including `izcal-v8-build.yml`, the CI that built the V8
archives whose checksums live in this folder's SHA256SUMS).

**Full history:** a git bundle of the fork's complete history survives at
`~/izcal-archives/rusty_v8-izcal.gitbundle` (created 22 October 2026,
SHA-256 `55ccb5f6857c5d7f0bb8303c1ba38ae68109c6e6b3253c639636e14eae8391fd`). Restore with `git clone --bare rusty_v8-izcal.gitbundle`.

The M0 measurements that cite this fork's code (the determinism probes,
the TokenCell spike, the sandbox measurements) keep their evidence in
`m0/` — the results stand on their own; this record preserves where the
code went.
