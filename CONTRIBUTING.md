# Contributing to Izcal

This covers the project, [Rorqualx/izcal](https://github.com/Rorqualx/izcal) (branch `main`). The 0.2 fork is retired (C-10): the engine is Izcal's own, and the fork-specific text below is kept as history. The git hooks and CI enforce every rule below that a machine can check. The hooks catch problems early, and CI catches anything committed with `--no-verify`.

## Setup

```
git clone https://github.com/Rorqualx/izcal.git
cd izcal
tools/setup-hooks.sh
```

`setup-hooks.sh` points git at `.githooks/` for the project. It also lists any missing tools. The checks need `gitleaks`, `shellcheck`, `actionlint`, `zizmor` and `cargo-deny` (`brew install gitleaks shellcheck actionlint zizmor cargo-deny`), `rustfmt` (from rustup), `python3`, and the Claude Code CLI `claude` for the review. A missing tool gives a warning, not a failure, except where noted.

## Branches

- **`main` and release tags `v*` are protected by server-side repository rulesets.** They change only through pull requests, and the hooks also refuse commits and pushes on `main`. `hgba`, the 0.2 fork, is historical (C-10). Required checks run from the protected base branch, pinned by commit, never from the pull request's own copy (SEC-27).
- **Work on a feature branch**, named `<type>/<topic>` with types `feat`, `fix`, `docs`, `spec`, `spike`, `ci`, `build`, `chore` and `rebase`. Examples: `spec/s4e-backport-rule`, `fix/windows-teardown`.
- Keep branches short-lived: one topic, merged or closed within days. Rebase on the latest `main` rather than merging it in.

## Worktrees

Use one worktree per branch, and one per agent session when several Claude Code sessions run at once. Parallel sessions must never share a working tree.

- **Project:**

  ```
  git worktree add ../izcal-wt/<topic> -b <type>/<topic> origin/main
  ```

  A new worktree has no fork checkout, and none is needed (C-10); the 0.2 submodule instructions apply only to historical work on the fork.
- **Fork (retired, C-10):** historically the worktree came from the fork itself, so it shared the fork's hooks setting (the commands are in git history):

  ```
  git -C m0/<fork> worktree add ../../fork-wt/<topic> -b <type>/<topic> izcal/hgba
  ```

  Each fork worktree builds its own engine: about 2 GB of output and 17–30 minutes per configuration. Disk is not a constraint, but time is, so set `SCCACHE=sccache`. When the work does not change the engine or the fork's C++, point `ENGINE_ARCHIVE` at an existing archive instead of rebuilding.
- **Cleanup**, after the branch is merged. Check that the worktree has nothing uncommitted (`git -C <worktree> status`) and that its branch is merged, then:

  ```
  git worktree remove <path>
  git branch -d <branch>
  git worktree prune
  ```

  Never delete a worktree folder by hand, and never use `--force` on a worktree that has changes.

## Commits

- **One logical change per commit**, building and passing tests on its own. The 0.2 rule that fork commits stay separable for an S-4e rebase retires with the fork (C-10).
- **Commit messages:** a subject of the form `<area>: <summary>`, with a lowercase area, at most 72 characters, no trailing period, and a blank line before the body. Examples: `ci: cache submodule state`, `spec: restate S-4e as an update policy`. The body says why, not what. The `commit-msg` hook enforces the subject rules. `Merge`, `Revert`, `fixup!` and `squash!` subjects are exempt.
- **Trailers:** commits written with an AI agent end with its `Co-Authored-By` line (and `Claude-Session` where the tool provides one).

## Checks

`pre-commit` runs `tools/checks.sh --staged --review` on what is staged; it checks the index, not the working tree. `pre-push` re-runs the deterministic checks on the commits being pushed, and CI runs them on every pull request. Required checks run the workflow from the protected base branch, pinned by commit, so a pull request cannot disable its own checks (SEC-27).

| Check | Fails when |
|---|---|
| `conflict` | A file contains merge-conflict markers |
| `size` | A file is over 5 MB |
| `binary` | A build output or archive is added (`.a`, `.lib`, `.o`, `.dll`, `.so`, `.dylib`, `.exe`, `.pdb`, `.gz`, `.xz`, `.zst`, `.tar`, `.zip`, `.7z`) |
| `secrets` | `gitleaks` finds a credential. Without gitleaks, a regex covers GitHub, AWS, Slack and API keys and private-key blocks |
| `rustfmt` | A Rust file is not formatted under its crate's edition and `rustfmt.toml` (the fork's files use the fork's pinned toolchain) |
| `shellcheck` | A shell script has a warning or error. Scripts under `logs*/` folders are records of what ran in M0 and are not checked |
| `actionlint` | A GitHub workflow has an error |
| `mdlinks` | A relative Markdown link points at a file that does not exist |
| `revision` | A frozen spec changes without adding or changing a revision or status line (see "Specs") |
| `security` | Added Rust `unsafe` lacks a `// SAFETY:` comment just above it; an added line disables TLS verification, pipes a download into a shell or makes files world-writable; a changed GitHub workflow fails zizmor (unpinned actions, persisted credentials, cache poisoning, template injection); a changed `Cargo.lock` has a known vulnerability (cargo-deny with `tools/deny.toml`: vulnerabilities fail, unmaintained notices do not) |
| `provenance` | Added code carries another browser project's copyright or license header (Chromium, V8, Servo, Mozilla, Apple/WebKit, KDE, SerenityOS/Ladybird) or a copyleft SPDX tag (see "Original code") |
| `review` | Advisory: the AI code review reports a high-severity defect (below). It is not an approval, a required check or a security control (SEC-27) |

**Skipping a check** is allowed only with a reason: `IZCAL_HOOKS_SKIP=review git commit ...`, with the reason in the commit body. The `commit-msg` hook adds a `Hooks-Skipped:` trailer, so the skip is visible in history. Never use `--no-verify` to skip checks. `pre-push` and CI re-run them anyway.

## Code review

- **AI review, on every commit that changes code.** The pre-commit hook sends the staged diff of code files (Rust, C++, Python, shell, TOML, YAML, gn; not Markdown or logs) to Claude Code (`IZCAL_REVIEW_MODEL`, default `sonnet`), with read-only access to the repository. It reports correctness, memory-safety, GC, security and build defects. Its security checklist is specific to Izcal: unvalidated data crossing a process or IPC boundary, capabilities passed further than needed, ambient authority in a sandboxed process, untrusted input parsed in a privileged process, cross-site data reaching one process, secrets logged or exported, missing consent or origin checks on inspection and agent surfaces, weak or hand-rolled cryptography, unsafe Rust whose safety argument does not hold, and injection, and checks the rules in this file and in the specs. A high-severity finding is printed and blocks the commit by default, but the review is **advisory**: it is not an approval or a security control, and a maintainer may override it with a recorded reason. Medium findings are printed as warnings. If the reviewer cannot run (no CLI, no network, timeout), the commit goes through with a warning. A typical review takes 5–30 seconds and costs about $0.01–0.10.
- **The one carve-out for hand-rolled cryptography.** Izcal writes its own cryptography on purpose: `hg-crypto` (networking spec NW-D23) and, since batch 3, `hg-webcrypto` and the WebCrypto additions to `hg-crypto` (WebCrypto spec WC-D1). The checklist's "weak or hand-rolled cryptography" item is **not** satisfied by pointing at that crate: the carve-out makes the code expected, not exempt. Hand-written crypto in `hg-crypto` or `hg-webcrypto` is in the yearly external audit (NW-52) and in the same gates as any other code, so the finding to raise there is a missing test vector, a non-constant-time comparison, a key path `expose_bytes` does not cover, or a primitive the audit has not approved — never the fact that Izcal wrote it. Hand-written crypto anywhere else is still a high-severity finding.
- **Pull requests** need green CI. Self-merge is allowed, because the project has one maintainer, and every self-approval is recorded in the pull request and counted in the quarterly security report (SEC-27). Once there are two, one approval from someone other than the author is required, and the AI review does not count as that approval.

## Merging

- **Project (`main`): squash merge.** The pull request title becomes the commit subject, so it follows the commit-message format.
- **Fork (`hgba`, retired — C-10):** the 0.2 rebase-merge rule, kept as history.
- Delete the branch after merging. GitHub does this automatically.
- **Submodule pointer bumps (retired, C-10):** the 0.2 rule. Engine images replace the fork submodule, and their hashes are verified at build time (SEC-13); there are no fork hash pins.

## Pushing

- **Feature branches:** push freely. `git push --force-with-lease` is fine on your own branch, never on someone else's.
- **Protected branches:** never pushed directly. The `IZCAL_ALLOW_PROTECTED=1` override exists only for repository administration, and GitHub's protection refuses the push anyway.
- **CI cost (0.2 fork, retired — C-10):** every push to `hgba` ran the full fork build matrix; the rule retires with the fork.
- **Release tags** `v*` are protected by a server-side ruleset that requires signed tags and records any bypass; the client-side `IZCAL_RELEASE=1` rule retires (SEC-27). The 0.2 fork tag rule (`v<upstream>-izcal.<n>`) retires with the fork (C-10).

## The S-4e upstream rebase (retired, C-10)

*This section is history. The engine is Izcal's own, so there is no upstream rebase and no force-push rule; engine image hashes are verified at build time instead (SEC-13).* The 0.2 text: every Chrome stable release, the fork owner rebased the fork's commits onto the new upstream tag (HGBA-2.1 S-4e). This was the one time `hgba` was rewritten:

1. On a `rebase/v<new>` branch, rebase the fork's commits onto the upstream tag. Fix conflicts commit by commit.
2. Open a pull request into `hgba` so CI runs the full fork matrix on the rebased branch, and complete S-4e steps 2–3.
3. Tag the old tip so its commits stay reachable, because the project's submodule history pins them: `git tag hgba-pre-v<old> izcal/hgba && git push izcal hgba-pre-v<old>`.
4. Temporarily allow force pushes on `hgba` (repository settings, or `gh api`), then push: `IZCAL_S4E_REBASE=1 git push --force-with-lease izcal rebase/v<new>:hgba`. `pre-push` refuses the push unless the old tip is tagged `hgba-pre-*`. Restore the protection immediately afterwards.
5. Bump the project's submodule pointer.

If the rebase cannot pass inside the security window, follow S-4e's backport rule instead: cherry-pick the upstream security commits onto the pinned engine through an ordinary pull request.

## Specs

- The frozen specs are `reports/HGBA 2 browser architecture spec.md`, `reports/Future software browser engine.md` (HGBA-3) and `reports/HGBA Stage 1 architecture design.md`. A content change adds a revision line to the header (`*Revision x.y.z (date): what changed.*`) and bumps the frozen or draft version. The `revision` check enforces this.
- In the same pull request, update the version pins that other documents quote (the README's table and each report's header).
- Every factual claim needs a source in `research_notes/` or an inline link. Mark unverified claims as unverified.

## Original code

Izcal is a new engine written from the ground up: it neither copies another browser's code nor uses another browser's components. The rule is HGBA-2's method constraint **M-1** (original, first-principles, spec-anchored implementation, revision 2.3.3) and the Stage 1 design's D-S10.

- **No code from another browser or browser engine is copied, ported or translated into Izcal**, whatever its license. That covers Servo, Chromium and Blink, WebKit, Gecko and Firefox, Ladybird, and their forks. It applies to people and to AI tools alike: an agent asked to "do what Chromium does" works from the specification, not from Chromium's source.
- **The specifications are the source.** Other engines are black-box references only: differential testing, WPT results, observed behavior. If you read another engine's source to understand a bug, do not write Izcal code from it, and do not cite it: specs, research notes and design records never link to or take values from any third-party project's source code, engine or not (M-1 part 3).
- **Build our own, following our specs.** When a component would otherwise come from another browser or from a library that does browser-engine work (a JS engine, a parser, a style or layout engine, a text shaper, a renderer, a protocol stack, a frontend, a tool, an importer), Izcal builds its own from the specifications. That is the standing answer, not a case-by-case question. Izcal's DevTools frontend was the first example (D-S11).
- **Dependencies are generic infrastructure only.** Allowed: the Rust standard library; general-purpose crates whose job is not browser-engine work (data structures, serialization, error handling, hashing, logging, test tooling); cryptography primitives from MIT-licensed crates (the TLS protocol is Izcal's own); OS interfaces and drivers (platform SDKs, Metal, Vulkan, Direct3D 12, OS media and accessibility frameworks). Not allowed, whoever wrote them: V8 or any JS engine, Stylo, `selectors`, `cssparser`, html5ever, WebRender, wgpu, Taffy, Parley, HarfBuzz and its ports, Fontations, neqo, `adblock`, or any other library doing engine work. Conformance suites (WPT, test262) and standards data (the Unicode Character Database, IANA registries, the Public Suffix List) are oracles and data, not engine code. Every shipped dependency is also MIT (see License). A dependency whose class is unclear needs an owner decision recorded in the Stage 1 design; the default is to build it.
- **Anchor every algorithm to its specification.** Code that implements web-platform behavior cites the specification section and step it implements: `// spec: https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate step 12`. Specifications are pinned in `specs.lock` next to the WPT pin, and `hg-anchor` checks that every anchor resolves and reports coverage (crate rule G9, target ANC-1). Where a specification is silent or wrong, record the engines' observed behavior in `compat/observations/` (a test case and each engine's result, never a link to their source), make the decision, and file a specification issue.
- **Internal design comes from first principles.** For anything no specification governs (data layout, scheduling, IPC, caching), the design record derives the choice from Izcal's principles and measurements. Another engine's design may be cited as evidence that an approach works, never as a template.
- **Enforcement.** The `provenance` check fails on code that carries another browser project's copyright or license header. The AI review reports code that looks copied or translated from another engine, and a maintainer decides.

## Coding standard

Izcal's own Rust code follows [the coding standard](reports/Izcal%20coding%20standard.md). Read its one-page summary before writing code; `cargo xtask tier <crate>` lists the rules for a crate's tier.

The 0.2 `v8-audit` check retired with the fork (C-10; ledger A-CON-4); no check reads a fork checkout.

- **Trailers go in the pull request body**, not in commits: `Spec:`, `Test:`, `Perf:`, `Binary-Size:` and `Fixes:` (CS-GA-3). The repository squashes with "pull request title and description", so the body becomes the commit message and the trailers stay in history.
- **Suppressions** are `#[expect(..., reason = "...")]`, or a `// cs-allow(<rule>): <reason>` marker on the line an `ast-grep` check matches. The reason states a fact a reviewer can check; CI lists every new one in the PR summary.
- **Agents** read the summary and their tier's rules first, and run `tools/checks.sh --range` before pushing. An agent never edits a gate file, adds a quarantine, inventory, allowlist or bypass entry, weakens or deletes a test, or relaxes a tier unless the task says so and the PR names the file and why. An agent that cannot state a checkable reason stops and reports instead of suppressing (CS-GA-2; the full rules are generated into `tools/coding-standard/agent-rules.md`).

## License

- Contributions are licensed under MIT, the project's license ([LICENSE](LICENSE)); no separate agreement is needed.
- **MIT only.** Every dependency and data set that ships must be available under MIT, alone or as one choice of a dual licence ("MIT OR Apache-2.0" is fine). If it is not, Izcal builds its own; we do not ask authors to relicense. Exceptions (HGBA-2 M-1 part 1): standards data from the standards bodies themselves (Unicode, WHATWG, IANA, tz, OpenType registries, the Public Suffix List, the CT log list); test-only crates and suites that never ship (dev-dependencies, WPT, test262, oracles), whatever their licence; OS interfaces, drivers and the toolchain.
- Release binaries ship the license notices of every dependency and data set.

## Large files and artifacts

Build outputs and engine images never go into git (`.gitignore`, and the `size` and `binary` checks). Publish engine images as release assets and record their hashes; release verification checks them at build time (SEC-13). The 0.2 fork archive rule retires with the fork (C-10).
