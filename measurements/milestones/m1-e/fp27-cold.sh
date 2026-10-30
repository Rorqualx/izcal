#!/usr/bin/env bash
# FP-27 cold spawn-to-ready on hueyj (E-Q6, owner's decision 26 Oct):
# N runs, each after dropping the page cache, of one broker launching both
# kinds through the spawner. The only sudo call is the cache drop.
#
# Run by the owner, interactively, on the Mac (where `rig` and the
# bookings live):
#
#   measurements/milestones/m1-e/fp27-cold.sh [N]
#
# It checks the exclusive booking (the coordinator books hueyj exclusive
# under m1-substrate first; this script holds none and stops if the check
# fails), then runs the loop on hueyj in one ssh session with a terminal,
# so sudo asks for the password once. N defaults to 20. The log,
# measurements/milestones/m1-e/fp27-cold-<date>.log, gets the rig check
# line, then a TSV (run, kind, spawn_to_ready_ms).
#
# On hueyj the release binaries must be built first, in ~/izcal-m1e2:
#   cargo build --release --workspace --bins
#
# `--remote N` is the loop itself, run on hueyj by the Mac side.
set -euo pipefail

if [ "${1:-}" = "--remote" ]; then
  N=$2
  TOOL=$HOME/izcal-m1e2/target/release/hg-launch-tool
  for exe in hg-launch-tool hg-spawner-main hg-renderer-main hg-net-main; do
    [ -x "$(dirname "$TOOL")/$exe" ] || { echo "fp27-cold: $exe is not built" >&2; exit 2; }
  done
  printf 'run\tkind\tspawn_to_ready_ms\n'
  for i in $(seq 1 "$N"); do
    sudo sh -c 'sync; echo 3 > /proc/sys/vm/drop_caches'
    # One broker per run; the run number is ours.
    "$TOOL" measure-spawn 1 | tail -n +2 | awk -v run="$i" 'BEGIN{OFS="\t"} {$1=run; print}'
  done
  exit 0
fi

N=${1:-20}
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
LOG=$HERE/fp27-cold-$(date +%Y-%m-%d).log

# The exclusive window must be ours (rig README): exit 0 means go.
if ! check=$(rig check hueyj --kind exclusive --session m1-substrate); then
  echo "fp27-cold: rig check refused: $check" >&2
  exit 1
fi

{
  echo "# fp27-cold $(date -Iseconds) N=$N host=hueyj"
  echo "# $check"
} >> "$LOG"
# The terminal goes to sudo's prompt; the TSV comes back on stdout. Under
# ssh -t the remote stderr arrives on stdout too, so everything is shown
# here as it comes (errors included) and only TSV lines reach the log.
ssh -t hueyj "bash ~/izcal-m1e2/measurements/milestones/m1-e/fp27-cold.sh --remote $N" \
  | tr -d '\r' | tee /dev/stderr | grep -E '^(run|[0-9]+)\b' >> "$LOG"
echo "fp27-cold: $N runs appended to $LOG"
