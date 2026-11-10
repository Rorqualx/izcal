#!/usr/bin/env python3
"""JS-10 (b): mutation-generated variants of the heap planted-bug classes.

JS-10 (a) plants one case per class. (b) asks for at least 20 mutation
variants of each, "because incomplete fixes produce variants": a
mechanism can catch the one spelling a case happens to use and miss the
family. The H-3 lint proved the point on 10 November 2026 — it read the
trait's text, so a renamed import walked past it.

A variant here is not a random edit. For each class the axes below are
the independent ways the same bug can be spelled, chosen so that a
narrower mechanism would catch some values and miss others: which GC
point, which API produced the reference, what holds it, how it is used
again. The set is then

  * the full cross product, where that is at most FULL_PRODUCT cases;
  * a greedy pairwise covering array otherwise (every value of every
    axis, and every pair of values of every two axes, appears at least
    once), which is what makes "at least 20" a property rather than a
    count.

Every variant is emitted with its **twin**: the same program with the
single bug-introducing step undone — the GC point moved after the last
use, the foreign heap's mutator replaced by the local one, the raw
field's type replaced by a sealed one. The runner requires the variant
to be refused and the twin to compile, so a refusal is attributed to the
planted bug and not to the scaffolding. That guard is what makes a
hundred generated files worth as much as a hand-read one: in #83 four
hand-written cross-heap cases were refused for the wrong reason, and
only reading the diagnostics found it.

Usage:
    mutants.py --out DIR [--sample]

`--sample` emits one variant per axis value per class (the local gate's
set) instead of the full covering array. The output is a directory per
class plus `manifest.tsv`, whose columns are

    class  id  outcome  code  note  variant  twin  axes

where `outcome` is `refused` or `compiles`; a `compiles` row names in
`note` the mechanism that catches it instead, because a variant that
compiles and is not caught anywhere is an escape, not a pass.
"""

import argparse
import os
import sys

FULL_PRODUCT = 64
"""Above this many combinations the set is a pairwise covering array."""

MIN_PER_CLASS = 20
"""JS-10 (b)'s floor, checked here so a thinned axis cannot pass quietly."""

# --------------------------------------------------------------------------
# class: an unrooted reference across a GC point
# --------------------------------------------------------------------------

UNROOTED_AXES = {
    # Every public way to obtain a `Gc`. A mechanism that branded only
    # allocation's result would miss the others.
    "from": {
        "root-get": 'r.get(m)',
        "field": 'r.get(m).get(m, Cell::NEXT).expect("next")',
        "weak": 'r.get(m).get_weak(m, Cell::SEEN).expect("seen")',
        "erase-cast": 'r.get(m).erase().cast::<Cell>(m).expect("cast")',
        "fresh-alloc": 'm.alloc::<Cell>(s, 0).expect("alloc").get(m)',
    },
    # The four GC points. A mechanism that took `&mut` on allocation
    # alone would miss a cycle run on the spot.
    "point": {
        "alloc": 'let _ = m.alloc::<Cell>(s, 0).expect("alloc");',
        "charge": "m.charge(1);",
        "safepoint": "m.safepoint();",
        "collect": "m.collect();",
    },
    # What holds the reference across the point. A rule keyed on a local
    # binding of type `Gc` would miss every one of these but the first.
    "held-in": {
        "bare": ("let held = g;", "held"),
        "tuple": ("let held = (g, 0u32);", "held.0"),
        "array": ("let held = [g];", "held[0]"),
        "option": ('let held = Some(g);', 'held.expect("some")'),
        "vec": ("let held = vec![g];", "held[0]"),
        "boxed": ("let held = Box::new(g);", "(*held)"),
        "struct": ("let held = Holder { g };", "held.g"),
        "closure": ("let held = move || g;", "held()"),
    },
    # How it is used after the point: the use is what the borrow
    # checker needs in order to see the reference live across the point.
    "used-by": {
        "len": "let _ = {h}.len(m);",
        "read": "let _ = {h}.get(m, Cell::NEXT);",
        "erase": "let _ = {h}.erase();",
        "helper": "takes({h});",
    },
    # The control flow the live range crosses.
    "flow": {
        "straight": ("", ""),
        "loop": ("for _ in 0..2 {", "}"),
        "cond": ("if 1 + 1 == 2 {", "}"),
        "match": ("match 1u32 { _ => {", "} }"),
    },
}

UNROOTED_PREAMBLE = '''use hg_heap::{Gc, Heap};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        weak SEEN: Cell,
        tail none,
    }
}

/// A heap reference held in a field of a plain Rust struct.
pub struct Holder<'h, 'm> {
    #[allow(dead_code)]
    g: Gc<'h, 'm, Cell>,
}

/// A reference passed on by value.
fn takes(_g: Gc<'_, '_, Cell>) {}
'''


def render_unrooted(sel):
    """The variant and its twin: the twin moves the GC point after the use."""
    g = UNROOTED_AXES["from"][sel["from"]]
    point = UNROOTED_AXES["point"][sel["point"]]
    bind, read = UNROOTED_AXES["held-in"][sel["held-in"]]
    use = UNROOTED_AXES["used-by"][sel["used-by"]]
    open_, close = UNROOTED_AXES["flow"][sel["flow"]]

    use = use.format(h=read)

    def body(order):
        lines = []
        if open_:
            lines.append(" " * 12 + open_)
        pad = " " * (16 if open_ else 12)
        # The reference is made first and then put somewhere, so that
        # the `held-in` axis varies what holds it and nothing else: a
        # container that computed it instead would capture the mutator.
        lines.append(pad + f"let g = {g};")
        lines.append(pad + bind)
        for line in order:
            lines.append(pad + line)
        if close:
            lines.append(" " * 12 + close)
        return "\n".join(lines)

    def whole(order, head):
        return (
            head
            + UNROOTED_PREAMBLE
            + "\npub fn case(a: &mut Heap) {\n"
            + "    a.enter(|m| {\n"
            + "        m.scope(|m, s| {\n"
            + '            let r = m.alloc::<Cell>(s, 0).expect("alloc");\n'
            + body(order)
            + "\n        });\n    });\n}\n"
        )

    return whole([point, use], ""), whole([use, point], "")


# --------------------------------------------------------------------------
# classes: the four cross-heap cases of JS-D18
# --------------------------------------------------------------------------
#
# Two heaps are entered, each with its own brand, and one heap's
# reference is handed to the other heap's API. The brands are separate
# function parameters, which is the shape two nested `Heap::enter` calls
# give; the nested spelling itself is refused a second time over by
# `enter`'s `for<'h>` signature, with the brand never consulted (measured
# 9 November 2026), so it stays a hand-written case of its own
# (`fail-cross-heap-nested-enter`) and is not an axis here: a variant
# whose refusal has two causes cannot attribute either.

CROSS_AXES = {
    # Which heap's mutator receives the other heap's reference. A brand
    # check written one way round would catch one direction only.
    "direction": {"y-into-x": ("a", "b"), "x-into-y": ("b", "a")},
    # Where the two brands meet. Through a helper the error lands at the
    # call; through a generic or a trait method it lands in inference.
    "through": {
        "direct": "direct",
        "helper": "helper",
        "generic": "generic",
        "closure": "closure",
    },
    # What carries the foreign reference to the call.
    "carried-by": {
        "bare": "{g}",
        "option": 'Some({g}).expect("some")',
        "vec": 'vec![{g}][0]',
    },
}

CROSS_PREAMBLE = '''use hg_heap::{ExternalSlot, Gc, Mutator, Scope};

hg_heap::object! {
    /// A list cell.
    pub Cell {
        traced NEXT: Cell,
        tail none,
    }
}
'''

# Per case: the operation, written four ways. `{m}` is the mutator whose
# heap the operation belongs to, `{own}` a reference of that heap, `{it}`
# the reference handed in, `{s}` that heap's scope and `{slot}` its
# external slot.
CROSS_CASES = {
    "cross-heap-store": {
        "why": "a reference of one heap stored into an object of the other",
        "direct": "{own}.set({m}, Cell::NEXT, Some({it}));",
        "helper": (
            "fn store<'h>(m: &Mutator<'h>, own: Gc<'h, '_, Cell>, it: Gc<'h, '_, Cell>) {\n"
            "    own.set(m, Cell::NEXT, Some(it));\n}\n",
            "store({m}, {own}, {it});",
        ),
        "generic": (
            "fn store_any<'h, T: hg_heap::Referent>(\n"
            "    m: &Mutator<'h>,\n"
            "    own: Gc<'h, '_, Cell>,\n"
            "    it: Gc<'h, '_, T>,\n"
            ") where\n"
            "    Cell: hg_heap::Object,\n"
            "{\n"
            "    let _ = (m, own, it);\n}\n",
            "store_any({m}, {own}, {it});",
        ),
        "closure": (
            "",
            "let store = |m: &Mutator<'{lt}>, own: Gc<'{lt}, '_, Cell>, it: Gc<'{lt}, '_, Cell>| {{\n"
            "        own.set(m, Cell::NEXT, Some(it));\n"
            "    }};\n"
            "    store({m}, {own}, {it});",
        ),
    },
    "cross-heap-read": {
        "why": "a reference of one heap read through the other heap's mutator",
        "direct": "let _ = {it}.len({m});",
        "helper": (
            "fn read<'h>(m: &Mutator<'h>, it: Gc<'h, '_, Cell>) -> u32 {\n"
            "    it.len(m)\n}\n",
            "let _ = read({m}, {it});",
        ),
        "generic": (
            "fn read_any<'h, T: hg_heap::Referent>(m: &Mutator<'h>, it: Gc<'h, '_, T>) -> u32 {\n"
            "    it.len(m)\n}\n",
            "let _ = read_any({m}, {it});",
        ),
        "closure": (
            "",
            "let read = |m: &Mutator<'{lt}>, it: Gc<'{lt}, '_, Cell>| it.len(m);\n"
            "    let _ = read({m}, {it});",
        ),
    },
    "cross-heap-root": {
        "why": "a reference of one heap rooted in a scope of the other",
        "direct": "let _ = {s}.root({m}, {it});",
        "helper": (
            "fn hold<'h, 's>(m: &Mutator<'h>, s: &Scope<'h, 's>, it: Gc<'h, '_, Cell>) {\n"
            "    let _ = s.root(m, it);\n}\n",
            "hold({m}, {s}, {it});",
        ),
        "generic": (
            "fn hold_any<'h, 's, T: hg_heap::Referent>(\n"
            "    m: &Mutator<'h>,\n"
            "    s: &Scope<'h, 's>,\n"
            "    it: Gc<'h, '_, T>,\n"
            ") {\n"
            "    let _ = s.root(m, it);\n}\n",
            "hold_any({m}, {s}, {it});",
        ),
        "closure": (
            "",
            "let hold = |m: &Mutator<'{lt}>, s: &Scope<'{lt}, '_>, it: Gc<'{lt}, '_, Cell>| {{\n"
            "        let _ = s.root(m, it);\n"
            "    }};\n"
            "    hold({m}, {s}, {it});",
        ),
    },
    "cross-heap-external": {
        "why": "a reference of one heap written into the other heap's external traced slot",
        "direct": "{slot}.set({m}, Some({it}));",
        "helper": (
            "fn park<'h>(m: &Mutator<'h>, slot: &ExternalSlot<Cell>, it: Gc<'h, '_, Cell>) {\n"
            "    slot.set(m, Some(it));\n}\n",
            "park({m}, {slot}, {it});",
        ),
        "generic": (
            "fn park_any<'h, T: hg_heap::Object>(\n"
            "    m: &Mutator<'h>,\n"
            "    slot: &ExternalSlot<T>,\n"
            "    it: Gc<'h, '_, T>,\n"
            ") {\n"
            "    slot.set(m, Some(it));\n}\n",
            "park_any({m}, {slot}, {it});",
        ),
        "closure": (
            "",
            "let park = |m: &Mutator<'{lt}>, slot: &ExternalSlot<Cell>, it: Gc<'{lt}, '_, Cell>| {{\n"
            "        slot.set(m, Some(it));\n"
            "    }};\n"
            "    park({m}, {slot}, {it});",
        ),
    },
}


def render_cross(case):
    """A renderer for one of the four cross-heap cases."""

    def render(sel):
        host, other = CROSS_AXES["direction"][sel["direction"]]
        carrier = CROSS_AXES["carried-by"][sel["carried-by"]]
        shape = CROSS_CASES[case][sel["through"]]
        helper, call = ("", shape) if isinstance(shape, str) else shape

        def whole(src):
            # `src` is the heap whose reference is handed to `host`.
            foreign = carrier.format(g=f"g{src}")
            body = call.format(
                m=f"m{host}",
                own=f"g{host}",
                it=foreign,
                s=f"s{host}",
                slot=f"slot{host}",
                lt="x" if host == "a" else "y",
            )
            return (
                CROSS_PREAMBLE
                + "\n"
                + helper
                + "\n#[allow(clippy::too_many_arguments)]\n"
                + "pub fn case<'x, 'y>(\n"
                + "    ma: &Mutator<'x>,\n"
                + "    sa: &Scope<'x, '_>,\n"
                + "    slota: &ExternalSlot<Cell>,\n"
                + "    ga: Gc<'x, '_, Cell>,\n"
                + "    mb: &Mutator<'y>,\n"
                + "    sb: &Scope<'y, '_>,\n"
                + "    slotb: &ExternalSlot<Cell>,\n"
                + "    gb: Gc<'y, '_, Cell>,\n"
                + ") {\n"
                + "    let _ = (ma, sa, slota, ga, mb, sb, slotb, gb);\n"
                + f"    {body}\n"
                + "}\n"
            )

        # The twin hands the host heap its own reference: one identifier.
        return whole(other), whole(host)

    return render


# --------------------------------------------------------------------------
# class: a raw pointer in a heap type
# --------------------------------------------------------------------------
#
# `RawValue` is sealed, so no type of the author's own can be a raw word.
# The variants are the ways to try, and the ways to carry an address
# without trying: those compile, and name the mechanism that stops them
# being usable instead.

RAW_AXES = {
    "pointer": {
        "const-ptr": "*const u8",
        "mut-ptr": "*mut u8",
        "nonnull": "core::ptr::NonNull<u8>",
        "fn-ptr": "fn()",
        "static-ref": "&'static u8",
        "cell-ptr": "*const core::cell::Cell<u8>",
    },
    "wrapped-in": {
        "newtype": "pub struct Smuggled({p});",
        "transparent": "#[repr(transparent)]\npub struct Smuggled({p});",
        "named-field": "pub struct Smuggled {{\n    pub at: {p},\n}}",
        "array": "pub struct Smuggled([{p}; 1]);",
    },
}

RAW_PREAMBLE = """use hg_heap::RawValue;

"""

RAW_OBJECT = """hg_heap::object! {{
    /// A cell with one raw word.
    pub Leaky {{
        raw SMUGGLED: {ty},
        tail none,
    }}
}}
"""


def render_raw(sel):
    """The variant implements `RawValue` for a pointer-bearing type; the
    twin keeps the whole shape and stores a `u32` instead."""
    p = RAW_AXES["pointer"][sel["pointer"]]
    shape = RAW_AXES["wrapped-in"][sel["wrapped-in"]]

    variant = (
        RAW_PREAMBLE
        + "#[derive(Clone, Copy)]\n"
        + shape.format(p=p)
        + "\n\nimpl RawValue for Smuggled {\n"
        + "    fn to_bits(self) -> u64 {\n        0\n    }\n\n"
        + "    fn from_bits(_bits: u64) -> Self {\n"
        + "        unimplemented!()\n    }\n}\n\n"
        + RAW_OBJECT.format(ty="Smuggled")
    )
    # The twin undoes the one step: the raw word's type is a sealed one,
    # so the type of the author's own and the impl it needed both go. A
    # twin that kept the type and changed only the pointer inside it
    # would be refused as well -- the seal is on the type, not on what
    # it holds -- and would prove nothing (measured 10 November 2026).
    twin = (
        "//! The twin: the raw word is a sealed type, so no impl of the\n"
        "//! author's own is needed and the seal has nothing to refuse.\n"
        + RAW_OBJECT.format(ty="u32")
    )
    return variant, twin


# Variants that compile by design: an address fits in a sealed raw word,
# and the seal never claimed otherwise. Each names what stops it.
RAW_COMPILING = [
    (
        "address-in-u64",
        "u64",
        "ptr as u64",
        "the seal makes a pointer type unsayable, not a bit pattern; using "
        "this word as an address needs `unsafe`, which the workspace denies "
        "and the graph lint inventories (G5, expect-unsafe)",
    ),
    (
        "address-in-u32",
        "u32",
        "ptr as usize as u32",
        "the same, truncated: a 32-bit raw word cannot hold a host address "
        "on any target Izcal builds for",
    ),
    (
        "address-in-f64",
        "f64",
        "f64::from_bits(ptr as u64)",
        "the same through a float: `f64::to_bits` is the raw word, and the "
        "address is still unusable without inventoried `unsafe`",
    ),
]

RAW_COMPILING_TEMPLATE = """//! A host address carried in a sealed raw word ({ty}). This compiles,
//! and must: {why}.
use hg_heap::Mutator;

hg_heap::object! {{
    /// A cell with one raw word.
    pub Carrier {{
        raw WORD: {ty},
        tail none,
    }}
}}

pub fn case(m: &mut Mutator<'_>) {{
    m.scope(|m, s| {{
        let r = m.alloc::<Carrier>(s, 0).expect("alloc");
        let ptr = &0u8 as *const u8;
        r.get(m).set_raw(m, Carrier::WORD, {expr});
    }});
}}
"""


# --------------------------------------------------------------------------
# class: a hand-written heap object impl
# --------------------------------------------------------------------------
#
# `hg_heap::object!` is the only author of a layout (H-3). The trait
# cannot be sealed, because the macro expands in the calling crate, so
# the impl is made to name `Object::LAYOUT_WITNESS` instead: leaving it
# out is `E0046` whatever spelling the impl uses, including the macro
# argument that walked past all three of H-3's text rules.
#
# The twin here adds the witness, which is the one step undone and
# nothing else. It compiles -- and is itself a finding, of H-3's
# `heap-object-witness` rule and of clippy's `disallowed-methods`, which
# `crates/hg-graph-lint/tests/it/rules.rs` covers. So the pair reads:
# without the witness the compiler refuses it, with the witness the lint
# reports it, and there is no third spelling.

HAND_AXES = {
    # How the impl names the trait. A rule that reads the source catches
    # the first three and misses the fourth.
    "spelled": {
        "import": ("use hg_heap::Object;\nuse hg_heap::Referent;\n", "Object", "Referent"),
        "full-path": ("", "hg_heap::Object", "hg_heap::Referent"),
        "renamed": (
            "use hg_heap::{Object as Obj, Referent as Ref};\n",
            "Obj",
            "Ref",
        ),
        "macro-arg": ("", "@macro", "hg_heap::Referent"),
    },
    # The impl's own shape.
    "shaped": {
        "concrete": "concrete",
        "generic": "generic",
        "cfg-gated": "cfg-gated",
    },
    # Where it sits.
    "placed": {"root": "root", "module": "module", "function": "function"},
}

HAND_LAYOUT = """    const LAYOUT: hg_heap::Layout = hg_heap::Layout {
        name: "Forged",
        traced: 8,
        weak: 0,
        nodes: 0,
        raw_words: 0,
        tail: hg_heap::Tail::None,
    };
"""

HAND_WITNESS = (
    "    const LAYOUT_WITNESS: hg_heap::LayoutWitness = hg_heap::layout_witness();\n"
)

HAND_MACRO = """macro_rules! plant {
    ($t:path, $ty:ty) => {
        impl $t for $ty {
%s        }
    };
}
"""


def render_hand(sel):
    """The variant omits the witness (E0046); the twin adds it."""
    imports, obj, ref = HAND_AXES["spelled"][sel["spelled"]]
    shape = sel["shaped"]
    place = sel["placed"]

    def whole(witness):
        body = (HAND_WITNESS if witness else "") + HAND_LAYOUT
        if obj == "@macro":
            head = HAND_MACRO % _indent(body, 4)
            subject = "Forged"
            impl = f"plant!(hg_heap::Object, {subject});"
        else:
            head = ""
            if shape == "concrete":
                impl = f"impl {obj} for Forged {{\n{body}}}"
            elif shape == "generic":
                impl = (
                    f"impl<T: hg_heap::Referent> {obj} for Wrap<T> {{\n{body}}}"
                )
            else:
                impl = f'#[cfg(not(test))]\nimpl {obj} for Forged {{\n{body}}}'
        decls = "pub enum Forged {}\n"
        if shape == "generic" and obj != "@macro":
            decls += (
                "pub enum Wrap<T> {\n    Nothing(core::marker::PhantomData<T>),\n}\n"
                f"impl<T: hg_heap::Referent> {ref} for Wrap<T> {{}}\n"
            )
        else:
            decls += f"impl {ref} for Forged {{}}\n"
        inner = imports + decls + head + impl + "\n"
        if place == "root":
            return inner
        if place == "module":
            return "pub mod planted {\n" + _indent(inner, 4) + "}\n"
        return (
            "pub fn planted() {\n" + _indent(inner, 4) + "}\n"
        )

    return whole(False), whole(True)


def _indent(text, n):
    pad = " " * n
    return "".join(pad + line if line.strip() else line for line in text.splitlines(True))


CLASSES = [
    {
        "name": "unrooted-gc",
        "why": "an unrooted reference held across a GC point",
        "mechanism": "compile: the mutator is already borrowed",
        "outcome": "refused",
        "code": "E0502",
        "note": "",
        "axes": UNROOTED_AXES,
        "render": render_unrooted,
    },
]

for _case, _spec in CROSS_CASES.items():
    CLASSES.append(
        {
            "name": _case,
            "why": _spec["why"],
            "mechanism": "compile: the two heaps' brands do not unify",
            "outcome": "refused",
            "code": "LIFETIME",
            "note": "",
            "axes": CROSS_AXES,
            "render": render_cross(_case),
        }
    )

CLASSES.append(
    {
        "name": "hand-written-object",
        "why": "a heap object layout written by hand instead of by `object!`",
        "mechanism": "compile: the impl has no layout witness",
        "outcome": "refused",
        "code": "E0046",
        "note": "",
        "axes": HAND_AXES,
        "render": render_hand,
    }
)

CLASSES.append(
    {
        "name": "raw-pointer",
        "why": "a raw pointer stored in a heap type's raw word",
        "mechanism": "compile: `RawValue` is sealed",
        "outcome": "refused",
        "code": "E0277",
        "note": "",
        "axes": RAW_AXES,
        "render": render_raw,
    }
)


# --------------------------------------------------------------------------
# the covering array
# --------------------------------------------------------------------------


def product(axes):
    """Every combination, in a fixed order."""
    names = list(axes)
    rows = [{}]
    for n in names:
        rows = [dict(r, **{n: v}) for r in rows for v in axes[n]]
    return rows


def pairwise(axes):
    """A greedy pairwise covering array: every value of every axis and
    every pair of values of every two axes appears in some row. Greedy
    and deterministic, so the suite is the same on every machine."""
    names = list(axes)
    want = set()
    for i, a in enumerate(names):
        for b in names[i + 1 :]:
            for va in axes[a]:
                for vb in axes[b]:
                    want.add((a, va, b, vb))
    rows = []
    while want:
        # Seed the row from an uncovered pair, so every row covers at
        # least the pair it was seeded from and the loop always ends.
        sa, sva, sb, svb = sorted(want)[0]
        row = {sa: sva, sb: svb}
        for n in names:
            if n in row:
                continue
            best, best_n = None, -1
            for v in axes[n]:
                hit = sum(
                    1
                    for (a, va, b, vb) in want
                    if (a == n and va == v and row.get(b) == vb)
                    or (b == n and vb == v and row.get(a) == va)
                )
                if hit > best_n:
                    best, best_n = v, hit
            row[n] = best
        covered = {
            (a, va, b, vb)
            for (a, va, b, vb) in want
            if row.get(a) == va and row.get(b) == vb
        }
        if not covered:  # pragma: no cover - greedy always covers one
            raise SystemExit("mutants.py: the covering array made no progress")
        want -= covered
        rows.append(row)
    return rows


def sample(axes):
    """One row per axis value: the local gate's thinner set."""
    names = list(axes)
    rows = []
    for n in names:
        for v in axes[n]:
            row = {k: list(axes[k])[0] for k in names}
            row[n] = v
            if row not in rows:
                rows.append(row)
    return rows


def rows_for(axes, mode):
    if mode == "sample":
        return sample(axes)
    full = 1
    for n in axes:
        full *= len(axes[n])
    return product(axes) if full <= FULL_PRODUCT else pairwise(axes)


def slug(sel):
    return "-".join(sel[k] for k in sel)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--sample", action="store_true")
    args = ap.parse_args()
    mode = "sample" if args.sample else "full"

    manifest = []
    for c in CLASSES:
        d = os.path.join(args.out, c["name"])
        os.makedirs(d, exist_ok=True)
        rows = rows_for(c["axes"], mode)
        used = {n: set() for n in c["axes"]}
        for i, sel in enumerate(rows):
            for n, v in sel.items():
                used[n].add(v)
            variant, twin = c["render"](sel)
            head = (
                f"//! Generated by tools/mutants.py: JS-10 (b) variant of\n"
                f"//! {c['why']}.\n//!\n//! axes: "
                + " ".join(f"{k}={v}" for k, v in sel.items())
                + f"\n//!\n//! EXPECT: {c['outcome']} — {c['mechanism']}.\n"
            )
            name = f"{i:03d}-{slug(sel)}"
            vp = os.path.join(d, name + ".rs")
            tp = os.path.join(d, name + "-twin.rs")
            with open(vp, "w", encoding="utf-8") as f:
                f.write(head + variant)
            with open(tp, "w", encoding="utf-8") as f:
                f.write(
                    head.replace(f"EXPECT: {c['outcome']}", "EXPECT: compiles")
                    + "//! The twin: the same program with the planted step undone.\n"
                    + twin
                )
            manifest.append(
                (
                    c["name"],
                    name,
                    c["outcome"],
                    c["code"],
                    c["note"],
                    os.path.relpath(vp, args.out),
                    os.path.relpath(tp, args.out),
                    " ".join(f"{k}={v}" for k, v in sel.items()),
                )
            )
        # The generator's own invariant: a thinned axis is a quiet loss
        # of coverage, so it fails here rather than passing in CI.
        for n, vs in used.items():
            missing = set(c["axes"][n]) - vs
            if missing:
                raise SystemExit(
                    f"mutants.py: class {c['name']} axis {n} never used "
                    f"{sorted(missing)}"
                )
        if mode != "sample" and len(rows) < MIN_PER_CLASS:
            raise SystemExit(
                f"mutants.py: class {c['name']} has {len(rows)} variants, "
                f"under JS-10 (b)'s floor of {MIN_PER_CLASS}"
            )

    # The rows that compile by design, each citing what catches it.
    d = os.path.join(args.out, "raw-pointer")
    for name, ty, expr, why in RAW_COMPILING:
        p = os.path.join(d, f"900-{name}.rs")
        with open(p, "w", encoding="utf-8") as f:
            f.write(RAW_COMPILING_TEMPLATE.format(ty=ty, expr=expr, why=why))
        manifest.append(
            (
                "raw-pointer",
                f"900-{name}",
                "compiles",
                "",
                why,
                os.path.relpath(p, args.out),
                "",
                f"pointer=address-in-{ty} wrapped-in=none",
            )
        )

    # Tab is an IFS whitespace character, so a shell `read` collapses two
    # tabs into one and shifts every later field. An empty cell is
    # written `-` for that reason, and because a reader of the published
    # manifest should see "no twin" rather than a gap.
    with open(
        os.path.join(args.out, "manifest.tsv"), "w", encoding="utf-8"
    ) as f:
        for row in manifest:
            f.write("\t".join(c if c else "-" for c in row) + "\n")
    print(f"{len(manifest)} variants in {len(CLASSES)} classes ({mode})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
