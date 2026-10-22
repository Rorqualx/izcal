#!/bin/bash
# FP-M0 footprint sampler, tree-sum mode.
# Usage: sample-tree.sh <comma-separated-pgrep-patterns> <minutes> <label>
# Sums phys_footprint across every process in the trees rooted at the
# pattern matches and writes a dated TSV into ../results/. Portability:
# bash 3.2-compatible (no `local -A`), no seq(1) — C-style loops only.
set -u

patterns="$1"; minutes="${2:-2}"; label="${3:-manual}"
out="../results/$(date +%Y-%m-%d)-${label}.tsv"

# Collect the process tree: roots match the pattern, then walk descendants
# breadth-first (each pid has one parent, so no cycles); duplicates from
# helpers that match the pattern AND sit under a matching parent collapse
# in the final sort -u.
tree_pids() {
  local roots; roots=$(pgrep -f "$patterns" | sort -u | grep -v "^$$$")
  local queue; queue="$roots"; local out=""
  while [[ -n "$queue" ]]; do
    local next="" pid
    for pid in $queue; do
      out="$out $pid"
      local kids; kids=$(pgrep -P "$pid" | sort -u)
      next="$next $kids"
    done
    queue="$next"
  done
  printf '%s\n' "$out" | tr ' ' '\n' | sed '/^$/d' | sort -un
}

printf 't\tsample\tnprocs\ttotal_phys_MB\ttotal_peak_MB\tmax_single_MB\n' > "$out"
n=$(awk -v m="$minutes" 'BEGIN{printf "%d", m*12}')
for ((i = 0; i <= n; i++)); do
  pids=$(tree_pids)
  total_phys=0; total_peak=0; max_single=0; count=0
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    raw=$(footprint "$pid" 2>/dev/null | grep -E "phys_footprint" | tr -s ' ')
    # Normalize to MB with the unit the CLI printed (it prints KB/B for
    # small processes; a bare number dropped the unit and inflated rows).
    phys=$(printf '%s\n' "$raw" | grep -oE 'phys_footprint: [0-9.]+ ?[KMG]?B?' | awk '{v=$2; u=$3; if(u=="KB") v/=1024; else if(u=="B") v/=1048576; else if(u=="GB") v*=1024; printf "%.0f", v}')
    peak=$(printf '%s\n' "$raw" | grep -oE 'phys_footprint_peak: [0-9.]+ ?[KMG]?B?' | awk '{v=$2; u=$3; if(u=="KB") v/=1024; else if(u=="B") v/=1048576; else if(u=="GB") v*=1024; printf "%.0f", v}')
    [[ -z "$phys" ]] && continue
    count=$((count + 1)); total_phys=$((total_phys + phys))
    total_peak=$((total_peak + peak))
    [[ "$phys" -gt "$max_single" ]] && max_single=$phys
  done <<< "$pids"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$(date +%H:%M:%S)" "$i" "$count" "$total_phys" "$total_peak" "$max_single" >> "$out"
  sleep 5
done
echo "done: $(wc -l < "$out") rows ($count processes in the last sample)"
