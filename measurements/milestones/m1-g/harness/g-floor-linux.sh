#!/usr/bin/env bash
# M1 phase G, Linux: the measured floor for FP-2 under FP 6.3's rule
# (owner decision, 26 October 2026), with G's charged metric. FP-M0's
# Linux leg recorded 2,148 KB VmRSS; FP 6.2's Linux metric is
# RssAnon + VmSwap, so the floor is measured again with it: FP-M0's
# empty process (`fp-m0-empty` under `fp-m0-probe`, built at 46d1e6f,
# the Linux leg's build), sampled as g-run-linux.sh samples the renderer.
# Usage: g-floor-linux.sh FLOOR_DIR OUT_TSV [RUNS]
set -u
DIR=$1; OUT=$2; RUNS=${3:-5}
kb() { awk '/^RssAnon:/{a=$2} /^VmSwap:/{s=$2} END{print a+s}' "/proc/$1/status"; }
sw() { cat "/proc/$1"/task/*/status 2>/dev/null | awk '/^(voluntary|nonvoluntary)_ctxt_switches:/{s+=$2} END{print s}'; }
printf 'run\tkind\tpid\tsettled_kb\tpeak_kb\tswitches_per_s\tthreads\tvmas\n' > "$OUT"
for run in $(seq 1 "$RUNS"); do
  log=$(mktemp)
  "$DIR/fp-m0-probe" > "$log" 2>&1 &
  sup=$!
  for _ in $(seq 200); do grep -q 'child pid=' "$log" && break; sleep 0.1; done
  pid=$(pgrep -P "$sup" -f fp-m0-empty)
  [ -n "$pid" ] || { echo "run $run: no child" >&2; cat "$log" >&2; kill "$sup"; exit 1; }
  s0=$(sw "$pid"); mem=""
  for _ in $(seq 60); do mem="$mem $(kb "$pid")"; sleep 1; done
  s1=$(sw "$pid")
  # shellcheck disable=SC2086 # one sample per word
  settled=$(printf '%s\n' $mem | tail -n 30 | sort -n | awk '{a[NR]=$1} END {print (NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2}')
  # shellcheck disable=SC2086
  peak=$(printf '%s\n' $mem | sort -n | tail -1)
  rate=$(awk -v a="$s0" -v b="$s1" 'BEGIN{printf "%.3f", (b-a)/60}')
  threads=$(find "/proc/$pid/task" -mindepth 1 -maxdepth 1 | wc -l)
  vmas=$(wc -l < "/proc/$pid/maps")
  printf '%s\tfloor\t%s\t%s\t%s\t%s\t%s\t%s\n' "$run" "$pid" "$settled" "$peak" "$rate" "$threads" "$vmas" >> "$OUT"
  kill -TERM "$sup"; wait "$sup"
  rm -f "$log"
done
cat "$OUT"
