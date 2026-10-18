#!/bin/bash
# usage: monitor.sh <logfile> <pidfile-of-build>
# Gates on headroom (excluding reclaimable page cache: current - file) under the uid-1000 slice cap (memory.high, 9 GiB), not on MemAvailable.
# Pauses our clang jobs if headroom < 2.5 GiB, resumes when > 4 GiB.
log=$1; bpid=$(cat $2); paused=0; peak=0
S=/sys/fs/cgroup/user.slice/user-1000.slice
while kill -0 $bpid 2>/dev/null; do
  high=$(cat $S/memory.high); [ "$high" = max ] && high=$(cat $S/memory.max)
  cur=$(cat $S/memory.current); file=$(awk '/^file /{print $2}' $S/memory.stat); cur=$((cur-file)); head_mb=$(( (high-cur)/1048576 ))
  rss=$(ps -u joe -o rss=,args= | awk "/chromium-clang|ninja|cargo|rustc/{s+=\$1} END{print int(s/1024)}")
  [ $rss -gt $peak ] && peak=$rss
  echo "$(date +%T) slice_headroom_mb=$head_mb slice_cur_mb=$((cur/1048576)) build_rss_mb=$rss peak_rss_mb=$peak load=$(cut -d" " -f1 /proc/loadavg) paused=$paused" >> $log
  if [ $head_mb -lt 2560 ] && [ $paused = 0 ]; then pkill -STOP -u joe -f chromium-clang/bin/clang; pkill -STOP -u joe -x ninja; paused=1; echo "PAUSED" >> $log; fi
  if [ $head_mb -gt 4096 ] && [ $paused = 1 ]; then pkill -CONT -u joe -x ninja; pkill -CONT -u joe -f chromium-clang/bin/clang; paused=0; echo "RESUMED" >> $log; fi
  sleep 10
done
pkill -CONT -u joe -x ninja 2>/dev/null; pkill -CONT -u joe -f chromium-clang/bin/clang 2>/dev/null
echo "END peak_rss_mb=$peak" >> $log
