#!/bin/bash
# usage: build_v8.sh release|debug
set -u
prof=$1
source ~/hgba-m0/scripts/env.sh
export CARGO_TARGET_DIR=$HOME/hgba-m0/target-sandbox
cd ~/hgba-m0/rusty_v8
flag=""; [ $prof = release ] && flag="--release"
log=~/hgba-m0/logs/rebuild_$prof.log
echo "start $(date -Is)" > $log; free -g >> $log; uptime >> $log
echo $$ > ~/hgba-m0/logs/build_$prof.pid
nohup ~/hgba-m0/scripts/monitor_anon.sh ~/hgba-m0/logs/mem_rebuild_$prof.log ~/hgba-m0/logs/build_$prof.pid >/dev/null 2>&1 &
s=$(date +%s)
PRINT_GN_ARGS=1 nice -n 15 ionice -c3 /usr/bin/time -v cargo build $flag --lib --features v8_enable_sandbox -vv >> $log 2>&1
rc=$?
e=$(date +%s)
echo "EXIT rc=$rc wall_s=$((e-s)) end $(date -Is)" >> $log
du -sh $CARGO_TARGET_DIR/$prof >> $log 2>&1
