#!/usr/bin/env bash
# FP-M0 idle-wakeup counter, Linux: sums voluntary and involuntary context
# switches over every thread of a process (/proc/<pid>/task/*/status) at
# the start and end of a window; a thread's wakeup from a blocking wait is
# a voluntary switch out followed by a switch in, so voluntary switches
# per second approximate wakeups per second.
# Usage: wakeups-linux.sh <pid> <seconds>
set -euo pipefail
pid=$1; secs=$2
sum() { cat /proc/"$pid"/task/*/status 2>/dev/null | awk -v k="$1" '$1==k":" {s+=$2} END {print s+0}'; }
cpu() { awk '{print $14+$15}' /proc/"$pid"/stat; }
v0=$(sum voluntary_ctxt_switches); n0=$(sum nonvoluntary_ctxt_switches); c0=$(cpu)
sleep "$secs"
v1=$(sum voluntary_ctxt_switches); n1=$(sum nonvoluntary_ctxt_switches); c1=$(cpu)
tck=$(getconf CLK_TCK)
awk -v p="$pid" -v s="$secs" -v v=$((v1-v0)) -v n=$((n1-n0)) -v c=$((c1-c0)) -v t="$tck" -v th="$(find /proc/"$pid"/task -mindepth 1 -maxdepth 1 | wc -l)" \
  'BEGIN {printf "pid=%s window=%ds threads=%d voluntary=%d (%.3f/s) involuntary=%d (%.3f/s) cpu=%.1f ms\n", p, s, th, v, v/s, n, n/s, c*1000/t}'
