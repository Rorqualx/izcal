#!/bin/bash
# Run fetch_once.sh N times, INTERVAL seconds apart (background job).
S="$(cd "$(dirname "$0")" && pwd)"
FEEDS_ROOT="$1"; N="${2:-4}"; INTERVAL="${3:-3600}"
for i in $(seq 1 "$N"); do
  bash "$S/fetch_once.sh" "$FEEDS_ROOT"
  [ "$i" -lt "$N" ] && sleep "$INTERVAL"
done
echo DONE
