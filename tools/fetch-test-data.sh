#!/usr/bin/env bash
# Fetches Izcal's pinned test-only data sets (tools/test-data.toml) into a
# cache outside the repository, checks each archive's SHA-256 and unpacks
# it. Nothing fetched here is committed, linked or shipped.
#
# A set's `format` says what was downloaded: tar.gz (the default, with
# `strip` components removed), zip (needs unzip), or file (one data file,
# placed in the tree under its own name). The hash is always of the
# downloaded bytes, so a pin is checked before anything is unpacked.
#
# A tar.gz set may add `paths`, a space-separated list of members to
# unpack *after* `strip` is applied -- so `paths = "css/CSS2"` on a whole
# repository tarball leaves only that directory on disk. The hash still
# covers the whole archive, so the pin means the same thing either way;
# what `paths` saves is the unpacking and the disk, which for a
# repository the size of WPT is the difference between 51 MB and 1.5 GB.
#
#   tools/fetch-test-data.sh [--check] [NAME ...]
#
# NAME is a table of tools/test-data.toml (default: all of them).
# --check verifies the cached archives and unpacked trees, downloading
#         nothing; it fails if one is missing or its hash differs.
# Environment: IZCAL_TEST_DATA (default ~/.cache/izcal/test-data).
# Layout: $IZCAL_TEST_DATA/archives/<name>-<commit>.<ext> and the tree in
#         $IZCAL_TEST_DATA/<name>-<commit>/, with .izcal-sha256 inside it
#         naming the archive it came from. A set with no commit of its own
#         gives `pin` instead, and that is used in both names.
# Bash 3.2 compatible (macOS /bin/bash).
set -euo pipefail
cd "$(dirname "$0")/.."

TOML=tools/test-data.toml
CACHE=${IZCAL_TEST_DATA:-$HOME/.cache/izcal/test-data}
CHECK=0
names=()
while [ $# -gt 0 ]; do
  case "$1" in
    --check) CHECK=1 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    -*) echo "fetch-test-data.sh: unknown option $1" >&2; exit 2 ;;
    *) names+=("$1") ;;
  esac
  shift
done
if [ ${#names[@]} -eq 0 ]; then
  while IFS= read -r n; do names+=("$n"); done < <(awk '/^\[[A-Za-z0-9_-]+\]$/ { gsub(/[][]/, ""); print }' "$TOML")
fi

# field NAME KEY: the string value of KEY in table [NAME].
field() {
  awk -v t="[$1]" -v k="$2" '
    /^\[/ { in_t = ($0 == t); next }
    in_t && $1 == k && $2 == "=" {
      v = $0; sub(/^[^=]*= *"/, "", v); sub(/" *$/, "", v); print v; exit
    }' "$TOML"
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

status=0
mkdir -p "$CACHE/archives"
for name in "${names[@]}"; do
  commit=$(field "$name" commit); url=$(field "$name" url)
  # A file published without a revision of its own gives `pin` instead.
  [ -z "$commit" ] && commit=$(field "$name" pin)
  want=$(field "$name" sha256); strip=$(field "$name" strip)
  paths=$(field "$name" paths)
  format=$(field "$name" format); format=${format:-tar.gz}
  if [ -z "$commit" ] || [ -z "$url" ] || [ -z "$want" ]; then
    echo "fetch-test-data.sh: [$name] needs commit or pin, url and sha256 in $TOML" >&2; exit 2
  fi
  case "$format" in
    tar.gz) ext=tar.gz ;;
    zip)    ext=zip ;;
    file)   ext=${url##*.}; [ "$ext" = "$url" ] && ext=data ;;
    *) echo "fetch-test-data.sh: [$name] unknown format $format" >&2; exit 2 ;;
  esac
  archive="$CACHE/archives/$name-$commit.$ext"
  tree="$CACHE/$name-$commit"

  if [ "$CHECK" = 1 ]; then
    if [ ! -f "$archive" ]; then echo "MISSING $name: $archive"; status=1; continue; fi
    got=$(sha256_of "$archive")
    if [ "$got" != "$want" ]; then echo "BAD     $name: sha256 $got, pinned $want"; status=1; continue; fi
    if [ ! -f "$tree/.izcal-sha256" ] || [ "$(cat "$tree/.izcal-sha256")" != "$want" ]; then
      echo "STALE   $name: $tree is not unpacked from the pinned archive"; status=1; continue
    fi
    echo "OK      $name $commit $tree"
    continue
  fi

  if [ -f "$archive" ] && [ "$(sha256_of "$archive")" = "$want" ]; then
    echo "cached  $name: $archive"
  else
    echo "fetch   $name: $url"
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$archive.part" "$url"
    got=$(sha256_of "$archive.part")
    if [ "$got" != "$want" ]; then
      rm -f "$archive.part"
      echo "fetch-test-data.sh: $name: sha256 $got does not match the pin $want" >&2
      exit 1
    fi
    mv "$archive.part" "$archive"
  fi

  if [ -f "$tree/.izcal-sha256" ] && [ "$(cat "$tree/.izcal-sha256")" = "$want" ]; then
    echo "OK      $name $commit $tree"
    continue
  fi
  # Unpack into a fresh sibling, then swap it in, so a failed unpack never
  # leaves a half tree that looks complete.
  tmp="$tree.unpack.$$"
  rm -rf "$tmp"; mkdir -p "$tmp"
  case "$format" in
    tar.gz)
      if [ -n "$paths" ]; then
        # The members are named under the archive's own top directory,
        # which `strip` then removes; so the patterns are built from it.
        top=$(tar -tzf "$archive" | head -1 | cut -d/ -f1)
        wildcards=""
        for p in $paths; do wildcards="$wildcards $top/$p"; done
        # shellcheck disable=SC2086
        tar -xzf "$archive" -C "$tmp" --strip-components "${strip:-1}" $wildcards
      else
        tar -xzf "$archive" -C "$tmp" --strip-components "${strip:-1}"
      fi
      ;;
    zip)
      if ! command -v unzip >/dev/null 2>&1; then
        rm -rf "$tmp"
        echo "fetch-test-data.sh: $name is a zip and unzip is not installed" >&2; exit 2
      fi
      unzip -q -o "$archive" -d "$tmp"
      ;;
    file) cp "$archive" "$tmp/${url##*/}" ;;
  esac
  printf '%s\n' "$want" > "$tmp/.izcal-sha256"
  rm -rf "$tree"; mv "$tmp" "$tree"
  echo "OK      $name $commit $tree"
done
exit $status
