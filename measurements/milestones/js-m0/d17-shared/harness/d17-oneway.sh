#!/usr/bin/env bash
# JS-D17 leg 3: the brand discipline, checked by compiling it.
#
#   d17-oneway.sh <out-dir>
#
# Every `oneway/control-*.rs` must compile and every `oneway/fail-*.rs`
# must not. A case that compiles when it should not is the interesting
# failure: it means the rule the spec states is not enforced. The
# compiler's own message for each refusal is kept, because the message is
# the evidence that it was refused for the stated reason.
#
# A refusal alone is not enough. Every `fail-*.rs` carries an
# `// EXPECT-ERROR: E####` line, and a refusal that does not report that
# code reads as FAIL: without that check a crate-version mismatch
# (`E0460`, `E0464`), a missing crate (`E0463`) or a typo would refuse
# every case and read as the rule holding.
#
# The cases that use `hg-heap` are compiled against the rlibs the harness
# crate has already built; the one-way prototype cases need no crate at
# all, so they also run where `hg-heap` has not been built.
set -u

OUT=${1:?usage: d17-oneway.sh <out-dir>}
HERE=$(cd "$(dirname "$0")" && pwd)
CASES="$HERE/oneway"
DEPS="$HERE/d17-heaps/target/release/deps"
mkdir -p "$OUT" "$OUT/oneway-errors"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/d17-oneway.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

# Without a compiler every case would "fail to compile", which would read
# as the rule holding. That is the one outcome this script must never
# report, so a missing rustc stops it.
command -v rustc >/dev/null || { echo "d17-oneway.sh: no rustc in PATH" >&2; exit 2; }

rlib() { # rlib <crate> -> the newest rlib for it, or empty
  ls -t "$DEPS"/lib"$1"-*.rlib 2>/dev/null | head -1
}

EXTERN=""
MISSING=""
for c in hg_heap hg_substrate hg_substrate_os; do
  p=$(rlib "$c")
  if [ -n "$p" ]; then
    EXTERN="$EXTERN --extern $c=$p"
  else
    MISSING="$MISSING $c"
  fi
done

printf 'case\texpect\toutcome\tverdict\tnote\n' > "$OUT/oneway.tsv"
fails=0

for f in "$CASES"/control-*.rs "$CASES"/fail-*.rs; do
  name=$(basename "$f" .rs)
  case "$name" in control-*) expect=compiles ;; *) expect=refused ;; esac
  # The prototype cases include their module by path and need no crates.
  needs_crates=1
  grep -q '^mod shared_brand' "$f" && needs_crates=0
  note=""
  if [ "$needs_crates" = 1 ] && [ -n "$MISSING" ]; then
    printf '%s\t%s\tskipped\tSKIP\tno rlib for%s (build d17-heaps first)\n' \
      "$name" "$expect" "$MISSING" >> "$OUT/oneway.tsv"
    continue
  fi
  args="--edition 2024 --crate-type bin -o $TMP/$name"
  [ "$needs_crates" = 1 ] && args="$args -L dependency=$DEPS $EXTERN"
  # shellcheck disable=SC2086 # args and EXTERN are flag lists
  want_code=$(sed -n 's|^// EXPECT-ERROR: *\(E[0-9]*\).*|\1|p' "$f" | head -1)
  if rustc $args "$f" > "$OUT/oneway-errors/$name.txt" 2>&1; then
    outcome=compiles
  else
    outcome=refused
    note=$(grep -oE 'error\[E[0-9]+\]' "$OUT/oneway-errors/$name.txt" | sort -u | tr '\n' ',' | sed 's/,$//')
  fi
  if [ "$outcome" != "$expect" ]; then
    verdict=FAIL
    fails=$((fails + 1))
  elif [ "$expect" = refused ] && [ -n "$want_code" ] &&
       ! grep -q "error\[$want_code\]" "$OUT/oneway-errors/$name.txt"; then
    # Refused, but not for the reason the case is about.
    verdict=FAIL
    note="$note (expected $want_code)"
    fails=$((fails + 1))
  elif [ "$expect" = refused ] && [ -z "$want_code" ]; then
    verdict=FAIL
    note="no EXPECT-ERROR line in the case"
    fails=$((fails + 1))
  else
    verdict=ok
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$expect" "$outcome" "$verdict" "$note" \
    >> "$OUT/oneway.tsv"
done

cat "$OUT/oneway.tsv"
if [ "$fails" -gt 0 ]; then
  echo "$fails case(s) did not behave as the rule requires" >&2
  exit 1
fi
