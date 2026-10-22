#!/usr/bin/env bash
# CS-D5 measurement steps, one subcommand per workflow step so GitHub's step
# timings give the setup, build and run costs separately.
#
#   ci.sh setup                 valgrind, pinned toolchain, gungraun-runner
#   ci.sh build <base|head>     copy the crate to $WS/<copy>, cargo bench --no-run
#   ci.sh bench <phase> <copy> [gungraun args...]
#                               run the benchmarks, keep summary.json files
set -euo pipefail

TOOLCHAIN=1.99.1
RUNNER_VERSION=0.20.0
SRC=$(cd "$(dirname "${BASH_SOURCE[0]}")/bench" && pwd)
WS=${RUNNER_TEMP:?}/ws
OUT=${RUNNER_TEMP:?}/results
# One target directory for both copies: the dependencies build once, the
# benchmark crate builds once per copy (different paths), as in a real gate.
export CARGO_TARGET_DIR=$WS/target
mkdir -p "$WS" "$OUT"

stamp() { echo "$1 $(date +%s.%N)" >> "$OUT/stamps.txt"; }

case "$1" in
  setup)
    stamp setup-start
    sudo apt-get update -qq
    sudo apt-get install -y -qq valgrind > /dev/null
    stamp valgrind-done
    rustup toolchain install "$TOOLCHAIN" --profile minimal --no-self-update
    rustup default "$TOOLCHAIN"
    stamp toolchain-done
    case "$(uname -m)" in
      x86_64) triple=x86_64-unknown-linux-gnu; sha=b3b39d4243e43597512a1f79016a1f38cbf90e5869c0c2535fe4097c69ed91aa ;;
      aarch64) triple=aarch64-unknown-linux-gnu; sha=30ddf6737e486d1d3c11746eb603d8a228afeb1614dcf84ebe9ee1442f59d2f3 ;;
      *) echo "unsupported arch $(uname -m)" >&2; exit 1 ;;
    esac
    name=gungraun-runner-v$RUNNER_VERSION-$triple
    tmp=$(mktemp -d)
    curl -fsSL -o "$tmp/r.tar.gz" "https://github.com/gungraun/gungraun/releases/download/v$RUNNER_VERSION/$name.tar.gz"
    echo "$sha  $tmp/r.tar.gz" | sha256sum -c --quiet -
    tar -xzf "$tmp/r.tar.gz" -C "$tmp" "$name/gungraun-runner"
    mkdir -p "$RUNNER_TEMP/bin"
    install -m 0755 "$tmp/$name/gungraun-runner" "$RUNNER_TEMP/bin/gungraun-runner"
    echo "$RUNNER_TEMP/bin" >> "${GITHUB_PATH:?}"
    stamp setup-done
    {
      echo "image: ${ImageOS:-?} ${ImageVersion:-?}"
      echo "runner: ${RUNNER_NAME:-?} ${RUNNER_ARCH:-?}"
      echo "kernel: $(uname -r)"
      echo "cpu: $(grep -m1 -E 'model name|CPU part' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//')"
      lscpu | grep -E '^(Model name|Vendor ID|CPU\(s\)|Flags|L1d|L2|L3)' | sed 's/^/lscpu: /' | cut -c1-400
      echo "nproc: $(nproc)"
      echo "valgrind: $(valgrind --version)"
      echo "rustc: $(rustc --version)"
      echo "glibc: $(ldd --version | head -1)"
    } > "$OUT/meta.txt"
    cat "$OUT/meta.txt"
    ;;
  build)
    copy=$2
    rm -rf "${WS:?}/$copy"
    mkdir -p "$WS/$copy"
    cp -R "$SRC/." "$WS/$copy/"
    stamp "build-$copy-start"
    (cd "$WS/$copy" && cargo bench --locked --no-run)
    stamp "build-$copy-done"
    ;;
  bench)
    phase=$2 copy=$3
    shift 3
    stamp "bench-$phase-start"
    (cd "$WS/$copy" && cargo bench --locked --bench kernels -- --save-summary=pretty-json "$@") 2>&1 | tee "$OUT/$phase.log"
    stamp "bench-$phase-done"
    mkdir -p "$OUT/$phase"
    (cd "$CARGO_TARGET_DIR/gungraun" && find . -name summary.json -exec cp --parents {} "$OUT/$phase/" \;)
    ;;
  *) echo "usage: ci.sh setup | build COPY | bench PHASE COPY [ARGS]" >&2; exit 2 ;;
esac
