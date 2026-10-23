#!/usr/bin/env python3
"""CS-D2 proxy orchestrator (runs on the Mac, drives revire over ssh).
For every workload: `rig check` (kept as evidence), then instruction counts
(sudo perf stat, delta between N iterations and 0 iterations, 3 reps per build),
then wall time (3 interleaved processes x 15 samples per build)."""
import json, subprocess, sys, time, os

HOST = "joe@192.168.50.9"
ROOT = "~/izcal-measure/cs-d2"
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "raw")
os.makedirs(OUT, exist_ok=True)
CPU = "4"

# name, N for instruction counting, batch for wall time, warmup calls
W = [
    ("cssparser_tokenize_bootstrap", 200, 5, 5),
    ("cssparser_tokenize_bulma", 100, 3, 3),
    ("stylo_parse_bootstrap", 50, 2, 3),
    ("stylo_parse_bulma", 25, 1, 2),
    ("stylo_stylist_bootstrap", 300, 10, 10),
    ("stylo_stylist_bulma", 200, 8, 8),
    ("selectors_match_bootstrap_navbar", 2, 1, 1),
    ("html5ever_treebuild_wikipedia", 50, 2, 3),
    ("html5ever_treebuild_navbar", 40, 2, 3),
    ("html5ever_tokenize_lipsum", 50000, 5000, 1000),
    ("html5ever_tokenize_lipsum_zh", 100000, 10000, 1000),
    ("html5ever_tokenize_medium_fragment", 5000, 300, 100),
    ("html5ever_tokenize_small_fragment", 50000, 3000, 1000),
    ("html5ever_tokenize_tiny_fragment", 200000, 10000, 1000),
    ("html5ever_tokenize_strong", 5000, 400, 100),
    ("url_join_wikipedia", 500, 30, 10),
    ("url_parse_own_bench_strings", 100, 5, 3),
]
if len(sys.argv) > 1:
    W = [w for w in W if w[0] in sys.argv[1:]]


def sh(cmd, **kw):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)


def remote(cmd):
    r = sh(f"ssh {HOST} {json.dumps(cmd)}")
    return r.stdout, r.stderr


def perf(build, w, n):
    cmd = (f"cd {ROOT} && sudo perf stat -x, -e instructions:u,cycles:u -- "
           f"taskset -c {CPU} target-{build}/release/csd2-proxy inputs {w} {n} 0 1")
    so, se = remote(cmd)
    vals = {}
    for line in se.splitlines():
        p = line.split(",")
        if len(p) > 3 and p[2].startswith(("instructions", "cycles")):
            vals[p[2].split(":")[0]] = int(p[0])
    return vals, so.strip(), se


results = []
for name, n, batch, warm in W:
    chk = sh("rig check revire --kind exclusive --session browser-61")
    ev = (chk.stdout + chk.stderr).strip()
    print(f"[{time.strftime('%H:%M:%S')}] {name}: rig check exit={chk.returncode}: {ev}", flush=True)
    rec = {"name": name, "n": n, "batch": batch, "warm": warm, "rig_check": ev,
           "rig_check_exit": chk.returncode, "instr": {"off": [], "on": []},
           "cycles": {"off": [], "on": []}, "wall": {"off": [], "on": []}, "raw": []}
    if chk.returncode != 0:
        rec["skipped"] = True
        results.append(rec)
        continue
    for rep in range(3):
        for b in (("off", "on") if rep % 2 == 0 else ("on", "off")):
            full, so1, se1 = perf(b, name, n)
            zero, so0, se0 = perf(b, name, 0)
            rec["raw"].append({"build": b, "N": n, "full": full, "zero": zero, "out": so1})
            if "instructions" in full and "instructions" in zero:
                rec["instr"][b].append((full["instructions"] - zero["instructions"]) / n)
                rec["cycles"][b].append((full["cycles"] - zero["cycles"]) / n)
            else:
                print("perf failed:", se1[-500:], se0[-500:], flush=True)
    for rep in range(3):
        for b in (("off", "on") if rep % 2 == 0 else ("on", "off")):
            so, se = remote(f"cd {ROOT} && taskset -c {CPU} target-{b}/release/csd2-proxy "
                            f"inputs {name} 15 {warm} {batch}")
            line = so.strip()
            rec["raw"].append({"build": b, "wall": line})
            raw = line.split("raw=")[-1]
            rec["wall"][b].extend(float(x) for x in raw.split(","))
    io, ii = rec["instr"]["off"], rec["instr"]["on"]
    if io and ii:
        print(f"   instr/iter off={io} on={ii} delta={100*(sorted(ii)[1]/sorted(io)[1]-1):+.3f}%", flush=True)
    results.append(rec)
    with open(os.path.join(OUT, "results.json"), "w") as f:
        json.dump(results, f, indent=1)
print("ALL DONE", flush=True)
