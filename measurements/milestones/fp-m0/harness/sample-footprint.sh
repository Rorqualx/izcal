#!/bin/bash
# FP-M0 footprint sampler (macOS flavour).
# Usage: sample-footprint.sh <app-name|pid> <url-or-empty> <minutes> [label]
# Samples the target process every 5 s with the macOS `footprint` CLI and
# writes a dated TSV into ../results/. One process per run (main process
# sampling); the tree-sum mode is queued in the milestone README.
# Incumbents are black-box signals.
set -u

target="$1"; url="${2:-}"; minutes="${3:-1}"; label="${4:-manual}"
out="../results/$(date +%Y-%m-%d)-${label}.tsv"

# Open the target with an empty page; a PID is accepted as-is.
if [[ "$target" =~ ^[0-9]+$ ]]; then
  pid="$target"
else
  if [[ -n "$url" ]]; then open -a "$target" "$url"; else open -a "$target"; fi
  sleep 6   # first paint + idle settle
  pid=$(pgrep -f "^/Applications/${target}\.app/Contents/MacOS" | head -1)
  if [[ -z "$pid" ]]; then
    pid=$(pgrep -x "$target" | head -1)
  fi
fi
[[ -z "${pid:-}" ]] && { echo "no process found for $target" >&2; exit 1; }

echo "sampling pid $pid for ${minutes} min -> $out"
printf 't\tsample\tpid\tphys_footprint_MB\tpeak_MB\n' > "$out"
n=$(awk -v m="$minutes" 'BEGIN{printf "%d", m*12}')
for i in $(seq 0 "$n"); do
  raw=$(footprint "$pid" 2>/dev/null | grep -E "phys_footprint" | tr -s ' ')
  phys=$(printf '%s\n' "$raw" | grep -oE 'phys_footprint: [0-9.]+ ?[KMG]?B?' | awk '{print $2}')
  peak=$(printf '%s\n' "$raw" | grep -oE 'phys_footprint_peak: [0-9.]+ ?[KMG]?B?' | awk '{print $2}')
  printf '%s\t%s\t%s\t%s\t%s\n' "$(date +%H:%M:%S)" "$i" "$pid" "${phys:-NA}" "${peak:-NA}" >> "$out"
  sleep 5
done
echo "done: $(wc -l < "$out") rows"
