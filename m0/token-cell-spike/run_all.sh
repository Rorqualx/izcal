#!/usr/bin/env bash
# CS-D3 token-cell spike: run every check and write logs to logs/.
# Needs a heavy booking on the rig (see ~/.rigs/README.md). No V8 build: the
# V8 half links the fork's prebuilt sandbox release archive.
set -u
SPIKE="$(cd "$(dirname "$0")" && pwd)"
LOGS="$SPIKE/logs"
mkdir -p "$LOGS"
step=${1:-all}

{ rustc -V; cargo -V; rustc +nightly -V; cargo +nightly miri --version; sw_vers; sysctl -n machdep.cpu.brand_string; } > "$LOGS"/env.txt 2>&1

if [[ $step == all || $step == core ]]; then
  cd "$SPIKE/token-cell" || exit 1
  cargo test 2>&1 | tee "$LOGS"/core_test.log | tail -30
fi

if [[ $step == all || $step == miri ]]; then
  cd "$SPIKE/token-cell" || exit 1
  MIRIFLAGS="-Zmiri-strict-provenance" cargo +nightly miri test --lib 2>&1 | tee "$LOGS"/miri.log | tail -40
  MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test --lib 2>&1 | tee "$LOGS"/miri_tree_borrows.log | tail -10
fi

if [[ $step == all || $step == v8 ]]; then
  # shellcheck source=/dev/null
  source "$SPIKE/../scripts/consume-release.sh"
  cd "$SPIKE/token-cell-v8" || exit 1
  cargo test 2>&1 | tee "$LOGS"/v8_test.log | tail -40
fi

if [[ $step == all || $step == clippy ]]; then
  # shellcheck source=/dev/null
  source "$SPIKE/../scripts/consume-release.sh"
  cd "$SPIKE/token-cell-v8" || exit 1
  cargo clippy --lib 2>&1 | tee "$LOGS"/clippy_clean.log | tail -5
  cargo clippy --lib --features misuse 2>&1 | tee "$LOGS"/clippy_misuse.log | grep -E "^(warning|error)" | sort | uniq -c
fi

if [[ $step == all || $step == bench ]]; then
  cd "$SPIKE/token-cell" || exit 1
  cargo build --release --bin bench 2>&1 | tail -2
  B="$SPIKE/token-cell/target/release/bench"
  : > "$LOGS"/bench.log
  for round in 1 2 3; do
    for mode in token refcell lend nolend; do
      echo "== round $round mode $mode" >> "$LOGS"/bench.log
      /usr/bin/time -l "$B" $mode 15 200 >> "$LOGS"/bench.log 2>&1
    done
  done
  grep -E "^mode=|instructions retired|cycles elapsed" "$LOGS"/bench.log
fi
