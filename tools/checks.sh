#!/usr/bin/env bash
# Izcal repository checks. One script for the git hooks and for CI, so both
# enforce the same rules (CONTRIBUTING.md, "Checks").
#
#   tools/checks.sh --staged             files staged for commit (pre-commit hook)
#   tools/checks.sh --range BASE HEAD    files changed in BASE..HEAD (pre-push, CI)
#
# Options: --review runs the AI code review (pre-commit does by default).
# Environment:
#   IZCAL_HOOKS_SKIP=check1,check2   skip named checks; the commit-msg hook
#                                    records them in a Hooks-Skipped trailer
#   IZCAL_REVIEW_MODEL=sonnet        model for the review (default sonnet)
#   IZCAL_REVIEW_MAX_LINES=4000      diff lines sent to the review
#
# Checks: conflict, size, binary, secrets, rustfmt, shellcheck, actionlint,
#         mdlinks, revision, provenance, security, review.
# Bash 3.2 compatible (macOS /bin/bash).

set -u
MODE="" BASE="" HEAD="" REVIEW=0
while [ $# -gt 0 ]; do
  case "$1" in
    --staged) MODE=staged ;;
    --range) MODE=range; BASE=$2; HEAD=$3; shift 2 ;;
    --review) REVIEW=1 ;;
    *) echo "checks.sh: unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done
[ -n "$MODE" ] || { echo "usage: checks.sh --staged | --range BASE HEAD [--review]" >&2; exit 2; }

CHECKS_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(git rev-parse --show-toplevel) || exit 2
cd "$ROOT" || exit 2
# An unresolvable range would yield an empty file list and a silent pass.
if [ "$MODE" = range ]; then
  for c in "$BASE" "$HEAD"; do
    git rev-parse -q --verify "$c^{commit}" >/dev/null || { echo "checks.sh: commit '$c' not found (fetch it, or check the range)" >&2; exit 2; }
  done
fi
SKIP=",${IZCAL_HOOKS_SKIP:-},"
MAX_BYTES=$((5 * 1024 * 1024))
FAILED="" WARNED=""
TMP=$(mktemp -d "${TMPDIR:-/tmp}/izcal-checks.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

if [ -t 1 ]; then RED=$'\e[31m' GRN=$'\e[32m' YEL=$'\e[33m' DIM=$'\e[2m' OFF=$'\e[0m'; else RED="" GRN="" YEL="" DIM="" OFF=""; fi
skipped() { case "$SKIP" in *",$1,"*) echo "${YEL}- $1: skipped (IZCAL_HOOKS_SKIP)${OFF}"; return 0 ;; esac; return 1; }
fail() { FAILED="$FAILED $1"; echo "${RED}x $1${OFF}"; }
warn() { WARNED="$WARNED $1"; echo "${YEL}! $1${OFF}"; }
ok() { echo "${GRN}ok${OFF} $1"; }
indent() { sed 's/^/    /'; }

# Submodule paths are gitlinks, not files.
SUBMODULES=$(git config -f .gitmodules --get-regexp '\.path$' 2>/dev/null | awk '{print $2}')
is_submodule() { for s in $SUBMODULES; do [ "$1" = "$s" ] && return 0; done; return 1; }

# Changed files (added, copied, modified, renamed), NUL-safe: reports have spaces.
list_files() {
  if [ "$MODE" = staged ]; then git diff --cached --name-only --diff-filter=ACMR -z
  else git diff --name-only --diff-filter=ACMR -z "$BASE" "$HEAD"; fi
}
FILES="$TMP/files"
: > "$FILES"
while IFS= read -r -d '' f; do is_submodule "$f" || printf '%s\n' "$f" >> "$FILES"; done < <(list_files)

# File content as committed (index for --staged), not the working tree, so a
# partially staged file is checked as it will be committed.
content() { if [ "$MODE" = staged ]; then git show ":$1"; else git show "$HEAD:$1"; fi; }
blob_size() { if [ "$MODE" = staged ]; then git cat-file -s ":$1"; else git cat-file -s "$HEAD:$1"; fi; }
exists_in_tree() {
  local p=$1
  if [ "$MODE" = staged ]; then git cat-file -e ":$p" 2>/dev/null || [ -n "$(git ls-files -- "$p/" | head -1)" ] || is_submodule "$p"
  else git cat-file -e "$HEAD:$p" 2>/dev/null; fi
}
diff_text() { if [ "$MODE" = staged ]; then git diff --cached "$@"; else git diff "$BASE" "$HEAD" "$@"; fi; }
files_matching() { grep -E "$1" "$FILES" || true; }

n=$(wc -l < "$FILES" | tr -d ' ')
echo "${DIM}izcal checks: $MODE, $n file(s)${OFF}"

# --- conflict: leftover merge-conflict markers --------------------------------
if ! skipped conflict; then
  bad=""
  while IFS= read -r f; do
    content "$f" | grep -qE '^(<<<<<<<|>>>>>>>) ' && bad="$bad
$f"
  done < "$FILES"
  if [ -n "$bad" ]; then fail conflict; echo "    merge-conflict markers in:$bad" | indent; else ok conflict; fi
fi

# --- size: nothing over 5 MB (archives belong in releases) -------------------
if ! skipped size; then
  bad=""
  while IFS= read -r f; do
    s=$(blob_size "$f" 2>/dev/null || echo 0)
    [ "$s" -gt "$MAX_BYTES" ] && bad="$bad
$f ($((s / 1048576)) MB)"
  done < "$FILES"
  if [ -n "$bad" ]; then fail size; printf '    files over 5 MB:%s\n    publish them as release assets instead (CONTRIBUTING.md)\n' "$bad"; else ok size; fi
fi

# --- binary: build outputs and archives --------------------------------------
if ! skipped binary; then
  bad=$(files_matching '\.(a|lib|o|obj|exe|dll|so|dylib|pdb|gz|xz|zst|tar|zip|7z)$')
  if [ -n "$bad" ]; then fail binary; echo "$bad" | indent; else ok binary; fi
fi

# --- secrets: gitleaks, or a regex fallback ----------------------------------
if ! skipped secrets; then
  if command -v gitleaks >/dev/null; then
    if [ "$MODE" = staged ]; then
      out=$(gitleaks git --pre-commit --staged --redact --no-banner -v . 2>&1); rc=$?
    else
      out=$(gitleaks git --log-opts="$BASE..$HEAD" --redact --no-banner -v . 2>&1); rc=$?
    fi
    if [ $rc -eq 0 ]; then ok "secrets (gitleaks)"; else fail secrets; echo "$out" | grep -vE '^\s*$' | indent; fi
  else
    pat='(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY|xox[baprs]-[A-Za-z0-9-]{10,}|sk-(ant-)?[A-Za-z0-9_-]{30,})'
    # Report where, never what: the matched text would leak into logs.
    hits=$(diff_text -U0 | grep -E '^(\+\+\+ |\+)' | PAT="$pat" awk '/^\+\+\+ / {f=substr($0,7); next} $0 ~ ENVIRON["PAT"] {print f ": added line matches a credential pattern (redacted)"}' | sort -u)
    if [ -n "$hits" ]; then fail secrets; echo "$hits" | indent; else ok "secrets (regex fallback; install gitleaks for full rules)"; fi
  fi
fi

# --- rustfmt: per file, with its crate's edition and rustfmt config ----------
# Runs from the file's directory, so rustup honours that crate's
# rust-toolchain.toml.
nearest() { # nearest file named $2 (or $3) from dir $1 up to the repo root
  local d=$1
  while :; do
    [ -f "$d/$2" ] && { echo "$d/$2"; return; }
    [ -n "${3:-}" ] && [ -f "$d/$3" ] && { echo "$d/$3"; return; }
    [ "$d" = "." ] || [ "$d" = "/" ] && return
    d=$(dirname "$d")
  done
}
if ! skipped rustfmt; then
  rs=$(files_matching '\.rs$' | grep -v '/third_party/' || true)
  if [ -z "$rs" ]; then ok "rustfmt (no Rust files)"
  elif ! command -v rustfmt >/dev/null; then warn "rustfmt not installed"
  else
    bad=""
    while IFS= read -r f; do
      dir=$(dirname "$f")
      manifest=$(nearest "$dir" Cargo.toml)
      ed=$( [ -n "$manifest" ] && sed -n 's/^edition *= *"\([0-9]*\)".*/\1/p' "$manifest" | head -1 )
      # edition.workspace = true (the standard's form, CS-ST-9) resolves
      # against the root manifest's [workspace.package] edition.
      if [ -z "$ed" ] && [ -n "$manifest" ] && grep -q '^edition\.workspace' "$manifest"; then
        ed=$(sed -n '/^\[workspace\.package\]/,/^\[/ s/^edition *= *"\([0-9]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -1)
      fi
      cfg=$(nearest "$dir" rustfmt.toml .rustfmt.toml)
      args="--check --edition ${ed:-2021}"
      [ -n "$cfg" ] && args="$args --config-path $ROOT/$cfg"
      # shellcheck disable=SC2086 # args is a flag list
      # On stdin, rustfmt --check prints the diff but still exits 0, so a
      # diff in the output is the failure signal too.
      out=$(content "$f" | (cd "$dir" && rustfmt $args) 2>&1); rc=$?
      if [ $rc -ne 0 ] || printf '%s' "$out" | grep -q '^Diff in'; then
        bad="$bad
$f"
        echo "$out" | head -20 > "$TMP/rustfmt.$(echo "$f" | tr '/ ' '__')"
      fi
    done <<< "$rs"
    if [ -n "$bad" ]; then fail rustfmt; echo "    not formatted (run rustfmt):$bad" | indent; cat "$TMP"/rustfmt.* 2>/dev/null | head -30 | indent; else ok rustfmt; fi
  fi
fi

# --- shellcheck: warnings and errors ------------------------------------------
if ! skipped shellcheck; then
  # logs*/ folders hold scripts exactly as they ran in M0: records, not code.
  sh=$(files_matching '\.(sh|bash)$|^\.githooks/[^/.]+$' | grep -vE '(^|/)logs[^/]*/' || true)
  if [ -z "$sh" ]; then ok "shellcheck (no shell scripts)"
  elif ! command -v shellcheck >/dev/null; then warn "shellcheck not installed"
  else
    bad=""
    while IFS= read -r f; do
      out=$(content "$f" | shellcheck -S warning -s bash - 2>&1) || bad="$bad
== $f
$out"
    done <<< "$sh"
    if [ -n "$bad" ]; then fail shellcheck; echo "$bad" | head -60 | indent; else ok shellcheck; fi
  fi
fi

# --- actionlint: GitHub workflows ---------------------------------------------
if ! skipped actionlint; then
  wf=$(files_matching '^\.github/workflows/.*\.ya?ml$' || true)
  if [ -z "$wf" ]; then ok "actionlint (no workflows)"
  elif ! command -v actionlint >/dev/null; then warn "actionlint not installed"
  else
    bad=""
    while IFS= read -r f; do
      out=$(content "$f" | actionlint -stdin-filename "$f" - 2>&1) || bad="$bad
$out"
    done <<< "$wf"
    if [ -n "$bad" ]; then fail actionlint; echo "$bad" | head -40 | indent; else ok actionlint; fi
  fi
fi

# --- mdlinks: relative links in Markdown resolve -------------------------------
if ! skipped mdlinks; then
  md=$(files_matching '\.md$' || true)
  if [ -z "$md" ]; then ok "mdlinks (no Markdown)"
  else
    bad=""
    while IFS= read -r f; do
      dir=$(dirname "$f")
      # [text](target) with a relative target; drop #anchors, <>, %20.
      targets=$(content "$f" | python3 -c '
import re, sys, urllib.parse
text = sys.stdin.read()
text = re.sub(r"```.*?```", "", text, flags=re.S)  # ignore fenced code
text = re.sub(r"`[^`\n]*`", "", text)              # and inline code
for m in re.finditer(r"\]\(\s*(<[^>]+>|[^)\s]+)", text):
    t = m.group(1).strip("<>")
    if re.match(r"^([a-z][a-z0-9+.-]*:|#|/)", t, re.I):
        continue
    t = urllib.parse.unquote(t.split("#", 1)[0])
    if t:
        print(t)
')
      [ -n "$targets" ] || continue
      while IFS= read -r t; do
        p=$(python3 -c 'import os,sys; print(os.path.normpath(os.path.join(sys.argv[1], sys.argv[2])))' "$dir" "$t")
        exists_in_tree "$p" || bad="$bad
$f -> $t"
      done <<< "$targets"
    done <<< "$md"
    if [ -n "$bad" ]; then fail mdlinks; echo "    broken relative links:$bad" | indent; else ok mdlinks; fi
  fi
fi

# --- revision: a changed spec records a revision ------------------------------
# The frozen specs keep a revision log in their header. A content change must
# add or edit a "*Revision" or "Status:" line (CONTRIBUTING.md, "Specs").
if ! skipped revision; then
  bad=""
  for spec in "reports/HGBA 2 browser architecture spec.md" "reports/Future software browser engine.md" "reports/HGBA Stage 1 architecture design.md"; do
    grep -qxF "$spec" "$FILES" || continue
    diff_text -U0 -- "$spec" | grep -E '^\+' | grep -v '^+++' | grep -qE '^\+\*?(\*\*)?(Revision|Status)|Status: draft|frozen at' || bad="$bad
$spec"
  done
  if [ -n "$bad" ]; then fail revision; echo "    spec changed without a revision line:$bad" | indent; else ok revision; fi
fi

# --- provenance: no code from other browsers (Stage 1 design D-S10) ----------
# Flags added lines in code files that carry another browser project's
# copyright or license header. Docs may cite them, so only code is checked.
if ! skipped provenance; then
  code=$(files_matching '\.(rs|cc|cpp|c|h|hpp|mm|m|py|js|ts|gn|gni|idl|webidl|sh)$' | grep -v '/third_party/' || true)
  bad=""
  if [ -n "$code" ]; then
    printf '%s\n' "$code" | tr '\n' '\0' > "$TMP/prov_paths"
    # The [o]/[P] brackets keep this line from matching its own pattern.
    pat='C[o]pyright.{0,40}(Chromium Authors|V8 project authors|Servo Project Developers|M[o]zilla Foundation|Apple Inc|Google (Inc|LLC)|KDE|SerenityOS|Ladybird)|M[o]zilla Public License|S[P]DX-License-Identifier: *(MPL|GPL|LGPL|AGPL)'
    if [ "$MODE" = staged ]; then xargs -0 git diff --cached -U0 -- < "$TMP/prov_paths" > "$TMP/prov.diff"
    else xargs -0 git diff -U0 "$BASE" "$HEAD" -- < "$TMP/prov_paths" > "$TMP/prov.diff"; fi
    bad=$(PAT="$pat" awk '/^\+\+\+ / {f=substr($0,7); next} /^\+/ && $0 ~ ENVIRON["PAT"] {print f ": " substr($0,2,100)}' "$TMP/prov.diff")
  fi
  if [ -n "$bad" ]; then fail provenance; echo "    code with another browser project's copyright or license (D-S10):"; echo "$bad" | indent | indent; else ok provenance; fi
fi

# --- security: known-dangerous patterns, workflows and dependencies ---------
# Four deterministic parts, each on what this change adds:
#   (1) Rust `unsafe` blocks and impls need a `// SAFETY:` comment just above
#       them, saying why the invariants hold: in the three lines before, or
#       anywhere in the comment block that ends on the line before. Standalone
#       probe crates under research_notes/ (their own Cargo.toml `[workspace]`,
#       outside Izcal's workspace and never shipped) are exempt: they are
#       black-box measurements of raw OS calls, and every commit that added
#       one recorded the skip and its reason;
#   (2) risky patterns: TLS verification turned off, a download piped into a
#       shell, world-writable permissions;
#   (3) GitHub workflows pass zizmor (unpinned actions, credential
#       persistence, cache poisoning, template injection);
#   (4) a changed Cargo.lock has no known advisory (cargo-deny, RustSec),
#       and outside the archived m0 spikes every shipped crate is
#       available under MIT (cargo-deny licenses, HGBA-2 M-1 part 1).
if ! skipped security; then
  sec_bad=""
  rs=$(files_matching '\.rs$' | grep -v '/third_party/' || true)
  # Drop standalone probe crates: a .rs file under research_notes/ whose
  # nearest Cargo.toml, read at the commit being checked, has a [workspace]
  # table.
  if [ -n "$rs" ]; then
    kept=""
    while IFS= read -r f; do
      case "$f" in
        research_notes/*)
          d=$(dirname "$f"); probe=0
          while [ "$d" != "research_notes" ] && [ "$d" != "." ]; do
            if [ "$MODE" = staged ]; then toml=$(git show ":$d/Cargo.toml" 2>/dev/null)
            else toml=$(git show "$HEAD:$d/Cargo.toml" 2>/dev/null); fi
            if [ -n "$toml" ]; then
              printf '%s\n' "$toml" | grep -q '^\[workspace\]' && probe=1
              break
            fi
            d=$(dirname "$d")
          done
          [ "$probe" = 1 ] && continue ;;
      esac
      kept="$kept$f
"
    done <<EOF_RS
$rs
EOF_RS
    rs=$(printf '%s' "$kept" | sed '/^$/d')
  fi
  if [ -n "$rs" ]; then
    printf '%s\n' "$rs" | tr '\n' '\0' > "$TMP/sec_rs"
    if [ "$MODE" = staged ]; then xargs -0 git diff --cached -U3 -- < "$TMP/sec_rs" > "$TMP/sec_rs.diff"
    else xargs -0 git diff -U3 "$BASE" "$HEAD" -- < "$TMP/sec_rs" > "$TMP/sec_rs.diff"; fi
    # Added lines with `unsafe {` or `unsafe impl`, with no SAFETY: comment
    # in the three lines before them (added or context), nor in the comment
    # block ending on the line before. A block that starts above the diff's
    # context is not seen, so such a hit is a false alarm to fix by moving
    # the SAFETY: line down, never a pass.
    unsafe_hits=$(awk '
      /^\+\+\+ / {f=substr($0,7); split("", prev); inc=0; blk=0; next}
      /^@@/ {split("", prev); inc=0; blk=0; next}
      /^[ +]/ {
        line=substr($0,2)
        if ($0 ~ /^\+/ && line ~ /unsafe[ \t]*(\{|impl)/ && line !~ /^[ \t]*\/\//) {
          ok=0; for (i=1;i<=3;i++) if (prev[i] ~ /SAFETY:/) ok=1
          if (line ~ /SAFETY:/) ok=1
          if (inc && blk) ok=1
          if (!ok) print f ": unsafe without a SAFETY: comment: " substr(line,1,80)
        }
        if (line ~ /^[ \t]*\/\//) { if (!inc) blk=0; inc=1; if (line ~ /SAFETY:/) blk=1 }
        else { inc=0; blk=0 }
        prev[3]=prev[2]; prev[2]=prev[1]; prev[1]=line
      }' "$TMP/sec_rs.diff")
    [ -n "$unsafe_hits" ] && sec_bad="$sec_bad
$unsafe_hits"
  fi
  risky_files=$(files_matching '\.(rs|cc|cpp|h|py|sh|bash|ya?ml|toml|js|ts|bat|ps1)$|^\.githooks/' | grep -v '/third_party/' | grep -v '^tools/checks\.sh$' || true)
  if [ -n "$risky_files" ]; then
    printf '%s\n' "$risky_files" | tr '\n' '\0' > "$TMP/sec_risky"
    if [ "$MODE" = staged ]; then xargs -0 git diff --cached -U0 -- < "$TMP/sec_risky" > "$TMP/sec_risky.diff"
    else xargs -0 git diff -U0 "$BASE" "$HEAD" -- < "$TMP/sec_risky" > "$TMP/sec_risky.diff"; fi
    # The [x] brackets keep this file from matching its own patterns.
    # Passed through the environment, not awk -v: -v would process the
    # backslash escapes and silently change the pattern. POSIX awk has no \b.
    rpat='d[a]nger_accept_invalid_(certs|hostnames)|d[a]ngerous_configuration|I[n]secureSkipVerify|v[e]rify *= *False|N[O]DE_TLS_REJECT_UNAUTHORIZED *= *0|G[I]T_SSL_NO_VERIFY|(c[u]rl|w[g]et)[^|#]*[|] *(sudo +)?(ba|z)?sh( |;|$)|(ba|z)?s[h] +<[(] *(c[u]rl|w[g]et)|c[h]mod +(-R +)?(0?777|a[+]w)|--[i]nsecure( |$)'
    risky_hits=$(RPAT="$rpat" awk '/^\+\+\+ / {f=substr($0,7); next} /^\+/ && $0 ~ ENVIRON["RPAT"] {print f ": risky pattern: " substr($0,2,90)}' "$TMP/sec_risky.diff")
    [ -n "$risky_hits" ] && sec_bad="$sec_bad
$risky_hits"
  fi
  sec_warn=""
  wf=$(files_matching '^\.github/workflows/.*\.ya?ml$' || true)
  if [ -n "$wf" ]; then
    if command -v zizmor >/dev/null; then
      while IFS= read -r f; do
        content "$f" > "$TMP/zz.yml"
        zout=$(zizmor --offline --min-severity medium --no-progress --format plain "$TMP/zz.yml" 2>&1) || sec_bad="$sec_bad
== zizmor: $f
$(printf '%s\n' "$zout" | grep -E '^(error|warning)\[|^ *--> |\^' | sed "s|$TMP/zz.yml|$f|" | head -20)"
      done <<< "$wf"
    else sec_warn="$sec_warn zizmor"; fi
  fi
  locks=$(files_matching '(^|/)Cargo\.lock$' | grep -v '/third_party/' || true)
  if [ -n "$locks" ]; then
    if command -v cargo-deny >/dev/null; then
      while IFS= read -r lock; do
        dir=$(dirname "$lock")
        # --config goes before `check`: cargo-deny 0.20 rejects it after
        # (exit 2, "unexpected argument"), whatever `check --help` suggests.
        if ! dout=$(cd "$dir" && cargo-deny --log-level error --config "$CHECKS_DIR/deny.toml" check advisories 2>&1); then
          if printf '%s' "$dout" | grep -qiE 'failed to (fetch|open)|network|could not resolve|unable to (fetch|load)'; then
            sec_warn="$sec_warn cargo-deny(no advisory database: offline?)"
          else
            sec_bad="$sec_bad
== cargo-deny advisories: $lock
$(printf '%s\n' "$dout" | grep -E '^(error|warning)\[|ID:|Crate:|Title:|^ +[a-z_-]+ [0-9.]+' | head -20)"
          fi
        fi
        # The archived m0 spikes (HGBA-2 2.3.0) keep their historical
        # dependencies. Research measurement crates are probes, never
        # shipped, so their dependencies are outside the MIT-only rule
        # (HGBA-2 2.3.2 covers shipped dependencies). Test-only crates
        # may carry any licence, so dev-dependencies are left out of
        # this run only.
        case "$dir" in m0|m0/*|research_notes|research_notes/*) continue ;; esac
        if ! lout=$(cd "$dir" && cargo-deny --log-level error --exclude-dev --config "$CHECKS_DIR/deny.toml" check licenses 2>&1); then
          sec_bad="$sec_bad
== cargo-deny licenses (MIT only): $lock
$(printf '%s\n' "$lout" | grep -E '^[0-9]+ │ license = |^ +├ [A-Za-z0-9_-]+ v[0-9]' | sed -e 's/^[0-9]* │ /    /' -e 's/^ *├ /    crate: /' | head -20)"
        fi
      done <<< "$locks"
    else sec_warn="$sec_warn cargo-deny"; fi
  fi
  if [ -n "$sec_bad" ]; then
    fail security; echo "$sec_bad" | sed '/^$/d' | indent
    echo "    fix it, or skip with IZCAL_HOOKS_SKIP=security and say why in the commit body" | indent
  else ok security; fi
  [ -n "$sec_warn" ] && warn "security: not checked (tool missing or offline):$sec_warn"
fi

# --- review: AI code review of the diff (code only) ---------------------------
# Blocks only on a high-severity finding. If the reviewer cannot run (no CLI,
# no network, timeout), it warns and lets the commit through: an outage must
# not stop work, and CI re-runs the deterministic checks anyway.
if [ "$REVIEW" = 1 ] && ! skipped review; then
  code=$(files_matching '\.(rs|cc|cpp|h|hpp|py|sh|bash|toml|ya?ml|gn|gni|js|ts)$|^\.githooks/' | grep -v '/third_party/' || true)
  if [ -z "$code" ]; then ok "review (no code changes)"
  elif ! command -v claude >/dev/null; then warn "review: claude CLI not found"
  else
    max=${IZCAL_REVIEW_MAX_LINES:-4000}
    # NUL-separated pathspecs for the code files only.
    printf '%s\n' "$code" | tr '\n' '\0' > "$TMP/paths"
    if [ "$MODE" = staged ]; then xargs -0 git diff --cached -U5 -- < "$TMP/paths" > "$TMP/diff"
    else xargs -0 git diff -U5 "$BASE" "$HEAD" -- < "$TMP/paths" > "$TMP/diff"; fi
    total=$(wc -l < "$TMP/diff" | tr -d ' ')
    head -n "$max" "$TMP/diff" > "$TMP/diff.cut"
    note=""; [ "$total" -gt "$max" ] && note="The diff was cut to its first $max of $total lines; say so if that hides context you need."
    schema='{"type":"object","properties":{"verdict":{"enum":["pass","warn","block"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"enum":["high","medium","low"]},"file":{"type":"string"},"line":{"type":"integer"},"issue":{"type":"string"}},"required":["severity","file","issue"]}}},"required":["verdict","findings"]}'
    prompt="You are reviewing a staged change to Izcal, a Rust browser engine built from the ground up (HGBA-2 2.3.0: the engine is Izcal's own — no V8, no fork). The unified diff is on stdin. Report only real defects: correctness bugs, memory or thread safety (unsafe, the cage rules H-1/H-2, the token-cell discipline CS-V8-1, SAFETY arguments), security, broken builds or CI, security defects (for Izcal: unvalidated data crossing a process or IPC boundary, a capability or handle passed further than needed, ambient authority in a sandboxed process, untrusted input parsed in a privileged process, cross-site data reaching one process (HGBA-2 P1, I-1, I-2), secrets or credentials logged or exported, missing consent or origin checks on inspection, automation or agent surfaces (B9, B10), weak or hand-rolled cryptography, unsafe Rust whose SAFETY argument does not hold, injection into shells, SQL or HTML), violations of the project's written rules, and code that appears copied, ported or translated from another browser engine (Servo, Chromium/Blink, WebKit, Gecko/Firefox, Ladybird), which is forbidden by Stage 1 design D-S10 and is always high severity. You may read files in the repository for context: CONTRIBUTING.md, the specs in reports/ (HGBA-2 targets such as S-4a/S-4c, the Stage 1 design's crate rules G1-G9). Do not report style or formatting (other checks cover those) or speculation. Severity high = would ship a bug or break the build; medium = likely bug; low = minor. verdict: block if any high finding, warn if any medium, else pass. Keep each issue to one or two sentences. $note"
    start=$(date +%s)
    if command -v timeout >/dev/null; then TO="timeout 300"; elif command -v gtimeout >/dev/null; then TO="gtimeout 300"; else TO=""; fi
    # shellcheck disable=SC2086 # TO is a command prefix
    $TO claude -p --model "${IZCAL_REVIEW_MODEL:-sonnet}" --tools "Read,Grep,Glob" --allowedTools "Read,Grep,Glob" \
      --output-format json --json-schema "$schema" "$prompt" < "$TMP/diff.cut" > "$TMP/review.json" 2> "$TMP/review.err"
    rc=$?
    secs=$(( $(date +%s) - start ))
    verdict=$(python3 - "$TMP/review.json" <<'PY' 2>/dev/null
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(1)
res = d[-1] if isinstance(d, list) else d
out = res.get("structured_output")
if not isinstance(out, dict) or res.get("is_error"):
    sys.exit(1)
print(out["verdict"], f"{res.get('total_cost_usd', 0):.3f}")
for f in out.get("findings", []):
    loc = f["file"] + (f":{f['line']}" if f.get("line") else "")
    print(f"  [{f['severity']}] {loc}: {f['issue']}")
PY
)
    if [ $rc -ne 0 ] || [ -z "$verdict" ]; then
      warn "review could not run (exit $rc after ${secs}s); not blocking"
      head -5 "$TMP/review.err" | indent
    else
      head=$(echo "$verdict" | head -1); v=${head%% *}; cost=${head##* }
      body=$(echo "$verdict" | tail -n +2)
      label="review ($total diff lines, ${secs}s, \$$cost)"
      case "$v" in
        block) fail "$label: high-severity findings"; echo "$body" | indent
               echo "    fix them, or skip with IZCAL_HOOKS_SKIP=review and say why in the commit body" | indent ;;
        warn)  warn "$label: findings to consider"; echo "$body" | indent ;;
        *)     ok "$label"; [ -n "$body" ] && echo "$body" | indent ;;
      esac
    fi
  fi
fi

if [ -n "$FAILED" ]; then
  echo "${RED}checks failed:${FAILED}${OFF}"
  echo "${DIM}skip a check only with a reason: IZCAL_HOOKS_SKIP=<name> git commit ... (recorded in the commit)${OFF}"
  exit 1
fi
[ -n "$WARNED" ] && echo "${YEL}warnings:${WARNED}${OFF}"
exit 0
