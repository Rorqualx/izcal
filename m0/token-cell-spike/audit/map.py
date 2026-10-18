import json,re,sys
script=set(l.split()[1] for l in open(sys.argv[2]))
# drop clear false positive
script.discard("V8::GetSharedMemoryStatistics")
hits={}
for l in open(sys.argv[1]):
    d=json.loads(l); t=d["text"]
    m=re.search(r"fn\s+(\w+)",t)
    if not m: continue
    pub=t.lstrip().startswith("pub")
    for c in re.findall(r"\bv8__(\w+?)__(\w+?)(?:__\w+)?\s*\(",t):
        cls=c[0].replace("__","::"); meth=c[1]
        key=None
        for cand in script:
            cc,mm=cand.rsplit("::",1)
            if cc.split("::")[-1]==cls.split("::")[-1] and (meth==mm or meth in (mm+"Index", mm+"WithReceiver", mm+"WithSideEffectType")):
                key=cand
        if key:
            hits.setdefault((d["file"],d["range"]["start"]["line"]+1,m.group(1),pub),set()).add(key)
for k in sorted(hits):
    print(f"{k[0]}:{k[1]}\t{'pub ' if k[3] else 'priv'}\t{k[2]}\t<- {', '.join(sorted(hits[k]))}")
