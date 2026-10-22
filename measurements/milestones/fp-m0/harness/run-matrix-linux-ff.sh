#!/bin/bash
# FP-M0 Linux matrix, Firefox leg v3. Loads open as ONE invocation with N
# URLs (one window, N tabs) — the attach mechanism (repeated invocations)
# does not work headless: the instance runs the profile lock but no remote
# listener, so later invocations die and leave the tab count at zero (v2
# evidence: identical 13-process/1,431 MB rows at every load). Fresh
# instance per load, so no cross-load state. Same loads as the full
# matrix. Evidence tier: process/memory growth, the macOS Firefox caveat.
set -u
cd "$(dirname "$0")" || exit 1
FIREFOX=firefox
URL=https://example.com
FPROF=$(mktemp -d ~/fp-m0-lin-ff.XXXXXX)
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"

run_load() {  # $1 = label, rest = URLs to open in one window
  local label="$1"; shift
  pkill -f "$FPROF" 2>/dev/null; sleep 2
  "$FIREFOX" -headless -profile "$FPROF" "$@" >/dev/null 2>&1 &
  sleep 15
  echo "firefox-$label"
  ./sample-tree-linux.sh "$FPROF" 2 "firefox-${label}-v3" >/dev/null
}

run_load "blank" "about:blank"
for t in 1 10 30; do
  args=""; for ((i=0;i<t;i++)); do args="$args $URL"; done
  run_load "${t}tabs" $args
done
args=""; for s in $SITES; do for _ in 1 2 3; do args="$args https://$s"; done; done
run_load "crosssite" $args

pkill -f "$FPROF" 2>/dev/null; sleep 2
rm -rf "$FPROF"
echo "firefox v3 done"
