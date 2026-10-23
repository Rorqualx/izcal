#!/bin/bash
# FP-M0 open question 17, the HTTP-cache part: each incumbent's HTTP-cache
# size against free disk (black-box). Fresh profiles, the cross-site load
# (30 tabs, 10 domains), ~45 s to settle so the network stack writes its
# cache, then the profile's cache directories are sized and the configured
# capacity prefs read where the product stores them. macOS first.
set -u
cd "$(dirname "$0")" || exit 1
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
FIREFOX="/Applications/Firefox.app/Contents/MacOS/firefox"
CPROF=$(mktemp -d /tmp/fp-m0-cache-c.XXXXXX)
FPROF=$(mktemp -d /tmp/fp-m0-cache-f.XXXXXX)
CPORT=9351
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"

echo "free disk before: $(df -k / | awk 'NR==2{print $4}') KB free"

# --- Chrome: one launch, tabs via /json/new, verified page count ---
"$CHROME" --user-data-dir="$CPROF" --remote-debugging-port="$CPORT" "about:blank" >/dev/null 2>&1 &
sleep 8
for s in $SITES; do for _ in 1 2 3; do curl -s -X PUT "http://127.0.0.1:$CPORT/json/new?https://$s" >/dev/null 2>&1; sleep 0.4; done; done
sleep 45
echo "chrome pages: $(curl -s "http://127.0.0.1:$CPORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(len([p for p in d if p.get("type")=="page"]))')"
pkill -f "$CPROF" 2>/dev/null; sleep 3
echo "--- chrome cache dirs (KB, entries):"
# Chrome's HTTP cache lives at Default/Cache/Cache_Data (the Cache dir
# also holds No_Vary_Search); the pre-review build checked only
# Default/Cache_Data, which does not exist, and silently measured nothing.
du -sk "$CPROF/Default/Cache" "$CPROF/Default/Cache/Cache_Data" "$CPROF/Default/Code Cache" "$CPROF/Default/GPUCache" 2>/dev/null
find "$CPROF"/Default/Cache -type f 2>/dev/null | wc -l | sed 's/^/Cache files: /'
grep -o '"disk_cache[^,]*' "$CPROF"/Default/Preferences 2>/dev/null | head -5

# --- Firefox: one invocation, N URLs (the v3 pattern) ---
args=""; for s in $SITES; do for _ in 1 2 3; do args="$args https://$s"; done; done
"$FIREFOX" -headless -profile "$FPROF" "about:blank" $args >/dev/null 2>&1 &
sleep 60
pkill -f "$FPROF" 2>/dev/null; sleep 3
echo "--- firefox cache dirs (KB, entries):"
du -sk "$FPROF"/cache2 "$FPROF"/cache2/entries 2>/dev/null
find "$FPROF"/cache2/entries -type f 2>/dev/null | wc -l | sed 's/^/cache2 entry files: /'
grep -E "browser.cache.disk.(capacity|smart_size)" "$FPROF"/prefs.js 2>/dev/null || echo "(no cache prefs written — defaults apply)"
du -sk "$CPROF" "$FPROF" | awk '{print "profile total: " $1 " KB  " $2}'

echo "free disk after: $(df -k / | awk 'NR==2{print $4}') KB free"
rm -rf "$CPROF" "$FPROF"
echo "cache probe done"
