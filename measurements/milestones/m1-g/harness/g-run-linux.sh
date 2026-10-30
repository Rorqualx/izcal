#!/usr/bin/env bash
# M1 phase G, Linux: the renderer-skeleton row (and the network
# partition service beside it), as g-run-mac.sh does on macOS.
# - Charged memory: RssAnon + VmSwap (FP 6.2's Linux metric; no Izcal
#   shared region is mapped at rest), 1 Hz over 60 s from `ready`;
#   settled is the median of the last 30 s; 5 runs.
# - Idle wakeups: voluntary + involuntary context switches summed over
#   /proc/PID/task/*/status, FP-M0's wakeups-linux.sh method.
# - Mappings: lines of /proc/PID/maps (one VMA each), FP 6.20.6.
# The children are the spawner's (E-R2): found as the tool's spawner's
# children, by their command line (`comm` is cut to 15 characters).
# Usage: g-run-linux.sh BUILD_DIR OUT_TSV [RUNS]
set -u
BUILD=$1; OUT=$2; RUNS=${3:-5}
kb() { awk '/^RssAnon:/{a=$2} /^VmSwap:/{s=$2} END{print a+s}' "/proc/$1/status"; }
sw() { cat "/proc/$1"/task/*/status 2>/dev/null | awk '/^(voluntary|nonvoluntary)_ctxt_switches:/{s+=$2} END{print s}'; }
printf 'run\tkind\tpid\tsettled_kb\tpeak_kb\tswitches_per_s\tthreads\tvmas\n' > "$OUT"
for run in $(seq 1 "$RUNS"); do
  log=$(mktemp)
  "$BUILD/hg-launch-tool" hold 75 > "$log" 2>&1 &
  tool=$!
  for _ in $(seq 200); do grep -q '^hold: ready' "$log" && break; sleep 0.1; done
  grep -q '^hold: ready' "$log" || { echo "run $run: no ready" >&2; cat "$log" >&2; kill "$tool"; exit 1; }
  spawner=$(pgrep -P "$tool" -f hg-spawner-main)
  declare -A pid=([renderer]=$(pgrep -P "$spawner" -f 'hg-renderer-main') [net]=$(pgrep -P "$spawner" -f 'hg-net-main'))
  declare -A s0 mem
  for k in renderer net; do s0[$k]=$(sw "${pid[$k]}"); mem[$k]=""; done
  for _ in $(seq 60); do
    for k in renderer net; do mem[$k]="${mem[$k]} $(kb "${pid[$k]}")"; done
    sleep 1
  done
  for k in renderer net; do
    p=${pid[$k]}
    s1=$(sw "$p")
    settled=$(printf '%s\n' ${mem[$k]} | tail -n 30 | sort -n | awk '{a[NR]=$1} END {print (NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2}')
    peak=$(printf '%s\n' ${mem[$k]} | sort -n | tail -1)
    rate=$(awk -v a="${s0[$k]}" -v b="$s1" 'BEGIN{printf "%.3f", (b-a)/60}')
    threads=$(find "/proc/$p/task" -mindepth 1 -maxdepth 1 | wc -l)
    vmas=$(wc -l < "/proc/$p/maps")
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$run" "$k" "$p" "$settled" "$peak" "$rate" "$threads" "$vmas" >> "$OUT"
  done
  wait "$tool"
  rm -f "$log"
done
cat "$OUT"
