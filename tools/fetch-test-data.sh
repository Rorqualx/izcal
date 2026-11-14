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

# A content hash of an unpacked tree: every regular file's path and its
# bytes, in C order, hashed into one value.
#
# This exists because a `codeload.github.com` tar.gz is **not** a stable
# identity for a commit. Measured 13 November 2026, on WPT's archive for
# 33926aca: 106,514,027 bytes at 16:16 (sha256 4eda43c0...) and
# 106,513,262 bytes at 17:16 (sha256 3350857a...), both complete, both
# `gzip -t` clean, one hour apart, the same commit -- and `diff -rq`
# between the two unpacked css/CSS2 trees found nothing. GitHub had
# re-generated the archive; the content had not changed.
#
# An archive hash alone therefore fails for a reason that has nothing to
# do with the data, and repinning it proves nothing about what arrived.
# So a set may declare `tree_sha256`: the identity is then the content,
# the archive hash becomes a note rather than a gate, and a set without
# a tree hash keeps the archive hash as its gate, unchanged.
tree_sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then hasher="sha256sum"
  else hasher="shasum -a 256"; fi
  # The per-file lines are normalised to `<hash> <path>` with one space
  # before they are hashed, because GNU coreutils and the BSD/Perl
  # `shasum` separate the two differently; without that the same tree
  # would hash differently on Linux and macOS and the pin would be
  # useless. `.izcal-sha256` is excluded, since it holds the archive pin
  # and would make the content hash depend on it.
  # shellcheck disable=SC2086 # hasher is a command and its flags
  ( cd "$1" && find . -type f ! -name .izcal-sha256 -print0 \
      | LC_ALL=C sort -z \
      | xargs -0 $hasher \
      | awk '{ h = $1; $1 = ""; sub(/^ +/, ""); print h " " $0 }' \
      | $hasher | awk '{print $1}' )
}

status=0
mkdir -p "$CACHE/archives"
for name in "${names[@]}"; do
  commit=$(field "$name" commit); url=$(field "$name" url)
  # A file published without a revision of its own gives `pin` instead.
  [ -z "$commit" ] && commit=$(field "$name" pin)
  want=$(field "$name" sha256); strip=$(field "$name" strip)
  want_tree=$(field "$name" tree_sha256)
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
    if [ "$got" != "$want" ] && [ -z "$want_tree" ]; then
      echo "BAD     $name: sha256 $got, pinned $want"; status=1; continue
    fi
    if [ -n "$want_tree" ]; then
      # The content is the identity, so the marker file's archive pin
      # does not decide: a tree unpacked from a re-compressed archive of
      # the same commit is the same tree.
      if [ ! -d "$tree" ]; then
        echo "MISSING $name: $tree"; status=1; continue
      fi
      got_tree=$(tree_sha256_of "$tree")
      if [ "$got_tree" != "$want_tree" ]; then
        echo "BAD     $name: tree_sha256 $got_tree, pinned $want_tree"; status=1; continue
      fi
    elif [ ! -f "$tree/.izcal-sha256" ] || [ "$(cat "$tree/.izcal-sha256")" != "$want" ]; then
      echo "STALE   $name: $tree is not unpacked from the pinned archive"; status=1; continue
    fi
    echo "OK      $name $commit $tree"
    continue
  fi

  if [ -f "$archive" ] && { [ "$(sha256_of "$archive")" = "$want" ] || [ -n "$want_tree" ]; }; then
    echo "cached  $name: $archive"
  else
    echo "fetch   $name: $url"
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$archive.part" "$url"
    got=$(sha256_of "$archive.part")
    if [ "$got" != "$want" ]; then
      if [ -n "$want_tree" ]; then
        # The content is what this set is pinned by; the archive hash is
        # a note, because GitHub re-compresses archives (above).
        echo "note    $name: archive sha256 $got, pinned $want; checking tree_sha256"
      else
        rm -f "$archive.part"
        echo "fetch-test-data.sh: $name: sha256 $got does not match the pin $want" >&2
        exit 1
      fi
    fi
    mv "$archive.part" "$archive"
  fi

  if [ -f "$tree/.izcal-sha256" ] \
     && { [ "$(cat "$tree/.izcal-sha256")" = "$want" ] \
          || { [ -n "$want_tree" ] && [ "$(tree_sha256_of "$tree")" = "$want_tree" ]; }; }; then
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
        # `head -1` closes the pipe, and under `pipefail` a tar that
        # dies of SIGPIPE on a 1.5 GB listing would abort the script.
        # bsdtar does not, GNU tar can, so pipefail is turned off for
        # this one substitution rather than left to the platform.
        top=$(set +o pipefail; tar -tzf "$archive" | head -1 | cut -d/ -f1)
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
  if [ -n "$want_tree" ]; then
    got_tree=$(tree_sha256_of "$tmp")
    if [ "$got_tree" != "$want_tree" ]; then
      rm -rf "$tmp"
      echo "fetch-test-data.sh: $name: tree_sha256 $got_tree does not match the pin $want_tree" >&2
      exit 1
    fi
  fi
  printf '%s\n' "$want" > "$tmp/.izcal-sha256"
  rm -rf "$tree"; mv "$tmp" "$tree"
  echo "OK      $name $commit $tree"
done
exit $status
