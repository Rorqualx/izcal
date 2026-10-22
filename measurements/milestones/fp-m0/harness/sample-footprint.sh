#!/bin/bash
# FP-M0 footprint sampler (macOS flavour).
# Usage: sample-footprint.sh <app-name|pid> <url-or-empty> <minutes> [label]
# Samples the target's process tree every 5 s with the macOS `footprint` CLI
# and writes a dated TSV into ../results/. Incumbents are black-box signals.
set -u

target="$1"; url="${2:-}"; minutes="${3:-1}"; label="${4:-manual}"
out="../results/$(date +%Y-%m-%d)-${label}.tsv"

# Open the target with an empty page; a PID is accepted as-is.
if [[ "$target" =~ ^[0-9]+$ ]]; then
  pid="$target"
else
  if [[ -n "$url" ]]; then open -a "$target" "$url"; else open -a "$target"; fi
  sleep 6   # first paint + idle settle
  pid=$(pgrep -f "^/Applications/${target}\.app/Contents/MacOS" | head -1 || pgrep -x "$target" | head -1)
fi
[[ -z "${pid:-}" ]] && { echo "no process found for $target" >&2; exit 1; }

echo "sampling pid $pid for ${minutes} min -> $out"
echo -e "t\tsample\tpid\tphys_footprint\tfootprint\tnote" > "$out"
n=$((minutes * 12))
for i in $(seq 0 "$n"); do
  line=$(footprint "$pid" 2>/dev/null | grep -E "phys_footprint|footprint" | tr -s ' ' | paste -sd' ' -)
  echo -e "$(date +%H:%M:%S)\t$i\t$pid\t$line" >> "$out"
  sleep 5
done
echo "done: $(wc -l < "$out") rows"
