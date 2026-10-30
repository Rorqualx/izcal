#!/bin/bash
# M1 phase G, macOS: the renderer-skeleton row (and the network
# partition service beside it). FP 6.2: charged memory settled = the
# median of the last 30 s of a 60 s quiet period, median of 5 runs;
# FP-18 idle wakeups over the same 60 s; FP 6.20.6 mapping count.
# Usage: g-run-mac.sh BUILD_DIR PROBE OUT_TSV [RUNS]
#   BUILD_DIR: target/release with hg-launch-tool and the mains
#   PROBE: g-probe-mac (built from g-probe-mac.c)
# One row per run and kind: run kind pid settled_kb peak_kb
#   wakeups_per_s pkg_idle_per_s regions. bash 3.2, as FP-M0's harness.
set -u
BUILD=$1; PROBE=$2; OUT=$3; RUNS=${4:-5}
printf 'run\tkind\tpid\tsettled_kb\tpeak_kb\twakeups_per_s\tpkg_idle_per_s\tregions\n' > "$OUT"
for ((run = 1; run <= RUNS; run++)); do
  log=$(mktemp)
  "$BUILD/hg-launch-tool" hold 75 > "$log" 2>&1 &
  tool=$!
  for ((i = 0; i < 200; i++)); do grep -q '^hold: ready' "$log" && break; sleep 0.1; done
  grep -q '^hold: ready' "$log" || { echo "run $run: no ready" >&2; cat "$log" >&2; kill $tool; exit 1; }
  r=$(pgrep -P $tool -x hg-renderer-main); n=$(pgrep -P $tool -x hg-net-main)
  s0_renderer=$($PROBE $r); s0_net=$($PROBE $n)
  : > "$log.r"; : > "$log.n"
  for ((t = 0; t < 60; t++)); do
    $PROBE $r | awk '{print $1}' >> "$log.r"
    $PROBE $n | awk '{print $1}' >> "$log.n"
    sleep 1
  done
  for kind in renderer net; do
    if [ $kind = renderer ]; then pid=$r; f=$log.r; s0=$s0_renderer; else pid=$n; f=$log.n; s0=$s0_net; fi
    s1=$($PROBE $pid)
    settled=$(tail -n 30 "$f" | sort -n | awk '{a[NR]=$1} END {m=(NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2; printf "%.0f", m/1024}')
    peak=$(sort -n "$f" | tail -1 | awk '{printf "%.0f", $1/1024}')
    w=$(echo "$s0 $s1" | awk '{printf "%.3f", ($6-$2)/60}')
    p=$(echo "$s0 $s1" | awk '{printf "%.3f", ($7-$3)/60}')
    reg=$(echo "$s1" | awk '{print $4}')
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' $run $kind $pid $settled $peak $w $p $reg >> "$OUT"
  done
  wait $tool
  rm -f "$log" "$log.r" "$log.n"
done
cat "$OUT"
