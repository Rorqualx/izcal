#!/bin/bash
# FP-M0 macOS incumbent matrix: Chrome and Firefox, dedicated fresh profiles,
# four loads each (blank, 1, 10, 30 tabs of example.com), 2-minute tree-sum
# samples. Chrome tabs are created through its DevTools HTTP endpoint
# (/json/new), which opens real renderer-backed targets in this headless
# launch context; Firefox opens tabs by direct invocation (verified: the
# process tree grows with load). Portability: bash 3.2-compatible, no
# seq(1). Tab creation is verified for Chrome (its DevTools /json page
# count); Firefox's page count is checked via /json/list when that endpoint
# answers, and its rows carry the process-growth-evidence caveat in the
# decision log until a working Firefox debugger flag is found. Safari is queued
# separately.
set -u
cd "$(dirname "$0")" || exit 1

CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
FIREFOX="/Applications/Firefox.app/Contents/MacOS/firefox"
URL="https://example.com"
CHROME_PROF="$(mktemp -d /tmp/fp-m0-chrome.XXXXXX)"
FF_PROF="$(mktemp -d /tmp/fp-m0-firefox.XXXXXX)"
CPORT=9337

# --- Chrome: one launch, tabs via /json/new, verified by page count ---
lsof -nP -iTCP:"$CPORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port $CPORT busy; a previous run may have survived" >&2; exit 1; }
"$CHROME" --user-data-dir="$CHROME_PROF" --remote-debugging-port="$CPORT" "about:blank" >/dev/null 2>&1 &
sleep 8
chrome_pages() {
  curl -s "http://127.0.0.1:$CPORT/json" 2>/dev/null | python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: print(0); raise SystemExit
print(len([p for p in d if p.get("type")=="page"]))'
}
chrome_tabs() {  # $1 = number of example.com tabs to open; verifies the count
  local want="$1" i
  for ((i = 0; i < want; i++)); do
    curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?$URL" >/dev/null 2>&1
    sleep 0.4
  done
  sleep 6
  local got; got=$(chrome_pages)
  echo "  chrome: $got pages (want >= $want)"
  [[ "$got" -lt "$want" ]] && { echo "  WARNING: fewer pages than requested" >&2; }
}
echo "chrome-blank"; ./sample-tree.sh "$CHROME_PROF" 2 "chrome-blank-v2" >/dev/null
for tabs in 1 10 30; do
  chrome_tabs "$tabs"
  ./sample-tree.sh "$CHROME_PROF" 2 "chrome-${tabs}tabs-v2" >/dev/null
done
pkill -f "$CHROME_PROF" 2>/dev/null; sleep 2

# --- Firefox: direct invocation opens real windows and tabs; page count
# verified through its remote debugging endpoint ---
FPORT=9338
lsof -nP -iTCP:"$FPORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port $FPORT busy; a previous run may have survived" >&2; exit 1; }
"$FIREFOX" -profile "$FF_PROF" -remote-debugging-port "$FPORT" "about:blank" >/dev/null 2>&1 &
sleep 10
ff_pages() {
  curl -s "http://127.0.0.1:$FPORT/json/list" 2>/dev/null | python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: print(0); raise SystemExit
print(len([p for p in d if p.get("type")=="page"]))'
}
ff_tabs() {  # $1 = number of example.com tabs to open; verifies the count
  local want="$1" i
  for ((i = 0; i < want; i++)); do
    "$FIREFOX" -profile "$FF_PROF" -remote-debugging-port "$FPORT" "$URL" >/dev/null 2>&1
    sleep 1.2
  done
  sleep 8
  local got; got=$(ff_pages)
  echo "  firefox: $got pages (want >= $want)"
  [[ "$got" -lt "$want" ]] && { echo "  WARNING: fewer pages than requested" >&2; }
}
echo "firefox-blank"; ./sample-tree.sh "$FF_PROF" 2 "firefox-blank-v2" >/dev/null
for tabs in 1 10 30; do
  ff_tabs "$tabs"
  ./sample-tree.sh "$FF_PROF" 2 "firefox-${tabs}tabs-v2" >/dev/null
done
pkill -f "$FF_PROF" 2>/dev/null; sleep 2
rm -rf "$CHROME_PROF" "$FF_PROF"
echo "matrix v3 done"
