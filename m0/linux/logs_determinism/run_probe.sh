#!/bin/bash
# Same sequence as macOS m0/logs/determinism_probe{,_debugv8}.log
R=$HOME/hgba-m0/target-probe-release/release/determinism-probe
D=$HOME/hgba-m0/target-probe-debug/debug/determinism-probe
L=$HOME/hgba-m0/logs
{ uname -a; nproc; uptime
for v in pred pred-inline pred-noflags default default-flags; do
  nice -n 15 $R $v 1000; nice -n 15 $R spawn $v 50
done; echo done; } > $L/determinism_probe_linux.log 2>&1
{ uptime
nice -n 15 $D pred 200; nice -n 15 $D spawn pred 20
nice -n 15 $D pred-inline 200
nice -n 15 $D pred-noflags 200
nice -n 15 $D default-flags 200
nice -n 15 $D default 200
echo done; } > $L/determinism_probe_linux_debugv8.log 2>&1
