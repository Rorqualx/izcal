"""Analyze CS-D5 gungraun artifacts.

usage: python3 -I analyze_cs_d5.py DATA_DIR JOBS_JSON...
DATA_DIR/<run_id>/results-<os>-<rep>/... from gh run download.
JOBS_JSON: `gh api .../runs/ID/jobs --paginate` output (step timings).
"""
import json, os, sys, statistics, glob
from datetime import datetime

data_dir = sys.argv[1]
job_files = sys.argv[2:]
PHASES = ["base", "head", "head2", "cachesim"]
METRICS = ["Ir", "Dr", "Dw", "I1mr", "D1mr", "D1mw", "ILmr", "DLmr", "DLmw",
           "L1hits", "LLhits", "RamHits", "TotalRW", "EstimatedCycles"]

rows = []  # (run, os, rep, phase, bench, metric, new, old)
meta = {}
for run in sorted(os.listdir(data_dir)):
    rdir = os.path.join(data_dir, run)
    if not os.path.isdir(rdir) or not run.isdigit():
        continue
    for art in sorted(os.listdir(rdir)):
        adir = os.path.join(rdir, art)
        osname, rep = art[len("results-"):].rsplit("-", 1)
        mp = os.path.join(adir, "meta.txt")
        if os.path.exists(mp):
            meta[(run, osname, rep)] = dict(
                l.split(": ", 1) for l in open(mp).read().splitlines()
                if ": " in l and not l.startswith("lscpu"))
        for ph in PHASES:
            for f in glob.glob(os.path.join(adir, ph, "**", "summary.json"), recursive=True):
                d = json.load(open(f))
                tot = d["profiles"][0]["data"]["total"]["metrics"]
                for k in METRICS:
                    if k in tot:
                        v = tot[k]["values"]
                        rows.append((run, osname, rep, ph, d["function_name"], k, v.get("new"), v.get("old")))

def spread(vals):
    med = statistics.median(vals)
    return (max(vals) - min(vals)) / med * 100 if med else 0.0

oses = sorted({r[1] for r in rows})
benches = sorted({r[4] for r in rows})
print("## Cross-job spread, head phase: (max-min)/median %")
for o in oses:
    print(f"\n### {o}")
    print("| bench | metric | n | min | median | max | spread % | distinct |")
    print("|---|---|---|---|---|---|---|---|")
    for b in benches:
        for k in METRICS:
            vals = [r[6] for r in rows if r[1] == o and r[4] == b and r[5] == k and r[3] == "head"]
            if vals:
                print(f"| {b} | {k} | {len(vals)} | {min(vals)} | {statistics.median(vals)} | {max(vals)} | {spread(vals):.4f} | {len(set(vals))} |")

print("\n## Cross-job spread over all phases, per metric (distinct values / spread %)")
for o in oses:
    for b in benches:
        parts = []
        for k in METRICS:
            vals = [r[6] for r in rows if r[1] == o and r[4] == b and r[5] == k]
            if vals:
                parts.append(f"{k}:{len(set(vals))}/{spread(vals):.4f}")
        n = len([r for r in rows if r[1] == o and r[4] == b and r[5] == "Ir"])
        print(f"{o} {b} n={n}: " + " ".join(parts))

print("\n## Same-job diffs vs base")
for ph in ["head", "head2"]:
    for o in oses:
        for b in benches + ["ALL"]:
            for k in ["Ir", "EstimatedCycles"]:
                diffs = [(r[6] - r[7]) / r[7] * 100 if r[7] else 0.0 for r in rows
                         if r[1] == o and r[3] == ph and r[5] == k and r[7] is not None and (b == "ALL" or r[4] == b)]
                if diffs:
                    over = sum(1 for x in diffs if abs(x) > 1.0)
                    nz = sum(1 for x in diffs if x != 0)
                    print(f"{ph} vs base | {o} | {b} | {k} | n={len(diffs)} nonzero={nz} max|diff|={max(abs(x) for x in diffs):.5f}% >1%={over}")

print("\n## Hosts")
hosts = {}
for (run, o, rep), m in sorted(meta.items()):
    hosts.setdefault((o, m.get("cpu", "?"), m.get("image", "?"), m.get("kernel", "?"), m.get("valgrind", "?"), m.get("rustc", "?")), []).append(run)
for k, v in hosts.items():
    print(k, len(v), {r: v.count(r) for r in set(v)})

print("\n## Distinct Ir per CPU model (head phase)")
for o in oses:
    for b in benches:
        by = {}
        for r in rows:
            if r[1] == o and r[4] == b and r[5] == "Ir" and r[3] == "head":
                by.setdefault(meta.get((r[0], r[1], r[2]), {}).get("cpu", "?"), set()).add(r[6])
        print(f"{o} {b}: " + "; ".join(f"{cpu}: {sorted(s)}" for cpu, s in sorted(by.items())))

print("\n## Raw Ir (head phase) per job")
print("| run | os | rep | cpu | " + " | ".join(benches) + " |")
print("|---|---|---|---|" + "---|" * len(benches))
for (run, o, rep), m in sorted(meta.items(), key=lambda x: (x[0][0], x[0][1], int(x[0][2]))):
    vals = []
    for b in benches:
        v = [r[6] for r in rows if (r[0], r[1], r[2]) == (run, o, rep) and r[4] == b and r[5] == "Ir" and r[3] == "head"]
        vals.append(str(v[0]) if v else "-")
    print(f"| {run} | {o} | {rep} | {m.get('cpu', '?')} | " + " | ".join(vals) + " |")

def ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))

def p90(v):
    v = sorted(v)
    return v[min(len(v) - 1, int(round(0.9 * (len(v) - 1))))]

print("\n## Step wall times (s), GitHub jobs API")
steps_by, totals, concl = {}, {}, {}
for jf in job_files:
    txt = open(jf).read()
    dec = json.JSONDecoder(); i = 0
    while True:
        rest = txt[i:].lstrip()
        if not rest:
            break
        o, n = dec.raw_decode(rest); i = len(txt) - len(rest) + n
        for j in o["jobs"]:
            osn = j["name"].split("(")[1].split(",")[0]
            concl.setdefault(osn, []).append(j["conclusion"])
            if j["started_at"] and j["completed_at"]:
                totals.setdefault(osn, []).append((ts(j["completed_at"]) - ts(j["started_at"])).total_seconds())
            for s in j.get("steps", []):
                if s.get("started_at") and s.get("completed_at"):
                    steps_by.setdefault((osn, s["name"]), []).append((ts(s["completed_at"]) - ts(s["started_at"])).total_seconds())
for osn in sorted(totals):
    print(f"\n### {osn}: jobs={len(totals[osn])} conclusions={ {c: concl[osn].count(c) for c in set(concl[osn])} }")
    print("| step | n | median | p90 | max |")
    print("|---|---|---|---|---|")
    for (o, sname), v in steps_by.items():
        if o == osn:
            print(f"| {sname[:60]} | {len(v)} | {statistics.median(v):.0f} | {p90(v):.0f} | {max(v):.0f} |")
    v = totals[osn]
    print(f"| **job total** | {len(v)} | {statistics.median(v):.0f} | {p90(v):.0f} | {max(v):.0f} |")
