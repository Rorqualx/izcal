#!/bin/bash
# M1 phase G, Windows: FP-27 cold spawn-to-ready, cold after reboot (owner
# decision, 26 October 2026: reboot rig253 before each of 20 cold runs,
# all under one exclusive booking). Run on the Mac, where `rig` lives,
# inside the session's exclusive booking of rig253.
#
# Each run:
#   1. The game-lock gate: `rig check --json` must show staticR's per-host
#      game-run lock free, no live queue ticket and no running *_rt64
#      game. It is never taken here; while it is held this waits and
#      re-checks, and never reboots.
#   2. `shutdown /r /t 0` over ssh; wait for ssh to go down, then up.
#   3. A fixed settle period after ssh answers (SETTLE seconds).
#   4. `rig check` until an OK line (the evidence line, kept).
#   5. One measured launch: `hg-launch-tool measure-spawn 1`, one broker
#      launching both kinds, the first execution of each binary since
#      the reboot.
#
# Usage: g-cold-reboot-windows.sh N SETTLE OUT_TSV EVIDENCE [BUILD]
#   BUILD: the release directory on rig253, relative to %USERPROFILE%
#   (default izcal-m1g-final-target\release).
set -u
N=$1; SETTLE=$2; OUT=$3; EV=$4; BUILD=${5:-'izcal-m1g-final-target\release'}
SSH="ssh -o BatchMode=yes -o ConnectTimeout=5 rig253"
up() { $SSH "exit 0" 2>/dev/null; }  # cmd.exe: there is no `true`
boot_id() { $SSH "powershell -NoProfile -Command \"(Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToString('o')\"" 2>/dev/null | tr -d '\r'; }
log() { echo "$(date -Iseconds) $*" | tee -a "$EV"; }

gate() {
  # 0 when staticR's lock is free with no live ticket and no game runs.
  local j
  j=$(rig check rig253 --kind exclusive --session m1-substrate --json --fresh 2>/dev/null) || true
  python3 -I -c '
import json, sys
try:
    p = json.loads(sys.stdin.read())["probe"]
except (ValueError, KeyError):
    p = {}
lock = p.get("lock", {}).get("state")
live = p.get("queue", {}).get("live")
games = p.get("games")
ok = bool(p.get("reachable")) and lock == "free" and live == 0 and games == 0
print("reachable=%s lock=%s queue_live=%s games=%s" % (p.get("reachable"), lock, live, games))
sys.exit(0 if ok else 1)' <<< "$j"
}

[ -s "$OUT" ] || printf 'run\tkind\tspawn_to_ready_ms\tboot_s\n' > "$OUT"
for ((i = 1; i <= N; i++)); do
  until g=$(gate); do log "run $i: game-run lock not free ($g); waiting 60 s, no reboot"; sleep 60; done
  before=$(boot_id)
  [ -n "$before" ] || { log "run $i: cannot read the boot time; stopping"; exit 1; }
  log "run $i: gate $g; booted $before; rebooting"
  # The session ends with the host, without a reset: keepalives end ssh.
  $SSH -o ServerAliveInterval=2 -o ServerAliveCountMax=3 "shutdown /r /t 0" 2>&1 \
    | grep -v -i 'post-quantum\|store now\|upgraded' | tee -a "$EV"
  t0=$(date +%s)
  went=0
  for ((k = 0; k < 60; k++)); do up || { went=1; break; }; sleep 2; done
  [ $went = 1 ] || { log "run $i: ssh never went down (shutdown refused?); stopping"; exit 1; }
  for ((k = 0; k < 180; k++)); do up && break; sleep 5; done
  up || { log "run $i: ssh did not come back in 15 min; stopping"; exit 1; }
  boot=$(( $(date +%s) - t0 ))
  after=$(boot_id)
  [ -n "$after" ] && [ "$after" != "$before" ] || { log "run $i: boot time unchanged ($before -> $after); stopping"; exit 1; }
  log "run $i: ssh back after ${boot} s, booted $after; settling ${SETTLE} s"
  sleep "$SETTLE"
  for ((k = 0; k < 20; k++)); do
    c=$(rig check rig253 --kind exclusive --session m1-substrate --fresh) && break
    sleep 12
  done
  log "run $i: $c"
  case "$c" in "rig check OK"*) ;; *) log "run $i: no OK check; stopping"; exit 1 ;; esac
  rows=$($SSH "\"%USERPROFILE%\\$BUILD\\hg-launch-tool.exe\" measure-spawn 1" 2>/dev/null | tr -d '\r' | tail -n +2)
  [ -n "$rows" ] || { log "run $i: no measurement; stopping"; exit 1; }
  echo "$rows" | awk -v run="$i" -v boot="$boot" 'BEGIN{OFS="\t"} {print run, $2, $3, boot}' >> "$OUT"
  log "run $i: $(echo "$rows" | tr '\n' ' ')"
done
awk -F'\t' 'NR>1 {v[$2]=v[$2]" "$3} END {for (k in v) {n=split(v[k],a," "); for(i=1;i<=n;i++)for(j=i+1;j<=n;j++)if(a[j]+0<a[i]+0){t=a[i];a[i]=a[j];a[j]=t}; m=(n%2)?a[(n+1)/2]:(a[n/2]+a[n/2+1])/2; printf "%s n=%d median=%.3f min=%.3f max=%.3f\n", k, n, m, a[1], a[n]}}' "$OUT"
