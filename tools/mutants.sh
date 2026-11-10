#!/usr/bin/env bash
# JS-10 (b): the mutation-generated variants of the heap planted-bug
# classes, compiled one by one.
#
#   tools/mutants.sh [out-dir] [--sample]
#
# `tools/mutants.py` writes the variants; this script compiles each one
# and checks what the compiler did with it. A variant of a compile-caught
# class must be refused with the code its class names; a variant that
# compiles by design must compile, and its manifest row says which other
# mechanism catches it.
#
# Three things keep a green run from being empty. The first two are the
# ones tools/refusals.sh already uses -- the expected error code, and a
# toolchain that must be present, because without a compiler every
# variant would "fail to compile" and read as the whole discipline
# holding. The third is new and is what makes a generated suite worth as
# much as a hand-read one:
#
#  * Every variant is compiled together with its **twin**: the same
#    program with the single bug-introducing step undone. The variant
#    must be refused and the twin must compile. A refusal is then
#    attributed to the planted bug, because the twin differs from it by
#    nothing else. Without this a generator bug -- a missing import, a
#    renamed field, a stray brace -- would refuse all 166 variants and
#    report a perfect score. In #83 four hand-written cross-heap cases
#    were refused for the wrong reason and only reading the diagnostics
#    found it; at this size nobody reads them.
#
# The class floor (at least 20 variants, JS-10 (b)) is checked here as
# well as in the generator, so editing the manifest cannot thin a class.
#
# Bash 3.2 compatible (macOS /bin/bash).
set -u

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIN_PER_CLASS=20
PACKAGE=hg-heap
EXTERNS=hg-heap

OUT=${1:-}
MODE=${2:-}
case "${OUT:-}" in --sample) MODE=--sample; OUT="" ;; esac
TMP=$(mktemp -d "${TMPDIR:-/tmp}/mutants.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
[ -n "$OUT" ] || OUT="$TMP/out"
mkdir -p "$OUT/errors" "$TMP/src" || exit 2

command -v rustc > /dev/null || { echo "mutants.sh: no rustc in PATH" >&2; exit 2; }
command -v cargo > /dev/null || { echo "mutants.sh: no cargo in PATH" >&2; exit 2; }
command -v python3 > /dev/null || { echo "mutants.sh: no python3 in PATH" >&2; exit 2; }

sample=""
[ "${MODE:-}" = "--sample" ] && sample="--sample"
# shellcheck disable=SC2086 # sample is one optional flag
if ! python3 "$ROOT/tools/mutants.py" --out "$TMP/src" $sample > "$TMP/gen"; then
  echo "mutants.sh: the generator refused to write the suite" >&2
  cat "$TMP/gen" >&2
  exit 2
fi
generated=$(cat "$TMP/gen")

if ! ( cd "$ROOT" && cargo build -p "$PACKAGE" --locked --message-format=json ) \
    > "$TMP/build.json" 2> "$TMP/build.err"; then
  echo "mutants.sh: $PACKAGE does not build; every variant would be refused" >&2
  tail -n 20 "$TMP/build.err" >&2
  exit 2
fi
# shellcheck disable=SC2086 # EXTERNS is a list of package names
if ! python3 "$ROOT/tools/refusals-rlibs.py" "$TMP/build.json" $EXTERNS > "$TMP/rlibs"; then
  echo "mutants.sh: cannot name the rlibs, so the variants would be compiled against crates this run did not build" >&2
  exit 2
fi
extern_args="" deps=""
while IFS='=' read -r pkg path; do
  [ -n "${pkg:-}" ] || continue
  extern_args="$extern_args --extern $(echo "$pkg" | tr - _)=$path"
  d=$(dirname "$path")
  [ -d "$d/deps" ] && d="$d/deps"
  deps="$d"
done < "$TMP/rlibs"
[ -n "$deps" ] || { echo "mutants.sh: no dependency path" >&2; exit 2; }

# Compiles one file. Sets `outcome` and `codes`.
compile_one() {
  local f=$1 log=$2 name
  name=$(basename "$f" .rs | tr -c 'a-zA-Z0-9' _)
  # shellcheck disable=SC2086 # extern_args is a flag list
  if rustc --edition 2024 --crate-type lib --emit metadata \
      --crate-name "x$name" -L "dependency=$deps" $extern_args \
      --out-dir "$TMP" "$f" > "$log" 2>&1; then
    outcome=compiles
    codes=""
  else
    outcome=refused
    codes=$(grep -oE 'error\[E[0-9]+\]' "$log" | sed 's/error\[//;s/\]//' | sort -u | tr '\n' ',' | sed 's/,$//')
    if [ -z "$codes" ] && grep -qE 'lifetime may not live long enough|borrowed data escapes|closure may outlive|does not live long enough' "$log"; then
      codes=LIFETIME
    fi
  fi
}

printf 'class\tcase\texpect\toutcome\ttwin\tverdict\tcodes\tnote\n' > "$OUT/mutants.tsv"
fails=0 rows=0
while IFS=$'\t' read -r class case expect code _cnote variant twin _axes; do
  [ -n "${class:-}" ] || continue
  rows=$((rows + 1))
  # The generator writes `-` for an empty cell (see mutants.py).
  [ "$twin" = - ] && twin=""
  [ "$code" = - ] && code=""
  [ "$_cnote" = - ] && _cnote=""
  compile_one "$TMP/src/$variant" "$OUT/errors/$class-$case.txt"
  v_outcome=$outcome v_codes=$codes
  t_outcome=skipped
  if [ -n "$twin" ]; then
    compile_one "$TMP/src/$twin" "$OUT/errors/$class-$case-twin.txt"
    t_outcome=$outcome t_codes=$codes
  fi

  verdict=ok note=""
  if [ "$v_outcome" != "$expect" ]; then
    verdict=FAIL
    if [ "$expect" = refused ]; then
      note="compiled, but the class says the mechanism refuses it"
    else
      note="refused ($v_codes), but this variant must compile: $_cnote"
    fi
  elif [ "$expect" = refused ] && ! echo ",$v_codes," | grep -q ",$code,"; then
    verdict=FAIL
    note="refused with $v_codes but not $code, so not for the reason the class is about"
  elif [ -n "$twin" ] && [ "$t_outcome" != compiles ]; then
    verdict=FAIL
    note="the twin was refused too ($t_codes), so the variant's refusal is not the planted bug"
  fi
  [ "$verdict" = FAIL ] && fails=$((fails + 1))
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$class" "$case" "$expect" "$v_outcome" "$t_outcome" "$verdict" "$v_codes" "$note" \
    >> "$OUT/mutants.tsv"
done < "$TMP/src/manifest.tsv"

if [ "$rows" -eq 0 ]; then
  echo "mutants.sh: the manifest is empty, so this run checked nothing" >&2
  exit 2
fi

# Per-class counts, and JS-10 (b)'s floor. In --sample mode the set is
# deliberately thinner: one variant per axis value, with the full
# covering array left to CI.
if [ -z "$sample" ]; then
  thin=$(awk -F'\t' 'NR>1 {n[$1]++} END {for (c in n) if (n[c] < '"$MIN_PER_CLASS"') print c, n[c]}' "$OUT/mutants.tsv")
  if [ -n "$thin" ]; then
    echo "mutants.sh: under JS-10 (b)'s floor of $MIN_PER_CLASS variants: $thin" >&2
    exit 2
  fi
fi

awk -F'\t' 'NR>1 && $6 != "ok" { printf "%-22s %-46s %-9s %-9s %-8s %s %s\n", $1, $2, $3, $4, $5, $7, $8 }' "$OUT/mutants.tsv"
awk -F'\t' 'NR>1 {n[$1]++} END {for (c in n) printf "%s=%d ", c, n[c]}' "$OUT/mutants.tsv" | tr -d '\n'
echo ""
echo "$generated; $rows checked, $fails did not behave as the class requires"
[ "$fails" -eq 0 ] || exit 1
