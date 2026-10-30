#!/usr/bin/env bash
# VER-1 (Stage 1 4.1; security scope 6.6): runs every Kani harness in the
# IPC crates (hg-ipc, the generated hg-ipc-gen and hg-os-types), the
# launcher hand-off (hg-launch-wire), the spawner plan (hg-launch-plan)
# and the pure cores of the OS transports (hg-substrate-os `ipc::unix`
# and `ipc::win`). Fails on a failed proof, and on a `kani::cover!` that
# no input reaches, since that harness would prove nothing.
# Usage: tools/kani.sh [extra cargo-kani args]. Needs Kani 0.68.0
# (`cargo install --locked kani-verifier --version 0.68.0 && cargo kani setup`).
set -euo pipefail
cd "$(dirname "$0")/.."

# KANI_AS_LIMIT (bytes, optional; Linux): caps each run's address space
# with prlimit, as shared rigs need. CBMC's address space runs about
# 1 GiB above its resident memory (measured 25 Oct 2026, see
# measurements/milestones/m1-kani/), so 10737418240 (10 GiB) stops a
# run near 8.9 GiB resident. Raising it did not rescue the run measured
# at that cap: it ran out of time instead.
limit=()
if [[ -n "${KANI_AS_LIMIT:-}" ]]; then
  limit=(prlimit "--as=$KANI_AS_LIMIT")
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
status=0
for pkg in hg-ipc hg-ipc-gen hg-os-types hg-launch-wire hg-launch-plan hg-substrate-os; do
  echo "== cargo kani -p $pkg"
  # pipefail: the pipeline fails if cargo kani does.
  # -Z stubbing: the generated unknown-ordinal harnesses stub each
  # message's validator (see hg-ipc-codegen's emit.rs).
  if ! ${limit[@]+"${limit[@]}"} cargo kani -p "$pkg" -Z stubbing "$@" 2>&1 | grep -v "aborting path" >>"$log"; then
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
