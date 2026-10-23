#!/bin/bash
# Q-CS-3: run the probe under each seed configuration and hash what it generated.
cd "$(dirname "$0")" || exit 1
rm -rf out && mkdir out
run() { local name=$1; shift; env "$@" QCS3_OUT="out/$name" cargo test -q --test seed -- --test-threads=1 >"out/$name.log" 2>&1; }
run random1; run random2
run env42_1 PROPTEST_RNG_SEED=42; run env42_2 PROPTEST_RNG_SEED=42
run env43 PROPTEST_RNG_SEED=43
run env42_xs PROPTEST_RNG_SEED=42 PROPTEST_RNG_ALGORITHM=xs
for f in out/*.default out/*.fixed out/*.failing; do
  printf '%-22s n=%-4s sha=%s\n' "$f" "$(wc -l <"$f" | tr -d ' ')" "$(shasum -a 256 "$f" | cut -c1-16)"
done
