#!/bin/bash
# FP-M0 tree sampler, Linux flavour: sums VmRSS+VmSwap (anonymous resident
# plus swap, the counter the spec names for Linux) across the process tree
# rooted at the pattern matches. bash-3.2-portable, no seq.
set -u
patterns="$1"; minutes="${2:-2}"; label="${3:-manual}"
out="/tmp/fp-m0-linux/results/$(date +%Y-%m-%d)-${label}.tsv"
mkdir -p /tmp/fp-m0-linux/results

tree_pids() {
  local roots; roots=$(pgrep -f "$patterns" | sort -u | grep -v "^$$$")
  local queue; queue="$roots"; local acc=""
  while [[ -n "$queue" ]]; do
    local next="" pid
    for pid in $queue; do
      acc="$acc $pid"
      local kids; kids=$(pgrep -P "$pid" | sort -u)
      next="$next $kids"
    done
    queue="$next"
  done
  printf '%s\n' "$acc" | tr ' ' '\n' | sed '/^$/d' | sort -un
}

printf 't\tsample\tnprocs\ttotal_MB\tmax_single_MB\n' > "$out"
n=$(awk -v m="$minutes" 'BEGIN{printf "%d", m*12}')
for ((i = 0; i <= n; i++)); do
  pids=$(tree_pids)
  total=0; max_single=0; count=0
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    rss=$(awk '/VmRSS/{print $2}' "/proc/$pid/status" 2>/dev/null)
    swap=$(awk '/VmSwap/{print $2}' "/proc/$pid/status" 2>/dev/null)
    [[ -z "$rss" ]] && continue
    kb=$((rss + ${swap:-0}))
    mb=$((kb / 1024))
    count=$((count + 1)); total=$((total + mb))
    [[ "$mb" -gt "$max_single" ]] && max_single=$mb
  done <<< "$pids"
  printf '%s\t%s\t%s\t%s\t%s\n' "$(date +%H:%M:%S)" "$i" "$count" "$total" "$max_single" >> "$out"
  sleep 5
done
echo "done: $(wc -l < "$out") rows ($count processes in the last sample)"
