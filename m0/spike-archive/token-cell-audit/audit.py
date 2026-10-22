"""CS-D3 criterion (2): regenerate the list of rusty_v8 methods that can run
script and check it against clippy's `disallowed-methods`.

  python3 -I audit.py RUSTY_V8_DIR API_CC CLIPPY_TOML REVIEWED_TOML [OUT_DIR]

1. V8 side: api.cc functions that open `EnterV8Scope`,
   `PrepareForExecutionScope` or `PrepareForDebugInterfaceExecutionScope`, or a
   `CallDepthScope` without a no-script scope (V8's own "may run script"
   marker).
2. Rust side (ast-grep): rusty_v8 functions that call the binding of one of
   those, then, to a fixpoint, functions that call such a function on the same
   type (`self.f(`, `Self::f(`, `Type::f(`) or a free function in the same file.
3. Every public entry found must be in `disallowed-methods` or in the reviewed
   file's `[allow]` table with a reason. Every `v8::` path in either list must
   still exist. Exit 1 on any miss.
"""
import collections
import json
import os
import re
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
RUN = {"PrepareForExecutionScope", "EnterV8Scope", "PrepareForDebugInterfaceExecutionScope"}
NOSCRIPT = {"EnterV8NoScriptScope", "EnterV8NoScriptNoExceptionScope"}
SCOPES = RUN | NOSCRIPT | {"CallDepthScope"}


def v8_script_entries(api_cc):
    sig = re.compile(r"^(?:[A-Za-z_][\w:<>,\s\*&]*?\s)?(?:v8::)?([A-Z]\w*(?:::\w+)*)::(~?\w+)\("
    cur, found = None, collections.OrderedDict()
    for line in open(api_cc, encoding="utf-8", errors="replace"):
        m = sig.match(line)
        if m and not line.startswith((" ", "#")):
            cur = m.group(1) + "::" + m.group(2)
        if cur:
            for k in SCOPES:
                if re.search(r"\b" + k + r"\b", line):
                    found.setdefault(cur, set()).add(k)
    out = {}
    for f, ks in found.items():
        if ks & RUN or ("CallDepthScope" in ks and not ks & NOSCRIPT):
            out[f] = ks
    # A statistics getter that opens a CallDepthScope but never runs script.
    out.pop("V8::GetSharedMemoryStatistics", None)
    return out


def sg(rule, src):
    r = subprocess.run(
        ["ast-grep", "scan", "-r", rule, "--json=stream", src],
        capture_output=True, text=True, check=True)
    return [json.loads(l) for l in r.stdout.splitlines() if l.strip()]


def impl_type(text):
    head = text.split("{", 1)[0]
    head = re.sub(r"^\s*(unsafe\s+)?impl\s*", "", head)
    head = re.sub(r"^<.*?>\s*", "", head, count=1) if head.startswith("<") else head
    if re.search(r"\bfor\b", head):
        return None  # trait impls: not script-running inherent methods
    m = re.match(r"\s*(?:crate::)?(?:\w+::)*([A-Z]\w*)", head)
    return m.group(1) if m else None


def strip_generics_prefix(text):
    # `impl<'s, T: Foo<Bar>> X<..>`: drop the balanced `<...>` after `impl`.
    t = re.sub(r"^\s*(unsafe\s+)?impl", "", text, count=1)
    if t.startswith("<"):
        depth = 0
        for i, c in enumerate(t):
            depth += c == "<"
            depth -= c == ">"
            if depth == 0:
                return "impl " + t[i + 1:]
    return "impl " + t


def main():
    rv8, api_cc, clippy_toml, reviewed_toml = sys.argv[1:5]
    out_dir = sys.argv[5] if len(sys.argv) > 5 else None
    src = os.path.join(rv8, "src")
    script = v8_script_entries(api_cc)
    by_short = collections.defaultdict(set)  # (Class, Method) -> C++ names
    for full in script:
        cls, meth = full.rsplit("::", 1)
        by_short[(cls.split("::")[-1], meth)].add(full)

    pub_mods = set(re.findall(r"^pub mod (\w+);", open(os.path.join(src, "lib.rs")).read(), re.M))
    impls = collections.defaultdict(list)
    for d in sg(os.path.join(HERE, "impls.yml"), src):
        impls[d["file"]].append((d["byteOffset"] if "byteOffset" in d else d["range"]["byteOffset"],
                                 impl_type(strip_generics_prefix(d["text"]))))

    fns = []
    for d in sg(os.path.join(HERE, "fns.yml"), src):
        m = re.search(r"\bfn\s+(\w+)", d["text"])
        if not m:
            continue
        off = d["range"]["byteOffset"]["start"]
        owner = None
        best = None
        for rng, ty in impls[d["file"]]:
            if rng["start"] <= off < rng["end"] and (best is None or rng["start"] > best):
                best, owner = rng["start"], ty
        in_impl = best is not None
        if in_impl and owner is None:
            continue  # trait impl
        stem = os.path.splitext(os.path.basename(d["file"]))[0]
        mod = f"v8::{stem}::" if stem in pub_mods else "v8::"
        path = f"{mod}{owner}::{m.group(1)}" if in_impl else f"{mod}{m.group(1)}"
        sig = d["text"].split("{", 1)[0]
        fns.append({
            "file": os.path.relpath(d["file"], rv8), "line": d["range"]["start"]["line"] + 1,
            "name": m.group(1), "owner": owner, "path": path,
            "pub": d["text"].lstrip().startswith("pub ") and "pub(crate)" not in sig[:12],
            "body": d["text"], "why": set(),
        })

    # Direct: the function calls the binding of a script-running C++ function.
    for f in fns:
        for cls, meth in re.findall(r"\bv8__(\w+?)__(\w+?)(?:__\w+)?\s*\(", f["body"]):
            cls = cls.replace("__", "::").split("::")[-1]
            for (c, mm), fulls in by_short.items():
                sfx = ("", "Index", "WithReceiver", "WithSideEffectType")
                if c == cls and (meth in [mm + x for x in sfx] or mm in [meth + x for x in sfx]):
                    f["why"] |= fulls
    # Transitive, to a fixpoint.
    changed = True
    while changed:
        changed = False
        hot = [g for g in fns if g["why"]]
        for f in fns:
            for g in hot:
                if g is f or f["why"] >= {"via " + g["path"]}:
                    continue
                n = re.escape(g["name"])
                if g["owner"] and g["owner"] == f["owner"]:
                    pat = rf"(\bself\s*\.|\bSelf::|\b{re.escape(g['owner'])}::){n}\s*(::<[^>]*>)?\("
                elif not g["owner"] and g["file"] == f["file"] and not f["owner"]:
                    pat = rf"(?<![.\w:]){n}\s*\("
                else:
                    continue
                body = f["body"].split("{", 1)[1] if "{" in f["body"] else ""
                if re.search(pat, body):
                    f["why"].add("via " + g["path"])
                    changed = True

    entries = {}
    for f in fns:
        if f["why"] and f["pub"]:
            entries.setdefault(f["path"], f)
    all_paths = {f["path"] for f in fns if f["pub"]}

    clippy = tomllib.load(open(clippy_toml, "rb")).get("disallowed-methods", [])
    listed = {e["path"] if isinstance(e, dict) else e for e in clippy}
    reviewed = tomllib.load(open(reviewed_toml, "rb"))
    allowed = {a["path"]: a["reason"] for a in reviewed.get("allow", [])}

    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
        with open(os.path.join(out_dir, "v8_script_entry.txt"), "w") as o:
            for k, ks in script.items():
                o.write(f"SCRIPT {k} {','.join(sorted(ks))}\n")
        with open(os.path.join(out_dir, "rust_entry.txt"), "w") as o:
            for p in sorted(entries):
                f = entries[p]
                o.write(f"{f['file']}:{f['line']}\t{p}\t<- {', '.join(sorted(f['why']))}\n")

    missing = sorted(p for p in entries if p not in listed and p not in allowed)
    both = sorted(p for p in allowed if p in listed)
    stale = sorted(p for p in (listed | set(allowed)) if p.startswith("v8::") and p not in all_paths)
    hand = sorted(p for p in listed if p.startswith("v8::") and p not in entries)
    print(f"v8-audit: {len(script)} script-running V8 functions, {len(entries)} public rusty_v8 entries; "
          f"{len(listed)} disallowed, {len(allowed)} reviewed as safe, {len(hand)} listed by hand")
    for p in hand:
        print(f"  by hand: {p}")
    bad = False
    for p in missing:
        f = entries[p]
        print(f"MISSING {p} ({f['file']}:{f['line']} <- {', '.join(sorted(f['why']))}): "
              "add it to disallowed-methods, or to reviewed.toml [allow] with a reason")
        bad = True
    for p in stale:
        print(f"STALE {p}: no such public function in rusty_v8 any more")
        bad = True
    for p in both:
        print(f"CONFLICT {p}: both disallowed and reviewed as safe")
        bad = True
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
