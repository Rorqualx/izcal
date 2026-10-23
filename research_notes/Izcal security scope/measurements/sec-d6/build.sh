#!/bin/bash
# Build the probe four ways from one source, varying only LC_BUILD_VERSION.
#   sdk262      : real SDK 26.2, minos 14.0, sdk 26.2 (what Xcode 26.3 produces)
#   sdk27-ld    : real SDK 26.2 headers/libs, ld told -platform_version macos 14.0 27.0
#   sdk27-min27 : as above but minos 27.0 too
#   sdk27-vtool : sdk262 binary rewritten with vtool to sdk 27.0, re-signed ad hoc
#   sdk28-ld    : sdk field 28.0 (checks whether a gate is ">= 27" vs "== 27")
# No SDK 27 is installed on this host (Xcode 26.3 / 17C528 ships MacOSX26.2.sdk only).
set -euo pipefail
cd "$(dirname "$0")" || exit 1
SDK=$(xcrun --sdk macosx --show-sdk-path)
CC="xcrun clang"
CFLAGS="-isysroot $SDK -arch arm64 -O1 -Wall -Wextra -Wno-unused-parameter"
mkdir -p bin
set -x
$CC $CFLAGS -mmacosx-version-min=14.0 -o bin/probe-sdk262 src/probe.c
$CC $CFLAGS -mmacosx-version-min=14.0 -Wl,-platform_version,macos,14.0,27.0 -o bin/probe-sdk27-ld src/probe.c
$CC $CFLAGS -mmacosx-version-min=27.0 -Wl,-platform_version,macos,27.0,27.0 -o bin/probe-sdk27-min27 src/probe.c
$CC $CFLAGS -mmacosx-version-min=14.0 -Wl,-platform_version,macos,14.0,28.0 -o bin/probe-sdk28-ld src/probe.c
xcrun vtool -set-build-version macos 14.0 27.0 -replace -output bin/probe-sdk27-vtool bin/probe-sdk262
codesign -s - -f bin/probe-sdk27-vtool
set +x
for b in bin/probe-*; do echo "== $b"; xcrun vtool -show-build "$b" | grep -E 'platform|minos|sdk'; codesign -dv "$b" 2>&1 | grep -E 'Signature|flags'; done
