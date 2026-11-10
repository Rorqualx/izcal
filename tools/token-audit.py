#!/usr/bin/env python3
"""CS-D3 criterion 2: the script token guards every path to user code.

Criterion 2 asks that every engine entry point which can run script be
reachable only through a token-taking wrapper. In Izcal's own engine that
is checkable rather than audited by hand, because the same fact is written
down twice and generated once:

  * each `runtime` declaration in `crates/hg-js-ops/ops/*.hgop` says
    whether the function `call_user` -- whether it can run user code;
  * `hg-opgen` turns those declarations into the generated `Runtime`
    trait, giving a `t: &mut ScriptToken` parameter to exactly the ones
    that declared it.

So the audit is an equality: the set of declarations with `call_user` must
be the set of trait methods that take the token, and the two sets of
method names must themselves match. A drift either way is a real defect.
A method that takes the token without declaring the effect misstates what
an operation does, which the operation facts and the optimiser believe. A
method that declares `call_user` without taking the token is the one that
matters: it can run user code with nothing marking the re-entry, which is
criterion 2 failing.

The hand-written user-call entry point, `RuntimeBase::tail_call`, is
checked too: it is the one path to user code that no description
generates.

Usage: token-audit.py [repo-root]
"""

import pathlib
import re
import sys

OPS = "crates/hg-js-ops/ops"
TRAIT = "crates/hg-js-ops/src/generated/interp.rs"
BASE = "crates/hg-js-ops/src/support.rs"

# `runtime rt_name(args) -> Ret` then, on following lines, `effects { .. }`.
RUNTIME = re.compile(r"^runtime\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
EFFECTS = re.compile(r"effects\s*\{([^}]*)\}")


def declared(root):
    """{name: declares call_user} for every `runtime` declaration."""
    out = {}
    files = sorted(pathlib.Path(root, OPS).glob("*.hgop"))
    if not files:
        raise SystemExit(f"token-audit.py: no .hgop descriptions under {OPS}")
    for path in files:
        text = path.read_text(encoding="utf-8")
        # Comments may contain the words this parses, so drop them first.
        text = re.sub(r"//[^\n]*", "", text)
        for match in RUNTIME.finditer(text):
            name = match.group(1)
            # The effects clause of this declaration is the first one after
            # it, and always before the next `runtime` or `op`.
            rest = text[match.end():]
            stop = re.search(r"^(?:runtime|op)\s", rest, re.M)
            if stop:
                rest = rest[: stop.start()]
            effects = EFFECTS.search(rest)
            if effects is None:
                raise SystemExit(
                    f"token-audit.py: `runtime {name}` in {path.name} has no "
                    "effects clause, so what it may do is unstated"
                )
            words = {w.strip() for w in re.split(r"[,\s]+", effects.group(1))}
            out[name] = "call_user" in words
    return out


def generated(root):
    """{name: takes a ScriptToken} for every method of the Runtime trait."""
    text = pathlib.Path(root, TRAIT).read_text(encoding="utf-8")
    out = {}
    # Each method runs from its `fn` to the `;` that ends its signature.
    for match in re.finditer(r"\n    fn\s+([A-Za-z_][A-Za-z0-9_]*)", text):
        name = match.group(1)
        rest = text[match.end():]
        end = rest.find(";")
        if end < 0:
            raise SystemExit(f"token-audit.py: no end to `fn {name}`'s signature")
        out[name] = "ScriptToken" in rest[:end]
    if not out:
        raise SystemExit(
            f"token-audit.py: no trait methods found in {TRAIT}; the parser "
            "would then report every set as equal and prove nothing"
        )
    return out


def hand_written(root):
    """Whether `RuntimeBase::tail_call` takes the token."""
    text = pathlib.Path(root, BASE).read_text(encoding="utf-8")
    match = re.search(r"fn tail_call<'s>\((.*?)\)\s*->", text, re.S)
    if match is None:
        raise SystemExit(
            "token-audit.py: RuntimeBase::tail_call not found; it is the one "
            "path to user code no description generates, so it is not "
            "optional to check"
        )
    return "ScriptToken" in match.group(1)


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    decls = declared(root)
    trait = generated(root)
    bad = []

    only_declared = sorted(set(decls) - set(trait))
    only_generated = sorted(set(trait) - set(decls))
    for name in only_declared:
        bad.append(f"{name}: declared in a description but not in the trait")
    for name in only_generated:
        bad.append(f"{name}: in the trait but no description declares it")

    for name in sorted(set(decls) & set(trait)):
        if decls[name] and not trait[name]:
            bad.append(
                f"{name}: declares `call_user` but takes no ScriptToken, so it "
                "can run user code with nothing marking the re-entry"
            )
        elif trait[name] and not decls[name]:
            bad.append(
                f"{name}: takes a ScriptToken but does not declare `call_user`, "
                "so the operation facts understate what it does"
            )

    if not hand_written(root):
        bad.append(
            "RuntimeBase::tail_call: the hand-written user call takes no "
            "ScriptToken"
        )

    calling = sum(1 for v in decls.values() if v)
    if calling == 0:
        raise SystemExit(
            "token-audit.py: no description declares `call_user`, so the "
            "equality below holds trivially and checks nothing"
        )

    if bad:
        print("token-audit.py: the script token does not guard what it must:")
        for line in bad:
            print(f"  {line}")
        return 1
    print(
        f"token-audit.py: {len(decls)} runtime declarations, {calling} declaring "
        "`call_user`, each taking the script token and no other; tail_call too"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
