#!/usr/bin/env bash
# VER-1 (Stage 1 4.1; security scope 6.6): runs every Kani harness in the
# IPC crates and the pure core of the OS transport (hg-substrate-os
# `ipc::unix`). Fails on a failed proof, and on a `kani::cover!` that no
# input reaches, since that harness would prove nothing.
# Usage: tools/kani.sh [extra cargo-kani args]. Needs Kani 0.68.0
# (`cargo install --locked kani-verifier --version 0.68.0 && cargo kani setup`).
set -euo pipefail
cd "$(dirname "$0")/.."

log=$(mktemp)
trap 'rm -f "$log"' EXIT
status=0
for pkg in hg-ipc hg-ipc-gen hg-substrate-os; do
  echo "== cargo kani -p $pkg"
  # pipefail: the pipeline fails if cargo kani does.
  if ! cargo kani -p "$pkg" "$@" 2>&1 | grep -v "aborting path" >>"$log"; then
    status=1
  fi
done
grep -E "^Checking harness|^VERIFICATION|Verification Time|^Complete|^Summary|harnesses" "$log" || true

if grep -q "Status: UNSATISFIABLE" "$log"; then
  echo "error: a cover property is unreachable (a vacuous harness):" >&2
  grep -B3 "Status: UNSATISFIABLE" "$log" >&2
  status=1
fi
if grep -q "VERIFICATION:- FAILED" "$log"; then
  echo "error: a proof failed" >&2
  status=1
fi
exit "$status"
