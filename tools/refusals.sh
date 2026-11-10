#!/usr/bin/env bash
# The code the compiler must refuse, checked by compiling it: CS-D3
# criteria 1 and 4 re-run against `hg-heap`'s `Mutator` and `Root` and
# the engine's `ScriptToken` (coding standard CS-D3, CS-TY-10), and the
# four cross-heap cases of the heap design review's "Tests" section.
#
#   tools/refusals.sh [out-dir]
#
# Every `control-*.rs` fixture must compile and every `fail-*.rs` must
# not. A `fail-*` case that compiles is the interesting failure: it means
# a rule the specs state is not enforced by the type system after all.
# The fixture sets, and the crates each is compiled against, are the
# `SETS` table below.
#
# Four things keep a green run from being empty:
#
#  * A refusal alone is not evidence. Each `fail-*.rs` carries an
#    `// EXPECT-ERROR: E####` line and must be refused with that code. A
#    lifetime error has no code, so those cases write `LIFETIME`. Without
#    this check a stale rlib (E0460, E0464), a missing crate (E0463) or a
#    typo in a fixture would refuse every case and read as the whole
#    discipline holding.
#  * The code is not always enough either: a case can be refused by an
#    incidental borrow it happens to contain rather than by the rule. So
#    a fixture may add an `// EXPECT-NOTE: <text>` line that the message
#    must also contain -- for the cross-heap cases, the note where rustc
#    names which of the heap's types is invariant over its brand. That
#    check is not decoration: it caught `fail-cross-heap-store` being
#    refused because the outer mutator escaped the inner closure, with
#    the brand never consulted.
#  * The controls must compile. A run in which they do not is a broken
#    harness, and its `fail-*` results prove nothing.
#  * The rlibs come from cargo's own artifact messages, not from the
#    newest `lib<crate>-*.rlib` in `deps` (see tools/refusals-rlibs.py).
#
# The fixtures are compiled with `--emit=metadata`, so they type-check
# and borrow-check without allocating a cage or linking a binary. Each
# takes the heaps, mutators, scopes and agents it needs as parameters, so
# no fixture needs a `Memory` implementation or a running engine.
#
# Bash 3.2 compatible (macOS /bin/bash).
set -u

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
# One row per fixture set: name | cargo package to build | fixture
# directory | the crates the fixtures may name, as `--extern`s. Building
# the package also builds everything it depends on, and cargo reports an
# artifact for each, so one build serves every extern of the row.
SETS="
heap|hg-heap|crates/hg-heap/tests/ui|hg-heap
script|hg-js-runtime|crates/hg-js-runtime/tests/ui|hg-heap hg-js-ops hg-js-bytecode hg-js-runtime
"
OUT=${1:-}
TMP=$(mktemp -d "${TMPDIR:-/tmp}/refusals.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
[ -n "$OUT" ] || OUT="$TMP/out"
mkdir -p "$OUT/errors" || exit 2

# Without a compiler every case would "fail to compile", which would read
# as the discipline holding. That is the one outcome this script must
# never report, so a missing toolchain stops it.
command -v rustc > /dev/null || { echo "refusals.sh: no rustc in PATH" >&2; exit 2; }
command -v cargo > /dev/null || { echo "refusals.sh: no cargo in PATH" >&2; exit 2; }
command -v python3 > /dev/null || { echo "refusals.sh: no python3 in PATH" >&2; exit 2; }

printf 'set\tcase\texpect\toutcome\tverdict\tcodes\tnote\n' > "$OUT/refusals.tsv"
fails=0 controls=0 refusals=0 sets=0

while IFS='|' read -r setname package dir externs; do
  [ -n "${setname:-}" ] || continue
  cases="$ROOT/$dir"
  [ -d "$cases" ] || { echo "refusals.sh: no fixtures at $dir" >&2; exit 2; }
  sets=$((sets + 1))

  if ! ( cd "$ROOT" && cargo build -p "$package" --locked --message-format=json ) \
      > "$TMP/build.json" 2> "$TMP/build.err"; then
    echo "refusals.sh: $package does not build; its fixtures would all be refused" >&2
    tail -n 20 "$TMP/build.err" >&2
    exit 2
  fi
  # shellcheck disable=SC2086 # externs is a list of package names
  if ! python3 "$ROOT/tools/refusals-rlibs.py" "$TMP/build.json" $externs > "$TMP/rlibs"; then
    echo "refusals.sh: cannot name the rlibs for set $setname, so its fixtures would be compiled against crates this run did not build" >&2
    exit 2
  fi
  extern_args="" deps=""
  while IFS='=' read -r pkg path; do
    [ -n "${pkg:-}" ] || continue
    extern_args="$extern_args --extern $(echo "$pkg" | tr - _)=$path"
    # Cargo names the uplifted copy in `target/<profile>/`, so the search
    # path for a crate's own dependencies is the `deps` beside it.
    d=$(dirname "$path")
    [ -d "$d/deps" ] && d="$d/deps"
    deps="$d"
  done < "$TMP/rlibs"
  [ -n "$deps" ] || { echo "refusals.sh: no dependency path for set $setname" >&2; exit 2; }

  for f in "$cases"/control-*.rs "$cases"/fail-*.rs; do
    [ -f "$f" ] || continue
    name=$(basename "$f" .rs)
    case "$name" in
      control-*) expect=compiles; controls=$((controls + 1)) ;;
      *) expect=refused; refusals=$((refusals + 1)) ;;
    esac
    want_code=$(sed -n 's|^// EXPECT-ERROR: *\([A-Z0-9]*\).*|\1|p' "$f" | head -1)
    want_note=$(sed -n 's|^// EXPECT-NOTE: *||p' "$f" | head -1)
    log="$OUT/errors/$setname-$name.txt"
    # shellcheck disable=SC2086 # extern_args is a flag list
    if rustc --edition 2024 --crate-type lib --emit metadata \
        --crate-name "$(echo "$name" | tr - _)" \
        -L "dependency=$deps" $extern_args \
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

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$setname" "$name" "$expect" "$outcome" "$verdict" "$codes" "$note" >> "$OUT/refusals.tsv"
  done
done <<EOF
$SETS
EOF

# A run with no case of either kind, or a set that vanished, would pass
# while checking nothing.
want_sets=$(printf '%s\n' "$SETS" | sed '/^$/d' | wc -l | tr -d ' ')
if [ "$sets" -ne "$want_sets" ] || [ "$controls" -eq 0 ] || [ "$refusals" -eq 0 ]; then
  echo "refusals.sh: $sets of $want_sets sets, $controls control and $refusals fail fixtures; every set and both kinds are needed" >&2
  exit 2
fi

awk -F'\t' 'NR>1 { printf "%-7s %-34s %-9s %-9s %-5s %s %s\n", $1, $2, $3, $4, $5, $6, $7 }' "$OUT/refusals.tsv"
echo "$sets sets, $controls control, $refusals fail fixtures; $fails did not behave as the rule requires"
[ "$fails" -eq 0 ] || exit 1
