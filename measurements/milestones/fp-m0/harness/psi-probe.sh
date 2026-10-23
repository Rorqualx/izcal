#!/bin/bash
# FP-M0 open question 17, the Linux PSI part: does the incumbent react to
# memory pressure before the OOM killer? Black-box on hueyj: 10-tab
# chromium (snap, headless), baseline, then a staircase of anonymous
# allocations (8 -> 12 -> 16 GiB, ~60 s each, bail-out before a step if
# memory is critically low) while the browser tree total and
# /proc/pressure/memory are sampled every 5 s, then release and recovery.
#
# 22 Oct run note: the original single 16 GiB shot was too aggressive —
# hueyj's sshd wedged during the pressure window and the rig went down;
# chromium was SIGKILLed mid-run. The staircase above is the recorded
# redesign; the 22 Oct data died with the rig's /tmp on reboot.
set -u
cd "$(dirname "$0")" || exit 1
CPROF=$(mktemp -d /tmp/fp-m0-psi.XXXXXX)
PORT=9361
out="$HOME/fp-m0-psi/$(date +%Y-%m-%d)-psi-probe.tsv"
mkdir -p "$HOME/fp-m0-psi"   # $HOME survives a reboot; /tmp did not on 22 Oct

tree_pids() {
  local roots; roots=$(pgrep -f "$1" | sort -u | grep -v "^$$$")
  local queue; queue="$roots"; local acc=""
  while [[ -n "$queue" ]]; do
    local next="" pid
    for pid in $queue; do
      acc="$acc $pid"
      local kids; kids=$(pgrep -P "$pid" | sort -u)
      next="$next $kids"
    done
    queue="$next"
  done
  printf '%s\n' "$acc" | tr ' ' '\n' | sed '/^$/d' | sort -un
}
total_mb() {
  local total=0 count=0 pid rss swap kb
  for pid in $(tree_pids "$CPROF"); do
    rss=$(awk '/VmRSS/{print $2}' "/proc/$pid/status" 2>/dev/null)
    swap=$(awk '/VmSwap/{print $2}' "/proc/$pid/status" 2>/dev/null)
    [[ -z "$rss" ]] && continue
    kb=$((rss + ${swap:-0})); total=$((total + kb / 1024)); count=$((count + 1))
  done
  echo "$total $count"
}
psi() { awk -F'[ =]' '/^some/{print $3, $5}' /proc/pressure/memory 2>/dev/null; }  # some avg10, some avg60 (fields: some avg10=V avg60=V avg300=V total=V)
avail() { awk '/MemAvailable/{printf "%d", $2/1024}' /proc/meminfo; }      # MB

chromium --headless=new --no-sandbox --user-data-dir="$CPROF" --remote-debugging-port="$PORT" about:blank >/dev/null 2>&1 &
sleep 10
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -s -X PUT "http://127.0.0.1:$PORT/json/new?https://example.com" >/dev/null 2>&1; sleep 0.4
done
sleep 10
echo "pages: $(curl -s "http://127.0.0.1:$PORT/json" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(len([p for p in d if p.get("type")=="page"]))')"

printf 't\tphase\tbrowser_total_MB\tnprocs\tpsi_some_avg10\tpsi_some_avg60\tmemavail_MB\n' > "$out"
row() {  # $1 = phase label
  local phase="$1" tm t c p av
  tm=$(total_mb); set -- $tm
  t=$1; c=$2
  p=$(psi); av=$(avail)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(date +%H:%M:%S)" "$phase" "$t" "$c" $p "$av" >> "$out"
}

row baseline
for ((i = 0; i < 6; i++)); do sleep 5; row baseline; done   # 30 s baseline

# Staircase pressure: 8 -> 12 -> 16 GiB, ~60 s each, with a bail-out
# before a step if memory is already critically low (the 22 Oct single
# 16 GiB shot took the rig down; this is the recorded redesign).
for GB in 8 12 16; do
  if (( $(avail) < 1024 )); then row "bailout-${GB}g"; break; fi
  python3 - "$GB" <<'EOF' &
import sys, time
gb = int(sys.argv[1])
a = bytearray(gb * 1024**3)
step = 4096
a[::step] = b"\x01" * (gb * 1024**3 // step)  # touch every page, C-speed
time.sleep(60)
EOF
  ALLOCPID=$!
  sleep 2
  for ((i = 0; i < 12; i++)); do sleep 5; row "pressure-${GB}g"; done  # 60 s hold
  wait $ALLOCPID 2>/dev/null
done
for ((i = 0; i < 12; i++)); do sleep 5; row recovery; done  # 60 s recovery

pkill -f "$CPROF" 2>/dev/null; sleep 2
rm -rf "$CPROF"
echo "psi probe done: $(wc -l < "$out") rows -> $out"
