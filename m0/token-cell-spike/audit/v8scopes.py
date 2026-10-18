import re,sys,collections
src=open(sys.argv[1]).read().split("\n")
cur=None; res=collections.OrderedDict()
sig=re.compile(r"^(?:[A-Za-z_][\w:<>,\s\*&]*?\s)?(?:v8::)?([A-Z]\w*(?:::\w+)*)::(~?\w+)\(")
for i,l in enumerate(src):
    m=sig.match(l)
    if m and not l.startswith(" ") and not l.startswith("#"):
        cur=m.group(1)+"::"+m.group(2)
    for k in ["PrepareForExecutionScope","EnterV8Scope","EnterV8NoScriptScope","EnterV8NoScriptNoExceptionScope","PrepareForDebugInterfaceExecutionScope","CallDepthScope"]:
        if re.search(r"\b"+k+r"\b",l) and cur:
            res.setdefault(cur,set()).add(k)
for f,ks in res.items():
    if ks & {"PrepareForExecutionScope","EnterV8Scope","PrepareForDebugInterfaceExecutionScope"} or ("CallDepthScope" in ks and not ks & {"EnterV8NoScriptScope","EnterV8NoScriptNoExceptionScope"}):
        print("SCRIPT", f, ",".join(sorted(ks)))
