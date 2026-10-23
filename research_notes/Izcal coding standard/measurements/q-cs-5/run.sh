#!/bin/bash
# Q-CS-5: run the probe natively and under Valgrind Callgrind on revire.
# Usage: run.sh <outdir>. Needs env.sh (toolchain, RUSTY_V8_ARCHIVE from the
# fork's CI artifact for aabd96b, Valgrind extracted from Ubuntu's .deb).
# Writes one TSV row per run: workload, flags, aslr, rep, mode, Ir, wall_s, rc.
set -u
cd "$(dirname "$0")" || exit 1
# shellcheck disable=SC1091
. ./env.sh
OUT=${1:?outdir}
mkdir -p "$OUT/cg"
B=probe/target/release/qcs5-probe
WORKLOADS="init script10 gc"
FLAGSETS=("" "--single-threaded" "--jitless" "--single-threaded --jitless" "--predictable" "--predictable --jitless")

one() { # workload flags aslr rep mode
  local w=$1 f=$2 aslr=$3 rep=$4 mode=$5 tag start end rc ir pre=()
  tag="$w-${f// /_}-$aslr-$rep-$mode"
  [ "$aslr" = off ] && pre=(setarch -R)
  start=$(date +%s.%N)
  if [ "$mode" = native ]; then
    # shellcheck disable=SC2086
    env -i "${pre[@]}" "$B" "$w" $f > "$OUT/cg/$tag.out" 2>&1; rc=$?
    ir=-
  else
    # gungraun's defaults: environment cleared, ASLR off; Ir only.
    # shellcheck disable=SC2086
    env -i VALGRIND_LIB="$VALGRIND_LIB" "${pre[@]}" "$VG" --tool=callgrind \
      --callgrind-out-file="$OUT/cg/$tag.cg" "$B" "$w" $f > "$OUT/cg/$tag.out" 2>&1; rc=$?
    ir=$(sed -n 's/^summary: //p' "$OUT/cg/$tag.cg" 2>/dev/null)
  fi
  end=$(date +%s.%N)
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%.3f\t%s\n' "$w" "${f:--}" "$aslr" "$rep" "$mode" "${ir:--}" \
    "$(echo "$end - $start" | bc)" "$rc"
}
export -f one
export B OUT VG VALGRIND_LIB

{
  # Ir repeatability: 5 runs per config with ASLR off, 3 with ASLR on, 6 at a time.
  for w in $WORKLOADS; do for f in "${FLAGSETS[@]}"; do
    for r in 1 2 3 4 5; do printf '%s\0%s\0off\0%s\0vg\0' "$w" "$f" "$r"; done
    for r in 1 2 3; do printf '%s\0%s\0on\0%s\0vg\0' "$w" "$f" "$r"; done
  done; done | xargs -0 -n5 -P6 bash -c 'one "$@"' _
} > "$OUT/ir.tsv"

{
  # Slowdown: one at a time, 3 native and 3 Valgrind runs per config.
  for w in $WORKLOADS; do for f in "${FLAGSETS[@]}"; do
    for r in 1 2 3; do one "$w" "$f" off "$r" native; one "$w" "$f" off "$r" vgseq; done
  done; done
} > "$OUT/wall.tsv"
