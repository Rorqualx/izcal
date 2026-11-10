#!/usr/bin/env python3
"""The rlib `cargo build -p hg-heap --message-format=json` just produced.

`tools/heap-ui.sh` compiles its fixtures against this one file. Taking it
from cargo rather than from the newest `libhg_heap-*.rlib` in `deps`
matters because that directory also holds rlibs of other feature sets and
profiles, each with its own hash: the newest by modification time is not
necessarily the one the run just built, and compiling the fixtures
against another revision of the crate would make every result meaningless
while the script still printed a verdict.

Prints the path, or nothing (which the caller treats as a hard failure).
"""

import json
import sys


def rlibs(path):
    """Every rlib cargo reported for hg-heap's own lib target."""
    found = []
    with open(path, encoding="utf-8") as log:
        for line in log:
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                message = json.loads(line)
            except ValueError:
                continue
            if message.get("reason") != "compiler-artifact":
                continue
            target = message.get("target") or {}
            # Cargo reports the crate name with underscores here
            # (`hg_heap`), so the package is identified by its manifest
            # path, which is the name as the directory spells it.
            if "/crates/hg-heap/Cargo.toml" not in (
                message.get("manifest_path") or ""
            ).replace("\\", "/"):
                continue
            if "lib" not in (target.get("kind") or []):
                continue
            found += [
                name
                for name in (message.get("filenames") or [])
                if name.endswith(".rlib")
            ]
    return found


def main():
    if len(sys.argv) != 2:
        print("usage: heap-ui-rlib.py <cargo-json-log>", file=sys.stderr)
        return 2
    found = rlibs(sys.argv[1])
    # Two would mean two feature sets or profiles in one build, and
    # picking either silently would be exactly the guess this avoids.
    if len(found) != 1:
        print(
            f"heap-ui-rlib.py: cargo named {len(found)} hg-heap rlibs, expected 1",
            file=sys.stderr,
        )
        return 1
    print(found[0])
    return 0


if __name__ == "__main__":
    sys.exit(main())
