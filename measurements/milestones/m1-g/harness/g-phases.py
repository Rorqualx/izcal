# M1 phase G: summarises hg-launch-tool measure-phases output (Windows FP-27 breakdown).
# Per kind: the median, p10 and p90 of each step between consecutive stamps, and the
# cumulative median. Usage: python3 -I g-phases.py PHASES.tsv [RUNS_TO_SKIP, default 1]
import sys,statistics as st,collections
rows=[l.rstrip('\n').split('\t') for l in open(sys.argv[1]) if not l.startswith('run')]
skip=int(sys.argv[2]) if len(sys.argv)>2 else 1
order=['request','sweep','profile','token','job','pipes','create_process','job_assign','spawned','given','worker','hello','resumed','launched','entry','untrusted','link','entropy','executor','ipc_sid','ipc_hello','ipc_io_thread','substrate']
d=collections.defaultdict(lambda: collections.defaultdict(list))
ready=collections.defaultdict(list)
for r in rows:
    run=int(r[0])
    if run<=skip: continue
    if r[4]=='ready': ready[r[1]].append(float(r[5])); continue
    d[r[1]][r[4]].append(float(r[5]))
names={'izk1':'Renderer','izk4':'NetworkPartition'}
for k in d:
    print(f"== {names.get(k,k)} (n={len(d[k]['request'])})   step median ms [p10..p90] / cumulative median")
    prev=None
    for ph in order:
        if ph not in d[k]: continue
        if prev is None: prev=ph; continue
        steps=sorted(b-a for a,b in zip(d[k][prev],d[k][ph]))
        q=lambda p: steps[min(len(steps)-1,int(p*len(steps)))]
        print(f"  {prev:>14} -> {ph:<14} {st.median(steps):8.3f}  [{q(.1):7.3f}..{q(.9):7.3f}]  cum {st.median(d[k][ph]):8.3f}")
        prev=ph
    rk=names.get(k,k); rr=ready.get(rk,[])
    if rr:
        steps=sorted(r-s for r,s in zip(rr,d[k]['substrate']))
        print(f"  {'substrate':>14} -> {'ready(broker)':<14} {st.median(steps):8.3f}  cum {st.median(rr):8.3f}")
