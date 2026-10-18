#!/usr/bin/env bash
# Local stand-in for a GitHub windows-2022 runner: provides what the skipped
# `uses:` steps would (checkout done by setup.bat; rustup reads
# rust-toolchain.toml; Python on PATH), then runs the rendered jobs in order.
PY="/c/Users/joe's pc/AppData/Local/Programs/Python/Python312"
export PATH="$PY:$PY/Scripts:$HOME/.cargo/bin:$PATH"
PYTHON="$(cygpath -w "$PY/python.exe")"
export PYTHON
# Laptop-specific: VS Build Tools live under Program Files (x86).
export vs2022_install='C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
for j in "$@"; do
  export RUN_DIR=/c/hgba-m0/run/$j; rm -rf "$RUN_DIR"; mkdir -p "$RUN_DIR"
  echo "=== $j start $(date)"
  bash /c/hgba-m0/xfer/$j.sh > "$RUN_DIR/job.log" 2>&1
  rc=$?
  [ -d /c/hgba-m0/rusty_v8/dist ] && mv /c/hgba-m0/rusty_v8/dist "$RUN_DIR/dist"
  echo "=== $j end rc=$rc $(date)"
  grep -E '^::(step|rc|failed|job-done)' "$RUN_DIR/job.log" | tail -3
  [ $rc -eq 0 ] || exit $rc
done
