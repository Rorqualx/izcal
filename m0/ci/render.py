"""Render one matrix entry of a GitHub Actions job into a local bash script.

Runs the workflow's own `run:` steps in order, with ${{ }} expressions and
`if:` conditions evaluated, step/job env applied, and GITHUB_ENV handled the
way the runner does (re-read after every step). `uses:` steps are replaced by
local equivalents or skipped (listed in the script header).

usage: render.py workflow.yml JOB MATRIX_INDEX RUNNER_OS WORKDIR OUT.sh
"""
import re, sys, json, shlex
import yaml

wf, job_name, idx, runner_os, workdir, out = sys.argv[1:7]
idx = int(idx)
doc = yaml.safe_load(open(wf))
job = doc["jobs"][job_name]
matrix = job["strategy"]["matrix"]["include"][idx]

ctx = {"matrix": matrix, "runner": {"os": runner_os}, "vars": {},
       "github": {"ref": "refs/heads/hgba", "repository": "local/izcal-rusty_v8"}}

def lookup(path):
    cur = ctx
    for p in path.split("."):
        cur = cur.get(p, "") if isinstance(cur, dict) else ""
    return cur

def ev(expr):
    e = expr.strip()
    py = re.sub(r"\b(matrix|runner|vars|github)\.([A-Za-z0-9_.]+)",
                lambda m: f"_l({m.group(0)!r})", e)
    py = py.replace("&&", " and ").replace("||", " or ").replace("!", " not ").replace(" not =", "!=")
    py = re.sub(r"\bcontains\(", "_contains(", py)
    py = re.sub(r"\bstartsWith\(", "_starts(", py)
    v = eval(py, {"_l": lookup,
                  "_contains": lambda a, b: str(b) in str(a),
                  "_starts": lambda a, b: str(a).startswith(str(b))})
    return v

def sub(s):
    if not isinstance(s, str):
        return s
    def r(m):
        v = ev(m.group(1))
        return "" if v is None or v is False else str(v)
    return re.sub(r"\$\{\{(.*?)\}\}", r, s)

# GitHub evaluates expressions inside matrix values when it expands the matrix.
for k, v in list(matrix.items()):
    matrix[k] = sub(v)

skipped, lines = [], []
lines += ["#!/usr/bin/env bash", "set -u", f"cd {shlex.quote(workdir)}",
          'RUN=${RUN_DIR:?}', 'export GITHUB_ENV="$RUN/github_env" GITHUB_STEP_SUMMARY="$RUN/summary.md"',
          ': > "$GITHUB_ENV"; : > "$GITHUB_STEP_SUMMARY"',
          f"export RUNNER_OS={runner_os} GITHUB_REF=refs/heads/hgba",
          'load_env() { while IFS= read -r l; do l="${l#$\'\\xef\\xbb\\xbf\'}"; l="${l%$\'\\r\'}"; [ -n "$l" ] && export "$l"; done < "$GITHUB_ENV"; }']
for k, v in {**doc.get("env", {}), **job.get("env", {})}.items():
    lines.append(f"export {k}={shlex.quote(str(sub(str(v))))}")
lines.append('echo "matrix: ' + json.dumps(matrix).replace('"', "'") + '"')

for i, st in enumerate(job["steps"]):
    name = st.get("name") or st.get("uses")
    if "if" in st and not ev(st["if"].replace("${{", "").replace("}}", "")):
        lines.append(f'echo "::skip {i} {name} (if: false)"')
        continue
    if "uses" in st:
        u = st["uses"]
        if u.startswith("mozilla-actions/sccache-action"):
            lines += [f'echo "::step {i} {name} (local: install sccache)"',
                      'if ! command -v sccache >/dev/null; then',
                      '  url=$(curl -fsSL https://api.github.com/repos/mozilla/sccache/releases/latest | grep -o "https://[^\\"]*x86_64-pc-windows-msvc.tar.gz" | head -1)',
                      '  mkdir -p "$RUN/sccache" && curl -fsSL "$url" | tar -xz -C "$RUN/sccache" --strip-components=1',
                      '  export PATH="$RUN/sccache:$PATH"',
                      'fi', 'sccache --version']
        else:
            skipped.append(u)
            lines.append(f'echo "::skip {i} {u} (uses: provided by the local runner)"')
        continue
    env = {k: sub(str(v)) for k, v in st.get("env", {}).items()}
    body = sub(st["run"])
    shell = st.get("shell", "bash")
    lines.append(f'echo "::step {i} {name}"; date')
    envs = " ".join(f"{k}={shlex.quote(v)}" for k, v in env.items())
    if shell == "pwsh":
        ps = f"$RUN/step{i}.ps1"
        lines += [f"cat > \"{ps}\" <<'PS1EOF'", body, "PS1EOF",
                  f'PS=$(command -v pwsh || command -v powershell); env {envs} "$PS" -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "{ps}")"']
    else:
        sh = f"$RUN/step{i}.sh"
        lines += [f"cat > \"{sh}\" <<'SHEOF'", body, "SHEOF",
                  f'env {envs} bash -eo pipefail "{sh}"']
    lines += [f'rc=$?; echo "::rc {i} $rc"; [ $rc -eq 0 ] || {{ echo "::failed {i} {name}"; exit $rc; }}',
              "load_env"]
lines.append('echo "::job-done"')
open(out, "w", newline="\n").write("\n".join(lines) + "\n")
print("skipped uses:", skipped)
