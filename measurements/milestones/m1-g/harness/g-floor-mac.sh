#!/bin/bash
# M1 phase G, macOS: the measured floor for FP-2 under FP 6.3's rule
# (FP-2 = the measured floor plus Izcal's own line items; owner decision,
# 26 October 2026). The floor is FP-M0's empty process (`fp-m0-empty`
# under its supervisor `fp-m0-probe`, built at 25f7591, the FP-M0
# decision log's build), sampled exactly as g-run-mac.sh samples the
# renderer: phys_footprint at 1 Hz over 60 s, settled = median of the
# last 30 s; RUNS runs.
# Usage: g-floor-mac.sh FLOOR_DIR PROBE OUT_TSV [RUNS]
#   FLOOR_DIR: target/release of a 25f7591 build with fp-m0-probe and
#   fp-m0-empty; PROBE: g-probe-mac. bash 3.2.
set -u
DIR=$1; PROBE=$2; OUT=$3; RUNS=${4:-5}
printf 'run\tkind\tpid\tsettled_kb\tpeak_kb\twakeups_per_s\tpkg_idle_per_s\tregions\n' > "$OUT"
for ((run = 1; run <= RUNS; run++)); do
  log=$(mktemp)
  "$DIR/fp-m0-probe" > "$log" 2>&1 &
  sup=$!
  for ((i = 0; i < 200; i++)); do grep -q 'sandbox=applied' "$log" && break; sleep 0.1; done
  grep -q 'sandbox=applied' "$log" || { echo "run $run: no ready" >&2; cat "$log" >&2; kill "$sup"; exit 1; }
  pid=$(pgrep -P "$sup" -x fp-m0-empty)
  s0=$($PROBE "$pid")
  : > "$log.s"
  for ((t = 0; t < 60; t++)); do
    $PROBE "$pid" | awk '{print $1}' >> "$log.s"
    sleep 1
  done
  s1=$($PROBE "$pid")
  settled=$(tail -n 30 "$log.s" | sort -n | awk '{a[NR]=$1} END {m=(NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2; printf "%.0f", m/1024}')
  peak=$(sort -n "$log.s" | tail -1 | awk '{printf "%.0f", $1/1024}')
  w=$(echo "$s0 $s1" | awk '{printf "%.3f", ($6-$2)/60}')
  p=$(echo "$s0 $s1" | awk '{printf "%.3f", ($7-$3)/60}')
  reg=$(echo "$s1" | awk '{print $4}')
  printf "%s\tfloor\t%s\t%s\t%s\t%s\t%s\t%s\n" "$run" "$pid" "$settled" "$peak" "$w" "$p" "$reg" >> "$OUT"
  kill -TERM "$sup"; wait "$sup"
  rm -f "$log" "$log.s"
done
cat "$OUT"
