#!/usr/bin/env bash
# Installs Kani for CI (Linux x86-64), pinned like the other check tools
# (tools/ci-install-tools.sh): the cargo plugin from crates.io with
# --locked (Cargo checks each crate's checksum), and the release bundle
# (CBMC and the Kani compiler) verified against its SHA-256 before
# `cargo kani setup` installs it. The digest is GitHub's asset digest,
# re-hashed after download on 24 October 2026. Setup also installs the
# pinned nightly toolchain through rustup.
set -euo pipefail
version=0.68.0
url=https://github.com/model-checking/kani/releases/download/kani-$version/kani-$version-x86_64-unknown-linux-gnu.tar.gz
sha=32e2b484d73ede0bbf64a2cf0879c4259422497d8aec4ee67d448de9ae7843d3

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cargo install --locked kani-verifier --version "$version"
curl -fsSL -o "$tmp/kani.tar.gz" "$url"
echo "$sha  $tmp/kani.tar.gz" | sha256sum -c --quiet - || { echo "checksum mismatch for the Kani bundle" >&2; exit 1; }
cargo kani setup --use-local-bundle "$tmp/kani.tar.gz"
cargo kani --version
