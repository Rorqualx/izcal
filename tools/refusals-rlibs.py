#!/usr/bin/env python3
"""The rlibs a `cargo build --message-format=json` run just produced.

`tools/refusals.sh` compiles its fixtures against these exact files.
Taking them from cargo rather than from the newest `lib<crate>-*.rlib` in
`deps` matters because that directory also holds rlibs of other feature
sets and profiles, each with its own hash: the newest by modification
time is not necessarily the one the run just built, and compiling the
fixtures against another revision of a crate would make every result
meaningless while the script still printed a verdict.

Usage: refusals-rlibs.py <cargo-json-log> <package>...

Prints one `package=/path/to/lib….rlib` line per package, in the order
asked. Exits non-zero, having printed nothing, if any package has no rlib
or more than one (two would mean two feature sets or profiles in one
build, and picking either silently would be exactly the guess this
avoids).
"""

import json
import pathlib
import sys


def artifacts(log_path):
    """{package directory name: [rlib, ...]} for every lib target built."""
    found = {}
    with open(log_path, encoding="utf-8") as log:
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
            if "lib" not in (target.get("kind") or []):
                continue
            # Cargo reports the crate name with underscores here, so the
            # package is identified by its manifest path instead: the
            # directory name is the package name as Cargo.toml spells it.
            manifest = (message.get("manifest_path") or "").replace("\\", "/")
            if not manifest.endswith("/Cargo.toml"):
                continue
            package = pathlib.PurePosixPath(manifest).parent.name
            rlibs = [
                name
                for name in (message.get("filenames") or [])
                if name.endswith(".rlib")
            ]
            found.setdefault(package, []).extend(rlibs)
    return found


def main():
    if len(sys.argv) < 3:
        print(
            "usage: refusals-rlibs.py <cargo-json-log> <package>...",
            file=sys.stderr,
        )
        return 2
    log, wanted = sys.argv[1], sys.argv[2:]
    found = artifacts(log)
    lines, bad = [], []
    for package in wanted:
        rlibs = found.get(package) or []
        if len(rlibs) != 1:
            bad.append(f"{package}: cargo named {len(rlibs)} rlibs, expected 1")
            continue
        lines.append(f"{package}={rlibs[0]}")
    if bad:
        for line in bad:
            print(f"refusals-rlibs.py: {line}", file=sys.stderr)
        return 1
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())
