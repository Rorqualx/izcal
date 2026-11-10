#!/usr/bin/env bash
# The heap's brand discipline, checked by compiling it: CS-D3 criterion 1
# re-run against `hg-heap`'s `Mutator` and `Root` (coding standard CS-D3,
# CS-TY-10), and the four cross-heap cases of the heap design review's
# "Tests" section.
#
#   tools/heap-ui.sh [out-dir]
#
# Every `crates/hg-heap/tests/ui/control-*.rs` must compile and every
# `fail-*.rs` must not. A `fail-*` case that compiles is the interesting
# failure: it means a rule the specs state is not enforced by the type
# system after all.
#
# Three things keep a green run from being empty:
#
#  * A refusal alone is not evidence. Each `fail-*.rs` carries an
#    `// EXPECT-ERROR: E####` line and must be refused with that code. A
#    lifetime error has no code, so those cases write `LIFETIME`. Without
#    this check a stale rlib (E0460, E0464), a missing crate (E0463) or a
#    typo in a fixture would refuse every case and read as the whole
#    discipline holding.
#  * The code is not always enough either: a case can be refused by an
#    incidental borrow it happens to contain rather than by the brand. So
#    a fixture may add an `// EXPECT-NOTE: <text>` line that the message
#    must also contain — for the cross-heap cases, the note where rustc
#    names which of the heap's types is invariant over its brand. That
#    check is not decoration: it caught `fail-cross-heap-store` being
#    refused because the outer mutator escaped the inner closure, with
#    the brand never consulted.
#  * The controls must compile. A run in which they do not is a broken
#    harness, and its `fail-*` results prove nothing.
#
# The fixtures are compiled with `--emit=metadata`, so they type-check
# and borrow-check without allocating a cage or linking a binary. Each
# takes the heaps, mutators and scopes it needs as parameters, so no
# fixture needs a `Memory` implementation and none depends on a crate
# below `hg-heap`.
#
# Bash 3.2 compatible (macOS /bin/bash).
set -u

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CASES="$ROOT/crates/hg-heap/tests/ui"
OUT=${1:-}
TMP=$(mktemp -d "${TMPDIR:-/tmp}/heap-ui.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
[ -n "$OUT" ] || OUT="$TMP/out"
mkdir -p "$OUT/errors" || exit 2

# Without a compiler every case would "fail to compile", which would read
# as the discipline holding. That is the one outcome this script must
# never report, so a missing toolchain stops it.
command -v rustc > /dev/null || { echo "heap-ui.sh: no rustc in PATH" >&2; exit 2; }
command -v cargo > /dev/null || { echo "heap-ui.sh: no cargo in PATH" >&2; exit 2; }
[ -d "$CASES" ] || { echo "heap-ui.sh: no fixtures at $CASES" >&2; exit 2; }

# The fixtures are compiled against the rlib this build produces, so the
# rlib and the fixtures are always the same revision of the crate. Cargo
# names the artifact itself rather than this script picking the newest
# `libhg_heap-*.rlib` in `deps`: that directory also holds rlibs of other
# feature sets and profiles, each with its own hash, and the newest by
# modification time is not necessarily the one this build produced.
if ! ( cd "$ROOT" && cargo build -p hg-heap --locked --message-format=json ) \
    > "$TMP/build.json" 2> "$TMP/build.err"; then
  echo "heap-ui.sh: hg-heap does not build; the fixtures would all be refused" >&2
  tail -n 20 "$TMP/build.err" >&2
  exit 2
fi
RLIB=$(python3 "$ROOT/tools/heap-ui-rlib.py" "$TMP/build.json")
if [ -z "$RLIB" ] || [ ! -f "$RLIB" ]; then
  echo "heap-ui.sh: cargo named no single hg-heap rlib, so the fixtures would be compiled against a crate this run did not build" >&2
  exit 2
fi
# Cargo names the uplifted copy in `target/<profile>/`, so the search
# path for hg-heap's own dependencies is the `deps` directory beside it.
RLIB_DIR=$(dirname "$RLIB")
if [ -d "$RLIB_DIR/deps" ]; then DEPS="$RLIB_DIR/deps"; else DEPS="$RLIB_DIR"; fi

printf 'case\texpect\toutcome\tverdict\tcodes\tnote\n' > "$OUT/ui.tsv"
fails=0 controls=0 refusals=0

for f in "$CASES"/control-*.rs "$CASES"/fail-*.rs; do
  name=$(basename "$f" .rs)
  case "$name" in
    control-*) expect=compiles; controls=$((controls + 1)) ;;
    *) expect=refused; refusals=$((refusals + 1)) ;;
  esac
  want_code=$(sed -n 's|^// EXPECT-ERROR: *\([A-Z0-9]*\).*|\1|p' "$f" | head -1)
  want_note=$(sed -n 's|^// EXPECT-NOTE: *||p' "$f" | head -1)
  log="$OUT/errors/$name.txt"
  if rustc --edition 2024 --crate-type lib --emit metadata \
      --crate-name "$(echo "$name" | tr - _)" \
      -L "dependency=$DEPS" --extern "hg_heap=$RLIB" \
      --out-dir "$TMP" "$f" > "$log" 2>&1; then
    outcome=compiles
    codes=""
  else
    outcome=refused
    codes=$(grep -oE 'error\[E[0-9]+\]' "$log" | sed 's/error\[//;s/\]//' | sort -u | tr '\n' ',' | sed 's/,$//')
    # A lifetime error carries no code of its own.
    if [ -z "$codes" ] && grep -qE 'lifetime may not live long enough|borrowed data escapes|closure may outlive' "$log"; then
      codes=LIFETIME
    fi
  fi

  note=""
  verdict=ok
  if [ "$outcome" != "$expect" ]; then
    verdict=FAIL
    if [ "$expect" = compiles ]; then
      note="a control was refused ($codes): the harness is broken, so the fail cases prove nothing"
    else
      note="compiled, but the rule says it must not (expected $want_code)"
    fi
  elif [ "$expect" = refused ] && [ -z "$want_code" ]; then
    verdict=FAIL
    note="no EXPECT-ERROR line in the fixture"
  elif [ "$expect" = refused ] && ! echo ",$codes," | grep -q ",$want_code,"; then
    verdict=FAIL
    note="refused with $codes but not $want_code, so not for the reason the case is about"
  elif [ "$expect" = refused ] && [ -n "$want_note" ] && ! grep -qF "$want_note" "$log"; then
    verdict=FAIL
    note="refused with $want_code, but the message does not say \"$want_note\""
  fi
  [ "$verdict" = FAIL ] && fails=$((fails + 1))

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$name" "$expect" "$outcome" "$verdict" "$codes" "$note" >> "$OUT/ui.tsv"
done

# A run with no case of either kind would pass while checking nothing.
if [ "$controls" -eq 0 ] || [ "$refusals" -eq 0 ]; then
  echo "heap-ui.sh: $controls control and $refusals fail fixtures; both kinds are needed" >&2
  exit 2
fi

awk -F'\t' 'NR>1 { printf "%-34s %-9s %-9s %-5s %s %s\n", $1, $2, $3, $4, $5, $6 }' "$OUT/ui.tsv"
echo "$controls control, $refusals fail fixtures; $fails did not behave as the rule requires"
[ "$fails" -eq 0 ] || exit 1
