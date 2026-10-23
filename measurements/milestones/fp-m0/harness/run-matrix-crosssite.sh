#!/bin/bash
# FP-M0 cross-site mix: 30 tabs across 10 distinct domains, Chrome and
# Firefox, disposable profiles, 2-minute tree samples. Same harness rules
# as run-matrix.sh (bash 3.2, verified Chrome page counts, unit-normalized
# sampler).
set -u
cd "$(dirname "$0")" || exit 1
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
FIREFOX="/Applications/Firefox.app/Contents/MacOS/firefox"
CPROF="$(mktemp -d /tmp/fp-m0-cs-chrome.XXXXXX)"; FPROF="$(mktemp -d /tmp/fp-m0-cs-ff.XXXXXX)"
CPORT=9341
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"

lsof -nP -iTCP:"$CPORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port busy" >&2; exit 1; }
"$CHROME" --user-data-dir="$CPROF" --remote-debugging-port="$CPORT" "about:blank" >/dev/null 2>&1 &
sleep 8
for s in $SITES; do
  for _ in 1 2 3; do
    curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?https://$s" >/dev/null 2>&1
    sleep 0.4
  done
done
sleep 10
pages=$(curl -s "http://127.0.0.1:$CPORT/json" 2>/dev/null | python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: print("NA"); raise SystemExit
print(len([p for p in d if p.get("type")=="page"]))')
echo "chrome cross-site: $pages pages (want 30)"
if [[ "$pages" = "NA" || "$pages" -lt 25 ]]; then
  echo "WARNING: chrome page count unverified ($pages); not sampling" >&2
  pkill -f "$CPROF" 2>/dev/null
  exit 1
fi
./sample-tree.sh "$CPROF" 2 "chrome-crosssite-v1" >/dev/null
pkill -f "$CPROF" 2>/dev/null; sleep 2

"$FIREFOX" -profile "$FPROF" "about:blank" >/dev/null 2>&1 &
sleep 10
for s in $SITES; do
  for _ in 1 2 3; do
    "$FIREFOX" -profile "$FPROF" "https://$s" >/dev/null 2>&1
    sleep 1.2
  done
done
sleep 10
./sample-tree.sh "$FPROF" 2 "firefox-crosssite-v1" >/dev/null
pkill -f "$FPROF" 2>/dev/null; sleep 2
rm -rf "$CPROF" "$FPROF"
echo "cross-site matrix done"
