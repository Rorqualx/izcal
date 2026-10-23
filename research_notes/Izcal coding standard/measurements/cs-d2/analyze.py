#!/usr/bin/env python3
import json, math, os, statistics as st
D = os.path.dirname(os.path.abspath(__file__))
# results.json (17 MB) is not in the repository; pass its path (see README).
import gzip, sys
_p = sys.argv[1] if len(sys.argv) > 1 else os.path.join(D, "raw", "results.json")
R = json.load((gzip.open if _p.endswith(".gz") else open)(_p, "rt"))

def crate(n):
    return n.split("_")[0]

def gm(xs):
    return math.exp(sum(math.log(x) for x in xs) / len(xs))

rows, by = [], {}
print("| benchmark | instr/iter off | instr/iter on | instr Δ% | instr spread (max-min)/med off, on | cycles Δ% | wall median off (µs) | wall median on (µs) | wall Δ% | wall IQR off / on (µs) |")
print("|---|---|---|---|---|---|---|---|---|---|")
for r in R:
    if r.get("skipped"):
        print(f"| {r['name']} | SKIPPED (rig check failed) |"); continue
    io, ii = st.median(r["instr"]["off"]), st.median(r["instr"]["on"])
    so = (max(r["instr"]["off"]) - min(r["instr"]["off"])) / io * 100
    si = (max(r["instr"]["on"]) - min(r["instr"]["on"])) / ii * 100
    co, ci = st.median(r["cycles"]["off"]), st.median(r["cycles"]["on"])
    wo, wi = st.median(r["wall"]["off"]), st.median(r["wall"]["on"])
    qo = st.quantiles(r["wall"]["off"], n=4); qi = st.quantiles(r["wall"]["on"], n=4)
    ratio = ii / io
    by.setdefault(crate(r["name"]), []).append((ratio, ci / co, wi / wo))
    print(f"| {r['name']} | {io:,.0f} | {ii:,.0f} | {100*(ratio-1):+.2f} | {so:.4f}%, {si:.4f}% | {100*(ci/co-1):+.2f} | {wo/1e3:,.2f} | {wi/1e3:,.2f} | {100*(wi/wo-1):+.2f} | {(qo[2]-qo[0])/1e3:,.2f} / {(qi[2]-qi[0])/1e3:,.2f} |")
print()
print("| crate | n | instr geomean Δ% | cycles geomean Δ% | wall geomean Δ% |")
print("|---|---|---|---|---|")
cg = {}
for c, v in by.items():
    g = [gm([x[k] for x in v]) for k in range(3)]
    cg[c] = g
    print(f"| {c} | {len(v)} | {100*(g[0]-1):+.2f} | {100*(g[1]-1):+.2f} | {100*(g[2]-1):+.2f} |")
allv = [x for v in by.values() for x in v]
o1 = [gm([g[k] for g in cg.values()]) for k in range(3)]
o2 = [gm([x[k] for x in allv]) for k in range(3)]
o3c = [c for c in cg if c != "url"]
o3 = [gm([cg[c][k] for c in o3c]) for k in range(3)]
print(f"| overall (geomean of crate geomeans) | {len(cg)} | {100*(o1[0]-1):+.2f} | {100*(o1[1]-1):+.2f} | {100*(o1[2]-1):+.2f} |")
print(f"| overall (geomean of all benchmarks) | {len(allv)} | {100*(o2[0]-1):+.2f} | {100*(o2[1]-1):+.2f} | {100*(o2[2]-1):+.2f} |")
print(f"| renderer-only (crate geomean, excl. url) | {len(o3c)} | {100*(o3[0]-1):+.2f} | {100*(o3[1]-1):+.2f} | {100*(o3[2]-1):+.2f} |")
print()
for r in R:
    print(f"- {r['name']}: `{r['rig_check']}` (exit {r['rig_check_exit']})")
