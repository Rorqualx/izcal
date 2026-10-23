#!/usr/bin/env bash
# Installs the check tools for CI (Linux x86-64) into $1 (default
# $RUNNER_TEMP/bin), each pinned to a version and verified against its
# SHA-256 before it is unpacked. Used by both repositories' check workflows,
# so the pins live in one place. Digests were checked on 18 October 2026
# against the projects' published checksum files and GitHub's asset digests
# (ast-grep 0.51.0: GitHub's asset digest, re-hashed after download).
set -euo pipefail
bin=${1:-${RUNNER_TEMP:?}/bin}
mkdir -p "$bin"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# name|url|sha256|path of the binary inside the archive
tools=(
  "gitleaks|https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz|551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb|gitleaks"
  "actionlint|https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz|8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8|actionlint"
  "zizmor|https://github.com/zizmorcore/zizmor/releases/download/v1.31.1/zizmor-x86_64-unknown-linux-gnu.tar.gz|d185f726bdaf92df78273217dfdd24ebcd6b0ef48cf3ce776ff5762b6e138c99|zizmor"
  "cargo-deny|https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz|9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f|cargo-deny-0.20.2-x86_64-unknown-linux-musl/cargo-deny"
  "ast-grep|https://github.com/ast-grep/ast-grep/releases/download/0.51.0/app-x86_64-unknown-linux-gnu.zip|a7eead5552d3a8daad70f5d50d935da44bd0c138e946b5b0a96b1d38169fa639|ast-grep"
)

for t in "${tools[@]}"; do
  IFS='|' read -r name url sha inner <<< "$t"
  curl -fsSL -o "$tmp/$name.archive" "$url"
  echo "$sha  $tmp/$name.archive" | sha256sum -c --quiet - || { echo "checksum mismatch for $name" >&2; exit 1; }
  case "$url" in
    *.zip) unzip -q -o "$tmp/$name.archive" "$inner" -d "$tmp" ;;
    *) tar -xzf "$tmp/$name.archive" -C "$tmp" "$inner" ;;
  esac
  install -m 0755 "$tmp/$inner" "$bin/$name"
  echo "installed $name ($("$bin/$name" --version 2>/dev/null | head -1 || echo version unknown))"
done
