#!/usr/bin/env python3
"""Summarises one results directory: the minimum and the median of each
cell's repeats, and the (a)-to-(b) deltas the note quotes.

    analyse.py results/<rig>

The minimum is the statistic for a latency measurement (the fastest run
is the one with the least interference); the median is printed beside it
so a reader can see the spread the minimum came out of.
"""

import statistics
import sys
from pathlib import Path


def rows(path):
    out = []
    for line in path.read_text().splitlines():
        if not line or line.startswith("#") or line.startswith("PASS"):
            continue
        # A failure line may be one field ("FAIL calibration did not
        # verify") or a tab-separated row, so match the prefix, not the
        # first field: a failure must be reported, never parsed as data.
        if line.startswith("FAIL"):
            out.append(("FAIL", line))
            continue
        f = line.split("\t")
        if f[0] in ("program", "unit"):
            continue
        out.append(("row", f))
    return out


def group(path, keys, value):
    """{key tuple: [values]} from a TSV with a header line."""
    header = path.read_text().splitlines()[0].split("\t")
    idx = {name: i for i, name in enumerate(header)}
    acc = {}
    for kind, f in rows(path):
        if kind == "FAIL":
            print("FAIL row in", path.name, ":", f)
            continue
        k = tuple(f[idx[x]] for x in keys)
        acc.setdefault(k, []).append(float(f[idx[value]]))
    return acc


def table(name, acc, unit):
    print(f"\n== {name} ({unit}: min / median of repeats)")
    width = max(len(" ".join(k)) for k in acc)
    for k in sorted(acc):
        v = acc[k]
        print(f"{' '.join(k):<{width}}  {min(v):9.4f}  {statistics.median(v):9.4f}  n={len(v)}")


def main():
    d = Path(sys.argv[1])
    disp = group(d / "dispatch.tsv", ["program", "streams", "variant"], "ns_per_instr")
    table("dispatch", disp, "ns/instruction")
    ghz = group(d / "dispatch.tsv", ["program"], "ghz")
    print("\n== clock per cell group (GHz): ", {k[0]: (min(v), max(v)) for k, v in ghz.items()})

    print("\n== dispatch deltas against a-trusted (minimum of repeats)")
    for prog in sorted({k[0] for k in disp}):
        for streams in sorted({k[1] for k in disp if k[0] == prog}):
            base = disp.get((prog, streams, "a-trusted"))
            if not base:
                continue
            b = min(base)
            line = [f"{prog} streams={streams} a-trusted={b:.4f}"]
            for v in ("a-trusted-vec", "a-ref-admits", "b-cage-use", "b-cage-revalidate"):
                cell = disp.get((prog, streams, v))
                if cell:
                    line.append(f"{v}={min(cell):.4f} ({(min(cell) / b - 1) * 100:+.1f}%)")
            print("  " + "  ".join(line))

    ent = group(d / "entry.tsv", ["unit", "constants", "variant"], "ns_per_entry")
    table("entry", ent, "ns/entry")

    e2e = group(d / "e2e.tsv", ["program", "place"], "ns_per_job")
    table("e2e", e2e, "ns/job")
    # The interpreter's per-instruction cost: the slope between the two
    # loop counts, which cancels the per-job work.
    for place in ("trusted", "caged"):
        a = e2e.get(("arith1000", place))
        b = e2e.get(("arith2000", place))
        if a and b:
            d_ns = min(b) - min(a)
            d_instr = (2000 * 10 + 6) - (1000 * 10 + 6)
            print(f"  interpreter slope [{place}]: {d_ns / d_instr:.3f} ns/instruction")

    win = group(d / "window.tsv", ["unit", "image_bytes"], "ns_per_insert")
    table("write window (seal)", win, "ns/insert")


if __name__ == "__main__":
    main()
