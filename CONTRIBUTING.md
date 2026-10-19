# Contributing to Izcal

This covers both repositories: the project, [Rorqualx/izcal](https://github.com/Rorqualx/izcal) (branch `main`), and the `rusty_v8` fork, [Rorqualx/rusty_v8-izcal](https://github.com/Rorqualx/rusty_v8-izcal) (branch `hgba`, checked out at `m0/rusty_v8`). The git hooks and CI enforce every rule below that a machine can check. The hooks catch problems early, and CI catches anything committed with `--no-verify`.

## Setup

```
git clone --recurse-submodules https://github.com/Rorqualx/izcal.git
cd izcal
tools/setup-hooks.sh
```

`setup-hooks.sh` points git at `.githooks/` for the project and for the fork. It also lists any missing tools. The checks need `gitleaks`, `shellcheck`, `actionlint`, `zizmor` and `cargo-deny` (`brew install gitleaks shellcheck actionlint zizmor cargo-deny`), `rustfmt` (from rustup), `python3`, and the Claude Code CLI `claude` for the review. A missing tool gives a warning, not a failure, except where noted.

## Branches

- **`main` (project) and `hgba` (fork) are protected.** They change only through pull requests. The hooks refuse commits on them and pushes to them, and GitHub branch protection refuses pushes too.
- **Work on a feature branch**, named `<type>/<topic>` with types `feat`, `fix`, `docs`, `spec`, `spike`, `ci`, `build`, `chore` and `rebase`. Examples: `spec/s4e-backport-rule`, `fix/windows-teardown`.
- Keep branches short-lived: one topic, merged or closed within days. Rebase on the latest `main` or `hgba` rather than merging it in.

## Worktrees

Use one worktree per branch, and one per agent session when several Claude Code sessions run at once. Parallel sessions must never share a working tree.

- **Project:**

  ```
  git worktree add ../izcal-wt/<topic> -b <type>/<topic> origin/main
  ```

  A new worktree has no fork checkout. Run `git -C ../izcal-wt/<topic> submodule update --init m0/rusty_v8` (not `--recursive`) only if the work needs the fork's sources, and then rerun `tools/setup-hooks.sh` in that worktree.
- **Fork:** create the worktree from the fork itself, so it shares the fork's hooks setting:

  ```
  git -C m0/rusty_v8 worktree add ../../rusty_v8-wt/<topic> -b <type>/<topic> izcal/hgba
  ```

  Each fork worktree builds its own V8: about 2 GB of output and 17–30 minutes per configuration. Disk is not a constraint, but time is, so set `SCCACHE=sccache`. When the work does not change V8 or the fork's C++, point `RUSTY_V8_ARCHIVE` at an existing archive instead of rebuilding.
- **Cleanup**, after the branch is merged. Check that the worktree has nothing uncommitted (`git -C <worktree> status`) and that its branch is merged, then:

  ```
  git worktree remove <path>
  git branch -d <branch>
  git worktree prune
  ```

  Never delete a worktree folder by hand, and never use `--force` on a worktree that has changes.

## Commits

- **One logical change per commit**, building and passing tests on its own. The fork's commits in particular must stay separable: each S-4e rebase replays them onto a new upstream, and they are offered upstream individually.
- **Commit messages:** a subject of the form `<area>: <summary>`, with a lowercase area, at most 72 characters, no trailing period, and a blank line before the body. Examples: `ci: cache submodule state`, `spec: restate S-4e as an update policy`. The body says why, not what. The `commit-msg` hook enforces the subject rules. `Merge`, `Revert`, `fixup!` and `squash!` subjects are exempt.
- **Trailers:** commits written with an AI agent end with its `Co-Authored-By` line (and `Claude-Session` where the tool provides one).

## Checks

`pre-commit` runs `tools/checks.sh --staged --review` on what is staged; it checks the index, not the working tree. `pre-push` re-runs the deterministic checks on the commits being pushed, and CI runs them on every pull request.

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
| `v8-audit` | The fork's gitlink, the CS-D3 audit (`m0/token-cell-spike/audit/`) or its `disallowed-methods` list changed, and a rusty_v8 method that can run script (V8's own `EnterV8Scope`/`PrepareForExecutionScope` marker in `api.cc`, then every caller of one on the same type) is neither in the list nor in `audit/reviewed.toml` with a reason, or a listed method no longer exists. Uses the fork checkout at the committed gitlink, or fetches it (needs `ast-grep` and network) |
| `review` | The AI code review finds a high-severity defect (below) |

**Skipping a check** is allowed only with a reason: `IZCAL_HOOKS_SKIP=review git commit ...`, with the reason in the commit body. The `commit-msg` hook adds a `Hooks-Skipped:` trailer, so the skip is visible in history. Never use `--no-verify` to skip checks. `pre-push` and CI re-run them anyway.

## Code review

- **AI review, on every commit that changes code.** The pre-commit hook sends the staged diff of code files (Rust, C++, Python, shell, TOML, YAML, gn; not Markdown or logs) to Claude Code (`IZCAL_REVIEW_MODEL`, default `sonnet`), with read-only access to the repository. It reports correctness, memory-safety, GC, security and build defects. Its security checklist is specific to Izcal: unvalidated data crossing a process or IPC boundary, capabilities passed further than needed, ambient authority in a sandboxed process, untrusted input parsed in a privileged process, cross-site data reaching one process, secrets logged or exported, missing consent or origin checks on inspection and agent surfaces, weak or hand-rolled cryptography, unsafe Rust whose safety argument does not hold, and injection, and checks the rules in this file and in the specs. A high-severity finding blocks the commit. Medium findings are printed as warnings. If the reviewer cannot run (no CLI, no network, timeout), the commit goes through with a warning. A typical review takes 5–30 seconds and costs about $0.01–0.10.
- **Pull requests** need green CI. Self-merge is allowed, because the project has one maintainer. Once there are two, one approval from someone other than the author is required, and the AI review does not count as that approval.

## Merging

- **Project (`main`): squash merge.** The pull request title becomes the commit subject, so it follows the commit-message format.
- **Fork (`hgba`): rebase merge.** Never squash, so each commit stays individually rebasable and upstreamable (S-4e).
- Delete the branch after merging. GitHub does this automatically.
- **Submodule pointer bumps:** after a fork pull request merges, bump `m0/rusty_v8` in the project with its own pull request (`build: bump rusty_v8 fork to <sha>`). The pointer must name a commit on the fork's `hgba`. `pre-push` refuses a pointer the fork's remote does not have.

## Pushing

- **Feature branches:** push freely. `git push --force-with-lease` is fine on your own branch, never on someone else's.
- **Protected branches:** never pushed directly. The `IZCAL_ALLOW_PROTECTED=1` override exists only for repository administration, and GitHub's protection refuses the push anyway.
- **CI cost:** every push to `hgba` (that is, every merged fork pull request) runs the full V8 build matrix (`izcal-v8-build`, six jobs, about 6–9 runner-hours). Batch fork changes into one pull request when you can.
- **Release tags** `v<upstream>-izcal.<n>` are pushed only by the `rusty_v8` owner, after S-4e step 3 passes. `pre-push` requires `IZCAL_RELEASE=1` for them.

## The S-4e upstream rebase (fork only)

Every Chrome stable release, the `rusty_v8` owner rebases the fork's commits onto the new upstream tag (HGBA-2.1 S-4e). This is the one time `hgba` is rewritten:

1. On a `rebase/v<new>` branch, rebase the fork's commits onto the upstream tag. Fix conflicts commit by commit.
2. Open a pull request into `hgba` so CI runs the full V8 matrix on the rebased branch, and complete S-4e steps 2–3.
3. Tag the old tip so its commits stay reachable, because the project's submodule history pins them: `git tag hgba-pre-v<old> izcal/hgba && git push izcal hgba-pre-v<old>`.
4. Temporarily allow force pushes on `hgba` (repository settings, or `gh api`), then push: `IZCAL_S4E_REBASE=1 git push --force-with-lease izcal rebase/v<new>:hgba`. `pre-push` refuses the push unless the old tip is tagged `hgba-pre-*`. Restore the protection immediately afterwards.
5. Bump the project's submodule pointer.

If the rebase cannot pass inside the security window, follow S-4e's backport rule instead: cherry-pick the V8 security commits onto the pinned V8 through an ordinary pull request.

## Specs

- The frozen specs are `reports/HGBA 2 browser architecture spec.md`, `reports/Future software browser engine.md` (HGBA-3) and `reports/HGBA Stage 1 architecture design.md`. A content change adds a revision line to the header (`*Revision x.y.z (date): what changed.*`) and bumps the frozen or draft version. The `revision` check enforces this.
- In the same pull request, update the version pins that other documents quote (the README's table and each report's header).
- Every factual claim needs a source in `research_notes/` or an inline link. Mark unverified claims as unverified.

## Original code

Izcal is a new engine, not a derivative of another browser. The rule is HGBA-2's method constraint **M-1** (original, first-principles, spec-anchored implementation) and the Stage 1 design's D-S10.

- **No code from another browser or browser engine is copied, ported or translated into Izcal**, whatever its license. That covers Servo, Chromium and Blink, WebKit, Gecko and Firefox, Ladybird, and their forks. It applies to people and to AI tools alike: an agent asked to "do what Chromium does" works from the specification, not from Chromium's source.
- **The specifications are the source.** Other engines are black-box references only: differential testing, WPT results, observed behavior. If you read another engine's source to understand a bug, do not write Izcal code from it.
- **Build our own, following our specs.** When a component would otherwise come from another browser (a frontend, a tool, an importer, a subsystem), Izcal builds its own from the specifications. That is the standing answer, not a case-by-case question. Izcal's DevTools frontend is the first example (D-S11).
- **Dependencies are not copying.** Izcal uses libraries as libraries: V8 (through the `rusty_v8` fork, whose changes are written against V8's public interfaces), Stylo, `selectors`, WebRender, html5ever, Taffy, Parley, wgpu, `adblock`, neqo and rustls. A new dependency that is itself part of another browser needs a decision recorded in the Stage 1 design.
- **Anchor every algorithm to its specification.** Code that implements web-platform behavior cites the specification section and step it implements: `// spec: https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate step 12`. Specifications are pinned in `specs.lock` next to the WPT pin, and `hg-anchor` checks that every anchor resolves and reports coverage (crate rule G9, target ANC-1). Where a specification is silent or wrong, record the engines' observed behavior in `compat/observations/` (a test case and each engine's result, never a link to their source), make the decision, and file a specification issue.
- **Internal design comes from first principles.** For anything no specification governs (data layout, scheduling, IPC, caching), the design record derives the choice from Izcal's principles and measurements. Another engine's design may be cited as evidence that an approach works, never as a template.
- **Enforcement.** The `provenance` check fails on code that carries another browser project's copyright or license header. The AI review blocks code that looks copied or translated from another engine.

## Coding standard

Izcal's own Rust code follows [the coding standard](reports/Izcal%20coding%20standard.md). Read its one-page summary before writing code; `cargo xtask tier <crate>` lists the rules for a crate's tier.

- **Trailers go in the pull request body**, not in commits: `Spec:`, `Test:`, `Perf:`, `Binary-Size:` and `Fixes:` (CS-GA-3). The repository squashes with "pull request title and description", so the body becomes the commit message and the trailers stay in history.
- **Suppressions** are `#[expect(..., reason = "...")]`, or a `// cs-allow(<rule>): <reason>` marker on the line an `ast-grep` check matches. The reason states a fact a reviewer can check; CI lists every new one in the PR summary.
- **Agents** read the summary and their tier's rules first, and run `tools/checks.sh --range` before pushing. An agent never edits a gate file, adds a quarantine, inventory, allowlist or bypass entry, weakens or deletes a test, or relaxes a tier unless the task says so and the PR names the file and why. An agent that cannot state a checkable reason stops and reports instead of suppressing (CS-GA-2; the full rules are generated into `tools/coding-standard/agent-rules.md`).

## License

- Contributions are licensed under MIT, the project's license ([LICENSE](LICENSE)); no separate agreement is needed.
- New dependencies must use a permissive license (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode) or MPL-2.0. Anything else needs a decision recorded in the Stage 1 design.
- Release binaries ship the license notices of every dependency, and say where the source of the MPL dependencies is.

## Large files and artifacts

Build outputs and V8 archives never go into git (`.gitignore`, and the `size` and `binary` checks). Publish archives as release assets of the fork: `izcal-v8-build` does this for tags. Commit their `SHA256SUMS`, `args.gn` and binding files instead.
