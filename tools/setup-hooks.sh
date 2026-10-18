#!/usr/bin/env bash
# One-time setup per clone: point git at the tracked hooks in .githooks/,
# for the project and for the rusty_v8 fork submodule, and report missing
# tools. Re-run after cloning or adding a worktree of the fork.
set -eu
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

git -C "$ROOT" config core.hooksPath .githooks
echo "project: core.hooksPath=.githooks"

# The fork is its own repository; point it at the project's hooks by
# absolute path (its worktrees share this setting).
if git -C "$ROOT/m0/rusty_v8" rev-parse --git-dir >/dev/null 2>&1; then
  git -C "$ROOT/m0/rusty_v8" config core.hooksPath "$ROOT/.githooks"
  echo "fork:    core.hooksPath=$ROOT/.githooks"
else
  echo "fork:    m0/rusty_v8 not checked out (git submodule update --init m0/rusty_v8), skipped"
fi

missing=""
for t in gitleaks shellcheck actionlint zizmor cargo-deny rustfmt python3 claude; do
  command -v "$t" >/dev/null || missing="$missing $t"
done
if [ -n "$missing" ]; then
  echo "missing tools:$missing"
  echo "  macOS: brew install gitleaks shellcheck actionlint zizmor cargo-deny; rustfmt comes with rustup;"
  echo "  claude is the Claude Code CLI (used by the review check)"
else
  echo "all check tools present"
fi
