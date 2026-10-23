# Shared by the Izcal hooks. Sourced, not executed.
# shellcheck shell=bash

# main changes only through pull requests.
is_protected() { case "$1" in main) return 0 ;; *) return 1 ;; esac; }

# The project's tools/checks.sh.
checks_script() { echo "$(cd "$HOOKS_DIR/.." && pwd)/tools/checks.sh"; }

default_remote_branch() {
  local r
  r=$(git symbolic-ref -q --short "refs/remotes/$1/HEAD" 2>/dev/null) && { echo "$r"; return; }
  git rev-parse -q --verify "refs/remotes/$1/main" >/dev/null && echo "$1/main"
}
