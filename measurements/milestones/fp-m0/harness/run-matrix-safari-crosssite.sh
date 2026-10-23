#!/bin/bash
# FP-M0 Safari cross-site leg, fresh window: 30 tabs across 10 domains
# (three each), 2-minute sample, window closed at the end. The full
# driver accumulates tabs in one window, which made its cross-site
# sample a 61-tab window; this run matches the matrix's cross-site load.
set -u
cd "$(dirname "$0")" || exit 1
HARNESS="$PWD"
LOG="/tmp/fp-m0-safari-cs.log"
PATTERNS='Safari[.]app,com[.]apple[.]WebKit[.]WebContent,com[.]apple[.]WebKit[.]Networking,com[.]apple[.]WebKit[.]GPU'
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"
: > "$LOG"
if ! pgrep -xq Safari; then open -a Safari; sleep 6; fi

osascript - "$HARNESS" "$LOG" "$PATTERNS" "$SITES" <<'EOF' &
on run argv
  set harnessPath to item 1 of argv
  set logPath to item 2 of argv
  set patterns to item 3 of argv
  set siteList to item 4 of argv

  set sites to {}
  set oldDelims to AppleScript's text item delimiters
  set AppleScript's text item delimiters to " "
  set sites to every text item of siteList
  set AppleScript's text item delimiters to oldDelims

  tell application "Safari"
    set w to make new document with properties {URL:"about:blank"}
    set win to window 1
  end tell
  tell application "Safari"
    repeat with s in sites
      repeat 3 times
        tell win to make new tab with properties {URL:"https://" & s}
        delay 0.5
      end repeat
    end repeat
  end tell
  delay 10
  set tabCount to 0
  tell application "Safari" to set tabCount to count of tabs of win
  do shell script "echo 'safari cross-site fresh window, tabs: " & tabCount & "' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-crosssite-v2 >/dev/null 2>&1 &"
  delay 135
  tell application "Safari" to close win
  do shell script "echo 'safari cross-site window closed' >> " & quoted form of logPath
end run
EOF
OSAPID=$!
echo "safari cross-site running (pid $OSAPID), log: $LOG"
wait $OSAPID
echo "--- log:"
cat "$LOG"
echo "safari cross-site done"
