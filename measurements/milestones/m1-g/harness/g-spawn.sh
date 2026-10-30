#!/bin/bash
# M1 phase G: FP-27 spawn-to-ready on the full sandbox path (macOS, Linux).
# Usage: g-spawn.sh BUILD_DIR warm|fresh OUT_TSV [N]
#   warm:  one hg-launch-tool measure-spawn N (each run a new broker).
#   fresh: N runs, each from a fresh copy of the binaries in a new
#          directory (E-R12's "fresh build directory where reboot isn't
#          possible"); the copy's pages are in the page cache, so this is
#          not a cold-after-reboot run, and the README says so.
# TSV: run kind spawn_to_ready_ms (the substrate clock, request to the
# accepted `ready`).
set -u
BUILD=$1; MODE=$2; OUT=$3; N=${4:-20}
bins="hg-launch-tool hg-renderer-main hg-net-main"
[ -x "$BUILD/hg-spawner-main" ] && [ "$(uname)" = Linux ] && bins="$bins hg-spawner-main"
if [ "$MODE" = warm ]; then
  "$BUILD/hg-launch-tool" measure-spawn "$N" > "$OUT"
else
  printf 'run\tkind\tspawn_to_ready_ms\n' > "$OUT"
  for ((i = 1; i <= N; i++)); do
    d=$(mktemp -d)
    for b in $bins; do cp "$BUILD/$b" "$d/"; done
    "$d/hg-launch-tool" measure-spawn 1 | tail -n +2 | awk -v run=$i 'BEGIN{OFS="\t"} {$1=run; print}' >> "$OUT"
    rm -rf "$d"
  done
fi
awk -F'\t' 'NR>1 {v[$2]=v[$2]" "$3} END {for (k in v) {n=split(v[k],a," "); asort_n=n; for(i=1;i<=n;i++)for(j=i+1;j<=n;j++)if(a[j]+0<a[i]+0){t=a[i];a[i]=a[j];a[j]=t}; m=(n%2)?a[(n+1)/2]:(a[n/2]+a[n/2+1])/2; printf "%s n=%d median=%.3f min=%.3f max=%.3f\n", k, n, m, a[1], a[n]}}' "$OUT"
