#!/usr/bin/env bash
# The remote file names are chosen on the Mac and meant to expand there.
# shellcheck disable=SC2029
# FP-27 cold spawn-to-ready on hueyj (E-Q6, owner's decision 26 Oct):
# N runs, each after dropping the page cache, of one broker launching both
# kinds through the spawner. The only sudo call is the cache drop.
#
# Run by the owner, interactively, in a terminal on the Mac (where `rig`
# and the bookings live):
#
#   measurements/milestones/m1-e/fp27-cold.sh [N]
#
# It checks the exclusive booking (the coordinator books hueyj exclusive
# under m1-substrate first; this script holds none and stops if the check
# fails). It then copies itself to hueyj and runs the loop there in one
# ssh session with a terminal. Nothing is piped on the Mac side, so
# sudo's password prompt reaches the terminal (a pipe through `tr` held
# the prompt back on 29 Oct and the run waited unseen). sudo asks once,
# up front (`sudo -v`), and the loop keeps the ticket fresh. The loop
# writes its TSV to a file on hueyj, which is then appended to the log,
# measurements/milestones/m1-g/fp27-cold-<date>.log (M1 phase G), after
# the rig check line. N defaults to 20.
#
# The tree on hueyj is ~/izcal-m1int-g (m1/integration) unless
# IZCAL_TREE names another directory under ~. Its release binaries must
# be built first:
#   cd ~/izcal-m1int-g && cargo build --release --workspace --bins
#
# `--remote N TREE OUT` is the loop itself, run on hueyj by the Mac side.
set -euo pipefail

if [ "${1:-}" = "--remote" ]; then
  N=$2
  TREE=$3
  OUT=$4
  BIN=$HOME/$TREE/target/release
  for exe in hg-launch-tool hg-spawner-main hg-renderer-main hg-net-main; do
    [ -x "$BIN/$exe" ] || { echo "fp27-cold: $exe is not built in $BIN" >&2; exit 2; }
  done
  echo "fp27-cold: sudo is needed only to drop the page cache before each run."
  sudo -v
  printf 'run\tkind\tspawn_to_ready_ms\n' > "$OUT"
  for i in $(seq 1 "$N"); do
    sudo -n sh -c 'sync; echo 3 > /proc/sys/vm/drop_caches'
    # One broker per run; the run number is ours.
    "$BIN/hg-launch-tool" measure-spawn 1 | tail -n +2 \
      | awk -v run="$i" 'BEGIN{OFS="\t"} {$1=run; print}' >> "$OUT"
    echo "fp27-cold: run $i of $N done"
  done
  exit 0
fi

N=${1:-20}
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
TREE=${IZCAL_TREE:-izcal-m1int-g}
STAMP=$(date +%Y%m%dT%H%M%S)
LOG=$HERE/../m1-g/fp27-cold-$(date +%Y-%m-%d).log
REMOTE_SCRIPT=.izcal-fp27-cold-$STAMP.sh
REMOTE_OUT=.izcal-fp27-cold-$STAMP.tsv

[ -t 0 ] || { echo "fp27-cold: run this in a terminal (sudo asks for a password)" >&2; exit 1; }

# The exclusive window must be ours (rig README): exit 0 means go.
if ! check=$(rig check hueyj --kind exclusive --session m1-substrate); then
  echo "fp27-cold: rig check refused: $check" >&2
  exit 1
fi

scp -q "${BASH_SOURCE[0]}" "hueyj:$REMOTE_SCRIPT"
if ssh -t hueyj "bash ~/$REMOTE_SCRIPT --remote $N $TREE ~/$REMOTE_OUT"; then
  {
    echo "# fp27-cold $(date -Iseconds) N=$N host=hueyj tree=$TREE"
    echo "# $check"
    ssh hueyj "cat ~/$REMOTE_OUT"
  } >> "$LOG"
  ssh hueyj "rm -f ~/$REMOTE_SCRIPT ~/$REMOTE_OUT"
  echo "fp27-cold: $N runs appended to $LOG"
else
  rc=$?
  ssh hueyj "rm -f ~/$REMOTE_SCRIPT"
  echo "fp27-cold: the run on hueyj failed (exit $rc); partial TSV, if any: hueyj:~/$REMOTE_OUT" >&2
  exit "$rc"
fi
