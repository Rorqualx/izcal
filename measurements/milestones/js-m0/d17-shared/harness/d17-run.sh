#!/usr/bin/env bash
# The JS-D17 spike's run script. It takes the rig-check evidence line as
# its first argument, because `rig` runs only on the Mac that books the
# rigs: a remote leg is given the line that was captured there, and the
# line is written into the results beside the data it authorises.
#
#   d17-run.sh "<rig check line>" <out-dir> [legs...]
#
# Legs: gate space space-max reservations churn-maps maps contend interleave
#       xheap  (the C micro-benchmark, which needs cc)
# With no legs named it runs them all.
#
# It never changes a system setting: no governor, no sysctl, no affinity.
#
# D17_COMMIT_CAP_MIB caps what the space-max leg may commit (default
# 3072) and D17_RES_LIMIT caps the reservations leg (default 20000), so a
# leg can be run on a machine that is sharing its memory with another
# session.
set -u

CHECK=${1:?usage: d17-run.sh "<rig check line>" <out-dir> [legs...]}
OUT=${2:?usage: d17-run.sh "<rig check line>" <out-dir> [legs...]}
shift 2
LEGS=${*:-gate space space-max reservations churn-maps maps contend interleave xheap}

HERE=$(cd "$(dirname "$0")" && pwd)
BIN="$HERE/d17-heaps/target/release/d17-heaps"
mkdir -p "$OUT"
printf '%s\n' "$CHECK" >> "$OUT/rig-check.txt"

# The environment the numbers belong to.
{
  # BSD date has no -Is; this form works on both.
  echo "date: $(date +%Y-%m-%dT%H:%M:%S%z)"
  echo "host: $(uname -n)"
  echo "kernel: $(uname -srm)"
  if [ -r /proc/cpuinfo ]; then
    echo "cpu: $(awk -F': ' '/^model name/ {print $2; exit}' /proc/cpuinfo)"
    echo "cores: $(nproc)"
    echo "governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
    echo "max_map_count: $(cat /proc/sys/vm/max_map_count 2>/dev/null || echo unknown)"
    echo "mem_total_kb: $(awk '/^MemTotal/ {print $2}' /proc/meminfo)"
    echo "overcommit_memory: $(cat /proc/sys/vm/overcommit_memory 2>/dev/null || echo unknown)"
  else
    echo "cpu: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
    echo "cores: $(sysctl -n hw.ncpu 2>/dev/null || echo unknown)"
    echo "mem_total_bytes: $(sysctl -n hw.memsize 2>/dev/null || echo unknown)"
    echo "pagesize: $(sysctl -n hw.pagesize 2>/dev/null || echo unknown)"
  fi
  echo "rustc: $(rustc --version 2>/dev/null || echo unknown)"
  echo "harness: $BIN"
} > "$OUT/env.txt"

run() { # run <file> <args...>
  local f=$1; shift
  echo "== $f"
  "$BIN" "$@" > "$OUT/$f" 2> "$OUT/${f%.tsv}.err"
  local rc=$?
  [ -s "$OUT/${f%.tsv}.err" ] || rm -f "$OUT/${f%.tsv}.err"
  return $rc
}

for leg in $LEGS; do
  case "$leg" in
    gate)
      echo "== gate.txt"
      if "$BIN" gate > "$OUT/gate.txt" 2>&1; then
        echo "   gate PASS"
      else
        echo "   gate FAILED — no timing below is kept" >&2
        cat "$OUT/gate.txt" >&2
        exit 1
      fi
      ;;
    space)        run space.tsv space --agents 1,2,4,8,16,32,64,128,256 --mib-per-agent 2 ;;
    space-max)    run space-max.tsv space-max --limit 4096 --commit-cap-mib "${D17_COMMIT_CAP_MIB:-3072}" ;;
    reservations) run reservations.tsv reservations --limit "${D17_RES_LIMIT:-20000}" --gib 16 ;;
    churn-maps)   run churn-maps.tsv churn-maps --rounds 2048 --blocks 40 --live 4 ;;
    maps)         run maps.txt maps --agents 8 --churn 64 ;;
    contend)
      # Cells long enough that thread start-up is a small part of the
      # window: the overlap gate on the shorter cells of the first run
      # failed, and a failed gate is not a measurement.
      run contend-grow.tsv contend --mode grow --agents 1,2,4,8 --n 2000000 --reps 3
      run contend-steady.tsv contend --mode steady --agents 1,2,4,8 --n 8000000 --reps 3
      run contend-churn.tsv contend --mode churn --agents 1,2,4,8 --n 800 --reps 3
      ;;
    interleave)   run interleave.tsv interleave --others 0,1,3,7 --cells 400000 --laps 20 --reps 3 ;;
    xheap)
      if command -v cc >/dev/null; then
        cc -O2 -o "$OUT/d17-xheap" "$HERE/d17-xheap.c" 2> "$OUT/xheap-build.err"
        if [ -x "$OUT/d17-xheap" ]; then
          [ -s "$OUT/xheap-build.err" ] || rm -f "$OUT/xheap-build.err"
          "$OUT/d17-xheap" > "$OUT/xheap.tsv" 2> "$OUT/xheap.err"
          [ -s "$OUT/xheap.err" ] || rm -f "$OUT/xheap.err"
          # The emitted sequences, which is the check that each variant is
          # the one the spec names.
          if command -v objdump >/dev/null; then
            objdump -d "$OUT/d17-xheap" > "$OUT/xheap-objdump.txt" 2>/dev/null
          elif command -v otool >/dev/null; then
            otool -tv "$OUT/d17-xheap" > "$OUT/xheap-objdump.txt" 2>/dev/null
          fi
          rm -f "$OUT/d17-xheap"
        else
          echo "   xheap: cc failed" >&2
        fi
      else
        echo "   xheap: no cc" >&2
      fi
      ;;
    *) echo "unknown leg: $leg" >&2; exit 2 ;;
  esac
done

echo "done -> $OUT"
