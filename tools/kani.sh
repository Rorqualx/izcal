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

# On a shared Linux host (not CI), each package runs in its own systemd
# user scope with a hard memory cap and no swap, so a harness that
# outgrows it is killed alone by the kernel, before it presses on the
# owner's other processes. On Huey the Destroyer (hueyj) every process of the user shares
# user-1000.slice, with MemoryHigh 9 GiB and MemoryMax 11 GiB (set by the
# owner, 24 Sep 2026; raised to 16 GiB and 18 GiB on 30 Oct). A CBMC run
# past 9 GiB once pressed that slice until systemd-oomd killed the user's
# login manager, and new logins failed until a reboot (30 Oct 2026).
# The cap is KANI_MEM_MAX (a systemd size such as 7G) if set, else the
# user slice's memory.high less 1.5 GiB for the user's services, else
# none. CI's runner has no such slice and runs uncapped.
scope=()
mem_cap=""
if [[ -z "${CI:-}" && "$(uname -s)" == Linux ]] && command -v systemd-run >/dev/null 2>&1; then
  mem_cap=${KANI_MEM_MAX:-}
  if [[ -z "$mem_cap" ]]; then
    high_file=/sys/fs/cgroup/user.slice/user-$(id -u).slice/memory.high
    high=$(cat "$high_file" 2>/dev/null || echo max)
    if [[ "$high" =~ ^[0-9]+$ ]]; then
      mem_cap=$(( high - 1536 * 1024 * 1024 ))
    fi
  fi
  if [[ -n "$mem_cap" ]]; then
    if systemd-run --user --scope --quiet -p MemoryMax="$mem_cap" -p MemorySwapMax=0 true 2>/dev/null; then
      scope=(systemd-run --user --scope --quiet -p "MemoryMax=$mem_cap" -p MemorySwapMax=0)
      echo "kani.sh: each package capped at MemoryMax=$mem_cap (no swap)"
    else
      echo "kani.sh: cannot make a memory-capped user scope here; refusing to run uncapped on a shared host (set CI=1 to override)" >&2
      exit 2
    fi
  fi
fi

log=$(mktemp)
trap 'rm -f "$log" "${pkg_log:-}"' EXIT
status=0
for pkg in hg-ipc hg-ipc-gen hg-os-types hg-launch-wire hg-launch-plan hg-substrate-os; do
  echo "== cargo kani -p $pkg"
  # pipefail: the pipeline fails if cargo kani does.
  # -Z stubbing: the generated unknown-ordinal harnesses stub each
  # message's validator (see hg-ipc-codegen's emit.rs).
  pkg_log=$(mktemp)
  if ! ${scope[@]+"${scope[@]}"} ${limit[@]+"${limit[@]}"} cargo kani -p "$pkg" -Z stubbing "$@" 2>&1 | grep -v "aborting path" >"$pkg_log"; then
    status=1
    # A package that fails without a failed proof (a build error, or a
    # harness killed at its memory cap) would otherwise print nothing.
    if ! grep -q "VERIFICATION:- FAILED" "$pkg_log"; then
      echo "error: cargo kani -p $pkg failed without a failed proof${mem_cap:+ (memory cap $mem_cap)}; its last lines:" >&2
      tail -n 30 "$pkg_log" >&2
    fi
  fi
  cat "$pkg_log" >>"$log"
  rm -f "$pkg_log"
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
