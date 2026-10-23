#!/bin/bash
# Run every mode under every build, recording output and exit status/signal.
# Then the sandbox-exec cross-checks. Output: logs/run-<stamp>.log
set -uo pipefail
cd "$(dirname "$0")" || exit 1
HERE=$(pwd -P)
STAMP=$(date +%Y%m%d-%H%M%S)
LOG=logs/run-$STAMP.log
ALLOWED=$HERE/fixtures/allowed.txt
DENIED=$HERE/fixtures/denied.txt
TMPD=$HERE/fixtures
echo "allowed fixture" > "$ALLOWED"; echo "denied fixture" > "$DENIED"
START=$(date '+%Y-%m-%d %H:%M:%S')
exec > >(tee "$LOG") 2>&1
echo "# start $START"; sw_vers; uname -a; xcrun clang --version | head -1; xcrun ld -v 2>&1 | head -1
xcrun --show-sdk-version; xcrun --show-sdk-path

status() { # decode $1
  local s=$1
  if [ "$s" -gt 128 ]; then echo "exit=$s signal=$((s-128)) ($(kill -l $((s-128)) 2>/dev/null))"; else echo "exit=$s"; fi
}

for b in bin/probe-sdk262 bin/probe-sdk27-ld bin/probe-sdk27-min27 bin/probe-sdk27-vtool bin/probe-sdk28-ld; do
  for m in baseline custom-init custom-params named-pure named-nonet mach; do
    echo "=== $b $m"
    "$b" "$m" "$ALLOWED" "$DENIED" "$TMPD"
    echo "STATUS $b $m $(status $?)"
  done
done

# sandbox-exec cross-checks (system binary /usr/bin/sandbox-exec applies the profile, then execs).
# Deny-default with broad reads minus one literal. A narrower read allowlist
# (/usr/lib, /System, /private/preboot, /private/var/db/dyld, the bin dir) makes
# dyld abort in CacheFinder at exec on every SDK build, so it is a profile
# problem, not an SDK one; see sec-d6.md.
SBX_RUNTIME='(version 1)
(deny default)
(allow process-exec* (subpath "'$HERE'/bin"))
(allow file-read*)
(deny file-read* (literal "'$DENIED'"))
(allow sysctl-read)
(allow network-bind (local ip "localhost:*"))
(allow network-inbound (local ip "localhost:*"))
(allow mach-lookup (global-name "com.apple.system.notification_center"))'
SBX_NARROW='(version 1)
(deny default)
(allow process-exec* (subpath "'$HERE'/bin"))
(allow file-read-metadata)
(allow file-read* (subpath "/usr/lib") (subpath "/System") (subpath "/private/preboot") (subpath "/private/var/db/dyld") (subpath "'$HERE'/bin") (literal "'$ALLOWED'"))
(allow sysctl-read)'
echo "=== sandbox-exec /bin/cat denied (allow default + deny one literal)"
sandbox-exec -p "(version 1)(allow default)(deny file-read* (literal \"$DENIED\"))" /bin/cat "$DENIED"
echo "STATUS sandbox-exec-cat-denied $(status $?)"
echo "=== sandbox-exec /bin/cat allowed (same profile)"
sandbox-exec -p "(version 1)(allow default)(deny file-read* (literal \"$DENIED\"))" /bin/cat "$ALLOWED"
echo "STATUS sandbox-exec-cat-allowed $(status $?)"
echo "=== sandbox-exec -n pure-computation /usr/bin/true"
sandbox-exec -n pure-computation /usr/bin/true
echo "STATUS sandbox-exec-named-pure $(status $?)"
for b in bin/probe-sdk262 bin/probe-sdk27-ld bin/probe-sdk27-min27; do
  echo "=== sandbox-exec deny-default $b checks"
  sandbox-exec -p "$SBX_RUNTIME" "$b" checks "$ALLOWED" "$DENIED" "$TMPD"
  echo "STATUS sandbox-exec $b $(status $?)"
done

for b in bin/probe-sdk262 bin/probe-sdk27-ld; do
  echo "=== sandbox-exec narrow-read $b checks"
  sandbox-exec -p "$SBX_NARROW" "$b" checks "$ALLOWED" "$DENIED" "$TMPD"
  echo "STATUS sandbox-exec-narrow $b $(status $?)"
done
echo "# end $(date '+%Y-%m-%d %H:%M:%S')"
echo "=== log show (Sandbox / kernel sandbox lines since start)"
log show --style compact --start "$START" --predicate '(sender == "Sandbox") OR (sender == "libsystem_sandbox.dylib") OR (subsystem == "com.apple.sandbox") OR (eventMessage CONTAINS[c] "probe-sdk")' 2>/dev/null | tail -200
echo "=== crash reports since start"
find ~/Library/Logs/DiagnosticReports -newermt "$START" -name '*probe*' -print 2>/dev/null
