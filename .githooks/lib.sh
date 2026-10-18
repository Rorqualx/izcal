# Shared by the Izcal hooks. Sourced, not executed.
# shellcheck shell=bash

# main (project) and hgba (rusty_v8 fork) change only through pull requests.
is_protected() { case "$1" in main|hgba) return 0 ;; *) return 1 ;; esac; }

# The project's tools/checks.sh, also when these hooks serve the fork
# (core.hooksPath points the fork at the project's .githooks).
checks_script() { echo "$(cd "$HOOKS_DIR/.." && pwd)/tools/checks.sh"; }

default_remote_branch() {
  local r
  r=$(git symbolic-ref -q --short "refs/remotes/$1/HEAD" 2>/dev/null) && { echo "$r"; return; }
  for b in main hgba; do git rev-parse -q --verify "refs/remotes/$1/$b" >/dev/null && { echo "$1/$b"; return; }; done
}
