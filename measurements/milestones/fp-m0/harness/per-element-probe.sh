#!/bin/bash
# FP-M0 open question 17, the per-element part: memory charged per element
# when a page creates a million of them (black-box). Serves one page that
# appends 1,000,000 <div>s (with a one-character text node each) from
# localhost, loads it in Chrome and Firefox (fresh profiles), and samples
# each browser's tree on the blank page and on the million-element page.
# Per-element cost = (million - blank) / 1e6. 1-minute samples.
set -u
cd "$(dirname "$0")" || exit 1
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
FIREFOX="/Applications/Firefox.app/Contents/MacOS/firefox"
WORK=$(mktemp -d /tmp/fp-m0-mill.XXXXXX)
CPROF=$(mktemp -d /tmp/fp-m0-mill-c.XXXXXX)
FPROF=$(mktemp -d /tmp/fp-m0-mill-f.XXXXXX)
PORT=9380
CPORT=9355

cat > "$WORK/blank.html" <<'EOF'
<!doctype html><meta charset="utf-8"><title>blank</title><body></body>
EOF
cat > "$WORK/million.html" <<'EOF'
<!doctype html><meta charset="utf-8"><title>million</title>
<body><script>
const N = 1000000;
const f = document.createDocumentFragment();
for (let i = 0; i < N; i++) { const d = document.createElement("div"); d.textContent = "x"; f.appendChild(d); }
document.body.appendChild(f);
document.title = "million done";
</script></body>
EOF
lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port $PORT busy; a previous run may have survived" >&2; exit 1; }
lsof -nP -iTCP:"$CPORT" -sTCP:LISTEN >/dev/null 2>&1 && { echo "port $CPORT busy; a previous run may have survived" >&2; exit 1; }
python3 -m http.server "$PORT" --directory "$WORK" >/dev/null 2>&1 &
SRVPID=$!
sleep 2

echo "--- chrome"
"$CHROME" --user-data-dir="$CPROF" --remote-debugging-port="$CPORT" "http://127.0.0.1:$PORT/blank" >/dev/null 2>&1 &
sleep 8
echo "chrome blank"; ./sample-tree.sh "$CPROF" 1 "chrome-million-blank-v1" >/dev/null
curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?http://127.0.0.1:$PORT/million.html" >/dev/null 2>&1
sleep 5
# Close the blank tab so the million sample measures the million page alone.
BLANK_ID=$(curl -s "http://127.0.0.1:$CPORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(next((p["id"] for p in d if p.get("type")=="page" and p.get("title")=="blank"), ""))')
[[ -n "$BLANK_ID" ]] && curl -s "http://127.0.0.1:$CPORT/json/close/$BLANK_ID" >/dev/null 2>&1
sleep 25
echo "chrome million: $(curl -s "http://127.0.0.1:$CPORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print([p.get("title") for p in d if p.get("type")=="page"])')"
curl -s "http://127.0.0.1:$CPORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
titles=[p.get("title") for p in d if p.get("type")=="page"]
sys.exit(0 if any(t=="million done" for t in titles) else 1)' || { echo "WARNING: million page not done; sample would be contaminated" >&2; exit 1; }
./sample-tree.sh "$CPROF" 1 "chrome-million-1m-v1" >/dev/null
pkill -f "$CPROF" 2>/dev/null; sleep 2

echo "--- firefox"
"$FIREFOX" -headless -profile "$FPROF" "http://127.0.0.1:$PORT/blank" >/dev/null 2>&1 &
sleep 10
echo "firefox blank"; ./sample-tree.sh "$FPROF" 1 "firefox-million-blank-v1" >/dev/null
# The blank instance holds the profile lock; a second -headless invocation
# against it would die without loading (the Linux v2 lesson). Kill first.
pkill -f "$FPROF" 2>/dev/null; sleep 3
"$FIREFOX" -headless -profile "$FPROF" "http://127.0.0.1:$PORT/million.html" >/dev/null 2>&1 &
sleep 30
./sample-tree.sh "$FPROF" 1 "firefox-million-1m-v1" >/dev/null
pkill -f "$FPROF" 2>/dev/null; sleep 2

kill $SRVPID 2>/dev/null
rm -rf "$WORK" "$CPROF" "$FPROF"
echo "per-element probe done"
