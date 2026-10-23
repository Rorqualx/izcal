#!/bin/bash
# FP-M0 Safari leg (macOS, rig local). Safari's single-instance model
# means the measurement runs inside the user's live session, so the rule
# is: ONE new window, bound by reference at creation, every load driven
# inside it, and only that window closed at the end — existing windows,
# tabs and the app itself are never touched. Loads match the matrix:
# blank/1/10/30 same-site plus the 30-tab cross-site mix, 2-minute
# samples. Tab counts are verified via AppleScript and logged. The tree
# sampler's roots cover Safari plus its WebKit helpers (they are
# XPC-launched and need their own patterns); bracket forms keep the
# patterns from matching this script's own processes.
set -u
cd "$(dirname "$0")" || exit 1
HARNESS="$PWD"
LOG="/tmp/fp-m0-safari.log"
PATTERNS='Safari[.]app,com[.]apple[.]WebKit[.]WebContent,com[.]apple[.]WebKit[.]Networking,com[.]apple[.]WebKit[.]GPU'
SITES="example.com example.org iana.org wikipedia.org archive.org openstreetmap.org gutenberg.org ietf.org w3.org python.org"
: > "$LOG"
if ! pgrep -xq Safari; then open -a Safari; sleep 6; fi   # -600 guard

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
    set win to window 1   -- ours, bound by reference at creation
  end tell

  do shell script "echo 'safari blank sample' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-blank-v1 >/dev/null 2>&1 &"
  delay 135

  tell application "Safari" to tell win to make new tab with properties {URL:"https://example.com"}
  delay 8
  do shell script "echo 'safari 1-tab sample' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-1tabs-v1 >/dev/null 2>&1 &"
  delay 135

  tell application "Safari"
    repeat 9 times
      tell win to make new tab with properties {URL:"https://example.com"}
      delay 0.5
    end repeat
  end tell
  delay 8
  do shell script "echo 'safari 10-tab sample' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-10tabs-v1 >/dev/null 2>&1 &"
  delay 135

  tell application "Safari"
    repeat 20 times
      tell win to make new tab with properties {URL:"https://example.com"}
      delay 0.5
    end repeat
  end tell
  delay 8
  do shell script "echo 'safari 30-tab sample' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-30tabs-v1 >/dev/null 2>&1 &"
  delay 135

  tell application "Safari"
    close win
    set w to make new document with properties {URL:"about:blank"}
    set win to window 1
  end tell
  delay 3
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
  do shell script "echo 'safari cross-site sample, fresh window, tabs: " & tabCount & "' >> " & quoted form of logPath
  do shell script "cd " & quoted form of harnessPath & " && ./sample-tree.sh '" & patterns & "' 2 safari-crosssite-v2 >/dev/null 2>&1 &"
  delay 135

  tell application "Safari" to close win
  do shell script "echo 'safari window closed' >> " & quoted form of logPath
end run
EOF
OSAPID=$!
echo "safari matrix running (pid $OSAPID), log: $LOG"
wait $OSAPID
echo "--- log:"
cat "$LOG"
echo "safari matrix done"
