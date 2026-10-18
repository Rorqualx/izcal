#!/bin/sh
# Rebuilds upstream/work/v8 (a patched copy of the v8-152.2.0 crate: big
# directories are symlinked to the registry copy) and runs upstream/verify.
# Requires the prebuilt: set RUSTY_V8_ARCHIVE or let rusty_v8 download it.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
src=$(ls -d "${CARGO_HOME:-$HOME/.cargo}"/registry/src/*/v8-152.2.0 | head -1)
rm -rf "$here/work/v8"
mkdir -p "$here/work/v8"
for f in "$src"/* "$src"/.gn "$src"/.cargo_vcs_info.json; do
  b=$(basename "$f")
  case "$b" in
    v8|third_party|build|buildtools|gen|tools) ln -s "$f" "$here/work/v8/$b" ;;
    *) cp -R "$f" "$here/work/v8/$b" ;;
  esac
done
(cd "$here/work/v8" && patch -p1 < "$here/rusty_v8-152.2.0-cppgc-ext.patch")
cd "$here/verify"
cargo test --release -- --test-threads=1 "$@"
