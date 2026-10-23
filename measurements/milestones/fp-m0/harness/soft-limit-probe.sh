#!/bin/bash
# FP-M0 open question 17, the soft-process-limit part: Chrome's soft
# process limit on this lab device, black-box. Opens same-site tabs one
# at a time through the DevTools endpoint and records the process-tree
# size every 5 tabs; the run stops at 80 tabs or once the tree size is
# unchanged across three consecutive checks (the plateau).
set -u
cd "$(dirname "$0")" || exit 1
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
CPROF=$(mktemp -d /tmp/fp-m0-soft.XXXXXX)
CPORT=9356
URL="https://example.com"

lsof -nP -iTCP:"$CPORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port $CPORT busy; a previous run may have survived" >&2; exit 1; }
"$CHROME" --user-data-dir="$CPROF" --remote-debugging-port="$CPORT" "about:blank" >/dev/null 2>&1 &
sleep 8

nprocs() { pgrep -f "$CPROF" | wc -l | tr -d ' '; }
pages() { curl -s "http://127.0.0.1:$CPORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(len([p for p in d if p.get("type")=="page"]))'; }

printf 'tabs\tnprocs\tpages\n'
prev=-1; plateau=0
for ((t = 1; t <= 80; t++)); do
  curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?$URL" >/dev/null 2>&1
  sleep 1.5
  if (( t % 5 == 0 )); then
    n=$(nprocs); p=$(pages)
    printf '%s\t%s\t%s\n' "$t" "$n" "$p"
    if [[ "$n" = "$prev" ]]; then plateau=$((plateau + 1)); else plateau=0; fi
    prev=$n
    (( plateau >= 3 )) && { echo "plateau at $t tabs ($n procs)"; break; }
  fi
done
pkill -f "$CPROF" 2>/dev/null; sleep 2
rm -rf "$CPROF"
echo "soft-limit probe done"
