#!/bin/bash
# FP-M0 Linux incumbent matrix: chromium and firefox, fresh profiles,
# blank/1/10/30 same-site plus a 30-tab cross-site mix, 2-minute samples.
# hueyj notes: chromium runs headless (no display); firefox needs
# -headless too, and a $HOME profile — the firefox snap cannot see host
# /tmp ("Could not find profile folder" on a valid /tmp dir).
set -u
cd "$(dirname "$0")" || exit 1
CHROME=chromium; FIREFOX=firefox
URL=https://example.com
CPROF=$(mktemp -d /tmp/fp-m0-lin-chrome.XXXXXX); FPROF=$(mktemp -d ~/fp-m0-lin-ff.XXXXXX)
CPORT=9343
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"

"$CHROME" --headless=new --no-sandbox --user-data-dir="$CPROF" --remote-debugging-port="$CPORT" "about:blank" >/dev/null 2>&1 &
sleep 10
pages() { curl -s "http://127.0.0.1:$CPORT/json" 2>/dev/null | python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: print("NA"); raise SystemExit
print(len([p for p in d if p.get("type")=="page"]))'; }
tabs() { local want="$1" i; for ((i=0;i<want;i++)); do curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?$URL" >/dev/null 2>&1; sleep 0.4; done; sleep 6; }
echo "chromium-blank"; ./sample-tree-linux.sh "$CPROF" 2 "chromium-blank-v1" >/dev/null
for t in 1 10 30; do tabs "$t"; echo "chromium-$t: $(pages) pages"; ./sample-tree-linux.sh "$CPROF" 2 "chromium-${t}tabs-v1" >/dev/null; done
# cross-site: open 30 across 10 domains
for s in $SITES; do for _ in 1 2 3; do curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?https://$s" >/dev/null 2>&1; sleep 0.4; done; done
sleep 8; echo "chromium-cross: $(pages) pages"; ./sample-tree-linux.sh "$CPROF" 2 "chromium-crosssite-v1" >/dev/null
pkill -f "$CPROF" 2>/dev/null; sleep 2

"$FIREFOX" -headless -profile "$FPROF" "about:blank" >/dev/null 2>&1 &
sleep 12
fftabs() { local want="$1" i; for ((i=0;i<want;i++)); do "$FIREFOX" -headless -profile "$FPROF" "$URL" >/dev/null 2>&1; sleep 1.2; done; sleep 8; }
echo "firefox-blank"; ./sample-tree-linux.sh "$FPROF" 2 "firefox-blank-v1" >/dev/null
for t in 1 10 30; do fftabs "$t"; echo "firefox-$t requested"; ./sample-tree-linux.sh "$FPROF" 2 "firefox-${t}tabs-v1" >/dev/null; done
for s in $SITES; do for _ in 1 2 3; do "$FIREFOX" -headless -profile "$FPROF" "https://$s" >/dev/null 2>&1; sleep 1.2; done; done
sleep 10; ./sample-tree-linux.sh "$FPROF" 2 "firefox-crosssite-v1" >/dev/null
pkill -f "$FPROF" 2>/dev/null; sleep 2
rm -rf "$CPROF" "$FPROF"
echo "linux matrix done"
