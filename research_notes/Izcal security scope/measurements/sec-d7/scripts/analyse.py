"""Analyse SEC-D7 feed samples as untrusted text. Never resolves or fetches any listed URL.

usage: python3 -I analyse.py <feeds_root> <ref_dir> <out_json>
"""
import csv, io, json, os, re, statistics, sys
from datetime import datetime, timezone
from itertools import combinations
from urllib.parse import urlsplit

FEEDS, REF, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
csv.field_size_limit(10_000_000)

# ---------- reference data (PSL, Tranco) ----------
def load_psl(path):
    rules, exc, wild = set(), set(), set()
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("//"):
            continue
        line = line.split()[0]
        if line.startswith("!"):
            exc.add(line[1:])
        elif line.startswith("*."):
            wild.add(line[2:])
        else:
            rules.add(line)
    return rules, exc, wild

PSL = load_psl(os.path.join(REF, "psl.dat"))

def etld1(host):
    rules, exc, wild = PSL
    labels = host.split(".")
    for i in range(len(labels)):
        cand = ".".join(labels[i:])
        if cand in exc:
            return ".".join(labels[i:]) if i < len(labels) else None
        parent = ".".join(labels[i + 1:])
        if cand in rules or (parent and parent in wild):
            # A wildcard rule (e.g. "*.ck") matches apex-level hosts (e.g.
            # "a.ck") at i == 0, where there is no label above to return;
            # keep the full host as its own eTLD+1 rather than dropping it.
            return ".".join(labels[i - 1:]) if i > 0 else host
    return ".".join(labels[-2:]) if len(labels) >= 2 else None

TRANCO = {}
for row in csv.reader(open(os.path.join(REF, "top-1m.csv"), encoding="utf-8")):
    TRANCO[row[1].lower()] = int(row[0])

# ---------- normalisation ----------
IP_RE = re.compile(r"^\d{1,3}(\.\d{1,3}){3}$")

def norm_host(h):
    h = (h or "").strip().lower().rstrip(".")
    if h.startswith("www."):
        h = h[4:]
    if not h or " " in h or "/" in h:
        return None
    return h

def host_of_url(u):
    u = u.strip()
    if not u:
        return None
    if "://" not in u:
        u = "http://" + u
    try:
        p = urlsplit(u)
        return norm_host(p.hostname)
    except ValueError:
        return None

def norm_url(u):
    u = u.strip()
    if "://" not in u:
        u = "http://" + u
    try:
        p = urlsplit(u)
    except ValueError:
        return None
    h = norm_host(p.hostname)
    if not h:
        return None
    path = p.path or "/"
    return h + path + (("?" + p.query) if p.query else "")

def lines(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        for l in f:
            l = l.strip()
            if l and not l.startswith("#"):
                yield l

def parse_ts(s):
    s = s.strip().strip('"')
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%dT%H:%M:%S.%fZ"):
        try:
            d = datetime.strptime(s, fmt)
            return d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        except ValueError:
            pass
    return None

# ---------- per-feed parsers: return dict(urls=set, hosts=set, ages=[min], header_ts=str, extra={}) ----------
def p_urllist(path):
    urls, hosts = set(), set()
    for l in lines(path):
        nu = norm_url(l)
        if nu:
            urls.add(nu)
        h = host_of_url(l)
        if h:
            hosts.add(h)
    return dict(urls=urls, hosts=hosts)

def p_domains(path):
    hosts = set()
    for l in lines(path):
        h = norm_host(l.split()[0])
        if h:
            hosts.add(h)
    return dict(urls=set(), hosts=hosts)

def header_value(path, pat):
    with open(path, encoding="utf-8", errors="replace") as f:
        for _ in range(40):
            l = f.readline()
            m = re.search(pat, l)
            if m:
                return m.group(1).strip()
    return None

def p_urlhaus_csv(path, fetched):
    urls, hosts, ages, threats, online = set(), set(), [], {}, 0
    for row in csv.reader((l for l in open(path, encoding="utf-8", errors="replace") if not l.startswith("#"))):
        if len(row) < 6:
            continue
        nu = norm_url(row[2]); h = host_of_url(row[2])
        if nu: urls.add(nu)
        if h: hosts.add(h)
        d = parse_ts(row[1])
        if d: ages.append((fetched - d).total_seconds() / 3600)
        threats[row[5]] = threats.get(row[5], 0) + 1
        online += row[3] == "online"
    return dict(urls=urls, hosts=hosts, ages_h=ages, extra=dict(threat=threats, online=online))

def p_threatfox(path, fetched):
    urls, hosts, types, ttypes, ages = set(), set(), {}, {}, []
    for row in csv.reader((l for l in open(path, encoding="utf-8", errors="replace") if not l.startswith("#")), skipinitialspace=True):
        if len(row) < 5:
            continue
        v, t = row[2], row[3]
        types[t] = types.get(t, 0) + 1
        ttypes[row[4]] = ttypes.get(row[4], 0) + 1
        d = parse_ts(row[0])
        if d: ages.append((fetched - d).total_seconds() / 3600)
        if t == "url":
            nu = norm_url(v); h = host_of_url(v)
            if nu: urls.add(nu)
            if h: hosts.add(h)
        elif t == "domain":
            h = norm_host(v)
            if h: hosts.add(h)
    return dict(urls=urls, hosts=hosts, ages_h=ages, extra=dict(ioc_type=types, threat_type=ttypes))

def p_phishtank(path, fetched):
    urls, hosts, ages, verif, targets = set(), set(), [], [], {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for row in csv.DictReader(f):
            u = row.get("url", "")
            nu = norm_url(u); h = host_of_url(u)
            if nu: urls.add(nu)
            if h: hosts.add(h)
            s = parse_ts(row.get("submission_time", "")); v = parse_ts(row.get("verification_time", ""))
            if s: ages.append((fetched - s).total_seconds() / 3600)
            if s and v: verif.append((v - s).total_seconds() / 60)
            t = row.get("target", "")
            targets[t] = targets.get(t, 0) + 1
    top = sorted(targets.items(), key=lambda x: -x[1])[:8]
    return dict(urls=urls, hosts=hosts, ages_h=ages,
                extra=dict(verif_delay_min_median=statistics.median(verif) if verif else None,
                           verif_delay_min_p90=(statistics.quantiles(verif, n=10)[-1] if len(verif) > 10 else None),
                           top_targets=top))

def p_phishstats(path, fetched):
    urls, hosts, ages, scores = set(), set(), [], []
    try:
        data = json.load(open(path, encoding="utf-8"))
    except Exception as e:
        return dict(urls=set(), hosts=set(), error=str(e))
    for r in data:
        u = r.get("url") or ""
        nu = norm_url(u); h = host_of_url(u)
        if nu: urls.add(nu)
        if h: hosts.add(h)
        d = parse_ts(r.get("date") or "")
        if d: ages.append((fetched - d).total_seconds() / 3600)
    return dict(urls=urls, hosts=hosts, ages_h=ages)

def p_bazaar(path, fetched):
    hs = {l.lower() for l in lines(path) if re.fullmatch(r"[0-9a-fA-F]{64}", l)}
    return dict(urls=set(), hosts=set(), hashes=hs)

FEEDDEF = {
    "openphish":        ("url",  lambda p, t: p_urllist(p), r"^$"),
    "urlhaus_online":   ("url",  lambda p, t: p_urllist(p), r"Last updated: (.*?)\s*#?$"),
    "urlhaus_recent":   ("url",  p_urlhaus_csv, r"Last updated: (.*?)\s*#?$"),
    "threatfox_recent": ("mix",  p_threatfox, r"Last updated: (.*?)\s*#?$"),
    "bazaar_sha256":    ("hash", p_bazaar, r"Last updated: (.*?)\s*#?$"),
    "pdb_links_new":    ("url",  lambda p, t: p_urllist(p), r"^$"),
    "pdb_domains_new":  ("dom",  lambda p, t: p_domains(p), r"^$"),
    "pdb_domains_act":  ("dom",  lambda p, t: p_domains(p), r"^$"),
    "phisharmy":        ("dom",  lambda p, t: p_domains(p), r"Generated: (\S+)"),
    "certpl":           ("dom",  lambda p, t: p_domains(p), r"^$"),
    "phishtank":        ("url",  p_phishtank, r"^$"),
    "phishstats_api":   ("url",  p_phishstats, r"^$"),
    "phishunt":         ("url",  lambda p, t: p_urllist(p), r"^$"),
}
PHISH_FEEDS = ["openphish", "phishtank", "phishstats_api", "phishunt", "pdb_domains_act", "phisharmy", "certpl"]
MAL_FEEDS = ["urlhaus_online", "urlhaus_recent", "threatfox_recent"]

def headers(path):
    out = {}
    try:
        for l in open(path, encoding="latin-1"):
            if ":" in l:
                k, v = l.split(":", 1)
                if k.lower() in ("last-modified", "etag", "content-length"):
                    out[k.lower()] = v.strip()
    except FileNotFoundError:
        pass
    return out

def summ(xs):
    if not xs:
        return None
    xs = sorted(xs)
    q = lambda f: xs[min(len(xs) - 1, int(f * len(xs)))]
    return dict(n=len(xs), min=round(xs[0], 2), p10=round(q(0.10), 2), median=round(q(0.5), 2), p90=round(q(0.9), 2), max=round(xs[-1], 2))

# ---------- load samples ----------
samples = sorted(d for d in os.listdir(FEEDS) if re.fullmatch(r"\d{8}T\d{6}Z", d))
data = {}   # sample -> feed -> parsed
report = dict(samples=samples, per_sample={}, churn={}, overlap={}, lag={}, fp_proxy={})
for s in samples:
    fetched = datetime.strptime(s, "%Y%m%dT%H%M%SZ").replace(tzinfo=timezone.utc)
    data[s] = {}
    report["per_sample"][s] = {}
    idx = {}
    ip = os.path.join(FEEDS, s, "_index.tsv")
    if os.path.exists(ip):
        for l in open(ip, encoding="utf-8"):
            parts = l.rstrip("\n").split("\t")
            idx[parts[0]] = parts[2] if len(parts) > 2 else ""
    for name, (kind, fn, hpat) in FEEDDEF.items():
        body = os.path.join(FEEDS, s, name + ".body")
        if not os.path.exists(body):
            continue
        code = idx.get(name, "")
        if code and not code.startswith("200"):
            report["per_sample"][s][name] = dict(error=code, err=open(os.path.join(FEEDS, s, name + ".err")).read().strip()[:200] if os.path.exists(os.path.join(FEEDS, s, name + ".err")) else "")
            continue
        r = fn(body, fetched)
        data[s][name] = r
        rec = dict(bytes=os.path.getsize(body), urls=len(r.get("urls", ())), hosts=len(r.get("hosts", ())),
                   http=headers(os.path.join(FEEDS, s, name + ".headers")),
                   header_ts=header_value(body, hpat) if hpat != r"^$" else None)
        if "hashes" in r: rec["hashes"] = len(r["hashes"])
        if r.get("ages_h"): rec["entry_age_hours"] = summ(r["ages_h"])
        if r.get("extra"): rec["extra"] = r["extra"]
        if r.get("error"): rec["error"] = r["error"]
        report["per_sample"][s][name] = rec

# ---------- churn between consecutive samples ----------
for name in FEEDDEF:
    seq = [(s, data[s][name]) for s in samples if name in data[s]]
    rows = []
    for (s0, a), (s1, b) in zip(seq, seq[1:]):
        key = "hashes" if "hashes" in a else ("urls" if a.get("urls") else "hosts")
        A, Bs = a[key], b[key]
        dt = (datetime.strptime(s1, "%Y%m%dT%H%M%SZ") - datetime.strptime(s0, "%Y%m%dT%H%M%SZ")).total_seconds() / 60
        rows.append(dict(frm=s0, to=s1, minutes=round(dt), key=key, before=len(A), after=len(Bs),
                         added=len(Bs - A), removed=len(A - Bs),
                         added_per_hour=round(len(Bs - A) / (dt / 60), 1) if dt else None))
    if seq:
        first, last = seq[0][1], seq[-1][1]
        key = "hashes" if "hashes" in first else ("urls" if first.get("urls") else "hosts")
        rows.append(dict(summary=True, first=seq[0][0], last=seq[-1][0], survivors=len(first[key] & last[key]),
                         first_n=len(first[key]), last_n=len(last[key]), union=len(set().union(*[x[1][key] for x in seq]))))
    report["churn"][name] = rows

# ---------- overlap on hostnames (union across all samples per feed) ----------
def union_hosts(name):
    return set().union(*[data[s][name]["hosts"] for s in samples if name in data[s]]) if any(name in data[s] for s in samples) else set()

U = {n: union_hosts(n) for n in PHISH_FEEDS + MAL_FEEDS}
U["urlhaus_any"] = U.pop("urlhaus_online") | U.pop("urlhaus_recent")
names = PHISH_FEEDS + ["urlhaus_any", "threatfox_recent"]
pairs = {}
for a, b in combinations(names, 2):
    A, Bs = U[a], U[b]
    inter = len(A & Bs)
    pairs[f"{a}|{b}"] = dict(inter=inter, jaccard=round(inter / len(A | Bs), 4) if (A | Bs) else 0,
                             frac_a_in_b=round(inter / len(A), 4) if A else 0, frac_b_in_a=round(inter / len(Bs), 4) if Bs else 0)
only = {}
for n in names:
    others = set().union(*[U[m] for m in names if m != n])
    only[n] = dict(total=len(U[n]), only_here=len(U[n] - others), frac_only=round(len(U[n] - others) / len(U[n]), 4) if U[n] else 0)
allh = set().union(*U.values())
report["overlap"] = dict(pairs=pairs, unique=only, union_all=len(allh),
                         in_k_feeds={k: sum(1 for h in allh if sum(h in U[n] for n in names) == k) for k in range(1, len(names) + 1)})

# eTLD+1 level overlap for small URL feeds vs big domain lists (subdomain match)
def covered_by_domain_list(host, domset):
    labels = host.split(".")
    return any(".".join(labels[i:]) in domset for i in range(len(labels) - 1))
dl = {n: U[n] for n in ("pdb_domains_act", "phisharmy", "certpl")}
cov = {}
for small in ("openphish", "phishtank", "phishstats_api", "phishunt"):
    cov[small] = {n: round(sum(covered_by_domain_list(h, d) for h in U[small]) / len(U[small]), 4) if U[small] else 0 for n, d in dl.items()}
report["overlap"]["url_feed_hosts_covered_by_domain_lists_incl_parent"] = cov

# ---------- cross-feed lag: hosts first seen in feed A at sample k (k>0); where are they in B ----------
first_seen = {}
for n in names:
    fs = {}
    src = ["urlhaus_online", "urlhaus_recent"] if n == "urlhaus_any" else [n]
    for i, s in enumerate(samples):
        hs = set()
        for m in src:
            if m in data[s]:
                hs |= data[s][m]["hosts"]
        for h in hs:
            fs.setdefault(h, i)
    first_seen[n] = fs
lag = {}
for a in PHISH_FEEDS:
    new_a = {h: i for h, i in first_seen[a].items() if i > 0}
    if not new_a:
        continue
    row = {"new_in_a_after_first_sample": len(new_a)}
    for b in PHISH_FEEDS:
        if a == b:
            continue
        earlier = same = later = never = 0
        for h, i in new_a.items():
            j = first_seen[b].get(h)
            if j is None: never += 1
            elif j < i: earlier += 1
            elif j == i: same += 1
            else: later += 1
        row[b] = dict(b_earlier=earlier, same_sample=same, b_later=later, never=never)
    lag[a] = row
report["lag"] = lag

# ---------- FP proxy: listed hostname whose registrable domain is popular (Tranco) and host == registrable domain ----------
for n in PHISH_FEEDS + ["urlhaus_any", "threatfox_recent"]:
    hits = []
    for h in U[n]:
        if IP_RE.match(h):
            continue
        e = etld1(h)
        if e and h == e and e in TRANCO and TRANCO[e] <= 100000:
            hits.append((TRANCO[e], h))
    hits.sort()
    sub_hits = sum(1 for h in U[n] if not IP_RE.match(h) and (e := etld1(h)) and h != e and e in TRANCO and TRANCO[e] <= 10000)
    report["fp_proxy"][n] = dict(apex_in_tranco_top100k=len(hits), top=hits[:25],
                                 hosts_under_top10k_registrable=sub_hits, total=len(U[n]),
                                 ip_literal_hosts=sum(1 for h in U[n] if IP_RE.match(h)))

json.dump(report, open(OUT, "w"), indent=1, default=str)
print("samples:", samples)
for s in samples:
    print(s, {k: (v.get("urls") or v.get("hosts") or v.get("hashes") or v.get("error")) for k, v in report["per_sample"][s].items()})
