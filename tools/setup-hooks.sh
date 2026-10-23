#!/usr/bin/env bash
# One-time setup per clone: point git at the tracked hooks in .githooks/
# and report missing tools. Re-run after cloning or adding a worktree.
# (The 0.2 rusty_v8 fork is retired, C-10: no submodule to repoint.)
set -eu
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

git -C "$ROOT" config core.hooksPath .githooks
echo "project: core.hooksPath=.githooks"

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
