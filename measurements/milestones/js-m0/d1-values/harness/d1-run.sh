#!/bin/sh
# JS-D1's spike on this machine: build the chase, record the instruction
# sequences the compiler emitted, and sweep the working sets. A timing
# run needs an exclusive booking, so the `rig check` line is taken first
# and kept beside the data (the plan's rule for timing runs).
#
#   d1-run.sh <rig-name> <out-dir> [cache-sizes-as-node-counts]
#
# The node counts should put the 32-bit encodings' footprint at this
# machine's L1, L2, L3 and beyond; the caller passes them because only
# the caller knows the machine.
set -eu

rig=${1:?usage: d1-run.sh RIG OUTDIR [NODES]}
out=${2:?usage: d1-run.sh RIG OUTDIR [NODES]}
nodes=${3:-1024,8192,131072,1048576,8388608}
here=$(dirname "$0")
mkdir -p "$out"

# the bytes list mirrors the node list, so equal-bytes compares the same
# footprints: 8 bytes a node under (a) and (c)
bytes=$(echo "$nodes" | tr ',' '\n' | awk '{printf "%s%d", (NR>1?",":""), $1*8}')

cc -O2 -Wall -Wextra -o "$out/d1-chase" "$here/d1-chase.c"

{
    echo "# JS-D1 spike, $(date -u +%Y-%m-%dT%H:%M:%SZ), rig=$rig"
    echo "# uname: $(uname -srm)"
    if [ -r /proc/cpuinfo ]; then
        echo "# cpu: $(awk -F': ' '/model name/{print $2; exit}' /proc/cpuinfo)"
        echo "# caches: $(lscpu 2>/dev/null | awk -F': +' '/^L[123]/{printf "%s=%s ", $1, $2}')"
    else
        echo "# cpu: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
        echo "# caches: l1d=$(sysctl -n hw.l1dcachesize 2>/dev/null) l2=$(sysctl -n hw.l2cachesize 2>/dev/null) line=$(sysctl -n hw.cachelinesize 2>/dev/null)"
    fi
    echo "# nodes: $nodes"
    echo "# bytes: $bytes"
} > "$out/env.txt"

# the emitted sequences, so a reader can check each encoding is the one
# Section 6.1 names and not something the compiler rewrote
if command -v objdump > /dev/null 2>&1; then
    dis="objdump -d"
elif command -v otool > /dev/null 2>&1; then
    dis="otool -tv"
else
    dis=""
fi
if [ -n "$dis" ]; then
    # only the three walk loops are kept: the rest of the dump is build
    # noise, and these are what a reader checks against Section 6.1
    full=$(mktemp)
    $dis "$out/d1-chase" > "$full" 2>/dev/null || true
    awk '/_?chase_scaled32>?:/,/ret/' "$full" > "$out/sequences.txt" || true
    awk '/_?chase_raw64>?:/,/ret/' "$full" >> "$out/sequences.txt" || true
    awk '/_?chase_byte32>?:/,/ret/' "$full" >> "$out/sequences.txt" || true
    rm -f "$full"
fi

# Evidence: the booking must be live and the machine quiet before timing.
# `rig` runs on the Mac that holds the bookings, not on every rig, so a
# remote leg is given its check line in IZCAL_RIG_CHECK by the caller
# who ran `rig check` there. Without a line, the run refuses: a timing
# run with no evidence is not a measurement.
if command -v rig > /dev/null 2>&1; then
    rig check "$rig" --kind exclusive --session js-m0 > "$out/rig-check.txt" 2>&1 || {
        echo "rig check refused; not running the timings" >&2
        cat "$out/rig-check.txt" >&2
        exit 1
    }
elif [ -n "${IZCAL_RIG_CHECK:-}" ]; then
    printf '%s\n' "$IZCAL_RIG_CHECK" > "$out/rig-check.txt"
else
    echo "no rig check: pass IZCAL_RIG_CHECK from the booking host" >&2
    exit 1
fi
cat "$out/rig-check.txt"

"$out/d1-chase" --nodes "$nodes" --reps 16 > "$out/equal-nodes.tsv"
"$out/d1-chase" --bytes "$bytes" --reps 16 > "$out/equal-bytes.tsv"

if command -v rig > /dev/null 2>&1; then
    rig check "$rig" --kind exclusive --session js-m0 >> "$out/rig-check.txt" 2>&1 || true
fi
echo "wrote $out/equal-nodes.tsv and $out/equal-bytes.tsv"
