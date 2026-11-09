#!/usr/bin/env bash
# JS-D4's spike: builds the harness, runs the validity gates, and records
# the byte accounting and the GC-time measurement with its rig evidence.
#
#   d4-run.sh <rig> <session> [repeats]
#
# <rig> names the results directory and the rig to check. Bytes-per-node
# is a memory measurement and needs no exclusive window; the GC timing
# does, so this script refuses to run it without a passing `rig check`
# and keeps that line beside the data.
set -euo pipefail

RIG=${1:?usage: d4-run.sh <rig> <session> [repeats]}
SESSION=${2:?usage: d4-run.sh <rig> <session> [repeats]}
REPEATS=${3:-15}

HERE=$(cd "$(dirname "$0")" && pwd)
OUT="$HERE/../results/$RIG"
mkdir -p "$OUT"
cd "$HERE"

echo "== building =="
cargo build --release

echo "== validity gates =="
# Every mode runs the gates itself and refuses to print a measurement if
# one fails; this call makes the failure visible here too.
./target/release/d4-nodes gates | tee "$OUT/gates.txt"

echo "== environment =="
{
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "rig: $RIG"
  echo "uname: $(uname -smr)"
  echo "rustc: $(rustc --version)"
  echo "harness commit: $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
  if [ -r /proc/cpuinfo ]; then
    echo "cpu: $(awk -F': ' '/model name/ {print $2; exit}' /proc/cpuinfo)"
    echo "cores: $(nproc)"
    echo "governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'n/a')"
  elif command -v sysctl >/dev/null; then
    echo "cpu: $(sysctl -n machdep.cpu.brand_string)"
    echo "cores: $(sysctl -n hw.ncpu) ($(sysctl -n hw.perflevel0.physicalcpu 2>/dev/null || echo '?') perf)"
  fi
} | tee "$OUT/env.txt"

echo "== bytes per node (no exclusive window needed) =="
./target/release/d4-nodes bytes > "$OUT/bytes.tsv"
wc -l < "$OUT/bytes.tsv"

echo "== rig check before the timing run =="
# The timing run is the only part that needs the window. `rig` lives on
# the controlling machine, so when this script runs on a remote rig the
# caller must pass the line it got there in RIG_CHECK. Either way a
# passing line is required: without one, no time is kept.
if command -v rig >/dev/null 2>&1; then
  if ! rig check "$RIG" --kind exclusive --session "$SESSION" > "$OUT/rig-check.txt" 2>&1; then
    cat "$OUT/rig-check.txt"
    echo "d4-run.sh: no exclusive window; the GC timing is not run and no time is kept" >&2
    exit 1
  fi
elif [ -n "${RIG_CHECK:-}" ] && [ -r "${RIG_CHECK}" ]; then
  cp "$RIG_CHECK" "$OUT/rig-check.txt"
  if ! grep -q '^rig check OK' "$OUT/rig-check.txt"; then
    cat "$OUT/rig-check.txt"
    echo "d4-run.sh: the supplied rig check did not pass; no time is kept" >&2
    exit 1
  fi
else
  echo "d4-run.sh: no rig command and no RIG_CHECK file; no time is kept" >&2
  exit 1
fi
cat "$OUT/rig-check.txt"

echo "== GC time, $REPEATS repeats =="
./target/release/d4-nodes gc "$REPEATS" > "$OUT/gc.tsv"
wc -l < "$OUT/gc.tsv"

if command -v rig >/dev/null 2>&1; then
  echo "== rig check after the timing run =="
  rig check "$RIG" --kind exclusive --session "$SESSION" > "$OUT/rig-check-after.txt" 2>&1 || true
  cat "$OUT/rig-check-after.txt"
fi

echo "== summary: node ns per node, by design, at the largest tree =="
awk -F'\t' 'NR==1 {next} $1=="large-500k" {k=$6"\t"$7"\t"$8; s[k]+=$15; n[k]++}
  END {for (i in s) printf "%s\t%.2f\n", i, s[i]/n[i]}' "$OUT/gc.tsv" | sort

echo "done: $OUT"
