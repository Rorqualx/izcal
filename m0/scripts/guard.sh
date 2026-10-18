#!/bin/bash
# Disk guard: samples free space every 20 s while $1 (pid) lives; kills the build
# if free space drops below 15 GB. Logs min free and size of $2 (dir).
pid=$1; dir=$2; log=$3; limit=$((15*1024*1024))
min=999999999; maxdu=0
while kill -0 $pid 2>/dev/null; do
  avail=$(df -k ~ | tail -1 | awk '{print $4}')
  du=$(du -sk "$dir" 2>/dev/null | awk '{print $1}')
  [ "$avail" -lt "$min" ] && min=$avail
  [ -n "$du" ] && [ "$du" -gt "$maxdu" ] && maxdu=$du
  echo "$(date +%T) avail_kb=$avail dir_kb=$du min_avail_kb=$min max_dir_kb=$maxdu" >> "$log"
  if [ "$avail" -lt "$limit" ]; then
    echo "$(date +%T) DISK GUARD TRIPPED: killing build" >> "$log"
    pkill -f ninja; pkill -P $pid; kill $pid
    break
  fi
  sleep 20
done
echo "FINAL min_avail_kb=$min max_dir_kb=$maxdu" >> "$log"
