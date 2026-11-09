#!/usr/bin/env bash
# Runs the JS-D16 measurement and writes one directory of raw results.
#
#   d16-run.sh <rig-label> <out-dir> "<rig check line>" [instrs] [entries] [reps]
#
# `rig` runs only on the Mac that books the rigs, so the check line is
# captured there and passed in here: it is the evidence that the rig was
# booked exclusive for this run. The script refuses to run without one.
#
# Nothing here changes a system setting. The governor and the clock are
# read for the record only.
set -euo pipefail

rig=${1:?rig label}
out=${2:?output directory}
check=${3:?the rig check line}
instrs=${4:-4000000}
entries=${5:-2000000}
reps=${6:-7}

here=$(cd "$(dirname "$0")" && pwd)
cd "$here"
mkdir -p "$out"
printf '%s\n' "$check" > "$out/rig-check.txt"
case "$check" in
  *"rig check OK"*) ;;
  *) echo "d16-run.sh: that is not a passing rig check line; refusing to run" >&2; exit 2 ;;
esac

{
  echo "rig: $rig"
  echo "date: $(date +%Y-%m-%dT%H:%M:%S%z)"
  echo "uname: $(uname -srm)"
  if [ -r /proc/cpuinfo ]; then
    echo "cpu: $(awk -F': ' '/model name/ {print $2; exit}' /proc/cpuinfo)"
    echo "cores: $(nproc)"
    echo "governor: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
    echo "mhz: $(awk -F': ' '/cpu MHz/ {print $2}' /proc/cpuinfo | sort -n | sed -n '1p;$p' | paste -sd- -)"
  else
    echo "cpu: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
    echo "cores: $(sysctl -n hw.ncpu 2>/dev/null || echo unknown)"
  fi
  echo "rustc: $(rustc --version)"
  echo "commit: $(git -C "$here" rev-parse HEAD 2>/dev/null || echo "${IZCAL_COMMIT:-unknown}")"
  echo "load: $(cat /proc/loadavg 2>/dev/null || uptime)"
} > "$out/env.txt"

cargo build --release 2>&1 | tail -3
# The resolved dependency versions, for the record (not a manifest: this
# crate ships nothing and keeps no lockfile in the repository).
cp Cargo.lock "$out/cargo-lock.txt" 2>/dev/null || true
bin=./target/release/d16-trusted

# The gate first: a failed gate must stop the run, not decorate it.
if ! "$bin" gate > "$out/gate.txt" 2>&1; then
  echo "d16-run.sh: the validity gate FAILED; see $out/gate.txt" >&2
  cat "$out/gate.txt" >&2
  exit 1
fi

"$bin" dispatch "$instrs" "$reps" > "$out/dispatch.tsv"
"$bin" entry "$entries" "$reps" > "$out/entry.tsv"
"$bin" e2e "$reps" > "$out/e2e.tsv"
"$bin" window "$reps" > "$out/window.tsv"

# The sequences the note quotes, from the binary that produced the rows
# above. The four `seq_` functions are kept out of line for this.
{
  "$bin" seq
  echo
  if command -v objdump >/dev/null; then
    objdump -d --no-show-raw-insn "$bin" 2>/dev/null |
      awk '/^[0-9a-f]+ <.*seq_(trusted_code|caged_code|trusted_table|caged_frame).*>:/ {p=1} p; p && /^$/ {p=0}'
  else
    echo "objdump not installed: no disassembly on this host"
  fi
} > "$out/sequences.txt"

echo "d16-run.sh: wrote $out"
