# JS-10 (a) and (b): the heap's planted bugs, and the variants of each

10 November 2026. JS-M0 plan Phase 2 item 8; JS spec 8.2 JS-10 and 6.7; heap design review H-1, H-2, H-3. Nothing here cites or derives from any third-party source code (M-1 part 3).

JS-10 (a) asks for a published planted-bug suite with at least one case per class. (b) asks for **mutation-generated variants of every case, at least 20† per class**, "because incomplete fixes produce variants" ([P0](https://projectzero.google/2021/02/deja-vu-lnerability.html)). Pass is 100% of (a) and (b) caught by the named mechanism at the named stage.

The first variant written proved the target's point on the mechanism that had shipped the day before, and a later one proved it again. Both are recorded below, with what replaced the mechanism they escaped.

## What a variant is here

Not a random edit. For each class the **axes** are the independent ways the same bug can be spelled, chosen so that a narrower mechanism would catch some values and miss others. The set is then the full cross product where that is at most 64 cases, and a greedy pairwise covering array where it is larger — every value of every axis, and every pair of values of every two axes, appears at least once. That makes "at least 20" a property of the set rather than a count of it, and the owner chose this reading over a flat 20 on 10 November 2026.

Every variant is emitted with its **twin**: the same program with the single bug-introducing step undone. The gate requires the variant to be refused and the twin to compile, which attributes the refusal to the planted bug rather than to the scaffolding. At this size that guard is the whole difference between evidence and a green light: #83 found four hand-written cross-heap cases refused for the wrong reason by reading rustc's diagnostics, and 202 diagnostics do not get read.

## The classes, at JS-M0

| Class | Axes | Full | In the suite | Mechanism, and the stage |
|---|---|---|---|---|
| An unrooted reference across a GC point | provenance 5 × GC point 4 × container 8 × use 4 × control flow 4 | 2,560 | **43**, pairwise | compile: the mutator is already borrowed (`E0502`) |
| Cross-heap store (JS-D18 case 1) | direction 2 × call shape 4 × carrier 3 | 24 | **24**, full | compile: the brands do not unify (a lifetime error) |
| Cross-heap read (case 2) | the same | 24 | **24**, full | the same |
| Cross-heap root (case 3) | the same | 24 | **24**, full | the same |
| Cross-heap external slot (case 4) | the same | 24 | **24**, full | the same |
| A hand-written heap object impl | spelling 4 × impl shape 3 × location 3 | 36 | **36**, full | compile: the impl has no layout witness (`E0046`), then the H-3 lint and clippy for an impl that writes one |
| A raw pointer in a heap type | pointer flavour 6 × wrapper 4 | 24 | **24**, full, plus 3 that compile by design | compile: `RawValue` is sealed (`E0277`) |
| A non-atomic slot store | — | **open**, ~6 spellings | outside `hg-heap` the cage and its accessors are `pub(crate)`, so the class has six spellings and no 20 to generate; the race half needs a second thread in the heap |
| A missing barrier | — | **open**, 0 | there is no barrier to omit until marking is concurrent |

202 variants, 199 twins, 401 compilations: **23.6 s** on the Mac (M-series, serial, warm cargo build). `tools/checks.sh` runs the thinner `--sample` set outside CI — one variant per axis value, 69 variants, 7 s — and CI runs the whole array; the owner chose that split on 10 November 2026.

The two open classes are not padded to twenty. Both come due with concurrent marking at JS-M1, which is also why CS-D3 criterion 5's loom half is deferred there (#83). Writing filler variants for them would raise the count and lower the evidence.

**Not an axis: the nested-`enter` spelling of the cross-heap cases.** `Heap::enter` takes a `for<'h> FnOnce(&mut Mutator<'h>) -> R` whose result cannot mention `'h`, so the nested shape is refused a second time over with the brand never consulted (measured 9 November 2026, #83). A variant whose refusal has two causes attributes neither, so that shape stays a hand-written case of its own, `fail-cross-heap-nested-enter`.

## The two escapes

**1. A renamed import walked past H-3's rule.** `heap-object-impl` (#84) matches the trait path's *text*, so

```rust
use hg_heap::{Object as Obj, Referent as Ref};
impl Ref for Evil {}
impl Obj for Evil { const LAYOUT: Layout = Layout { ... }; }
```

compiled against `hg-heap`'s own rlib and was reported by nothing. Q-CS-1's crypto rules had carried all three spellings — impl, renamed import, macro body — since M1, five lines above in the same file; H-3 shipped with one of them. It now has the triple.

**2. A macro argument walked past all three.** With the triple in place,

```rust
macro_rules! plant { ($t:path, $ty:ty) => { impl $t for $ty {} }; }
plant!(hg_heap::Referent, Evil);
```

still compiled and was still reported by nothing: the trait's name appears only in the *invocation*, which is neither an impl, an import, nor a macro body. A fourth text rule on `macro_invocation` was measured and rejected — it costs false positives on `HostValue::Object` and on `Value::Object`, which are ordinary names in a JS engine.

What closed it instead is to stop writing rules about an avoidable name. `Object` gained

```rust
#[doc(hidden)]
const LAYOUT_WITNESS: LayoutWitness;
```

whose only value comes from `hg_heap::layout_witness()`. A hand-written impl cannot leave it out — that is `E0046`, in every spelling, including the macro argument — and cannot write it without naming `LAYOUT_WITNESS`. Those names belong to this mechanism and to nothing else in the tree, so H-3's `heap-object-witness` rule owns them, and clippy's `disallowed-methods` carries `layout_witness` as a second layer that **resolves** paths rather than reading them. `object!` calls it under an `#[expect]` of that rule, so an object type's author sees nothing: measured clean over `hg-heap` and `hg-js-runtime`, and measured firing on a hand-written impl, on a full-path impl, and inside a local macro's body.

H-3 still cannot seal the trait, for the reason the design review gives: the macro expands in the calling crate, so a private supertrait would not be nameable there and would seal nothing. The witness is what a seal would have been.

**Why clippy could not do this on the trait itself.** Measured 10 November 2026: `disallowed_types` fires on a renamed import and on a trait **bound**, but not on the trait position of `impl Trait for Type`, and not on a unit struct named in expression position. `disallowed_methods` does fire on a call, including one written by an external macro — which is why the `#[expect]` is inside `object!` rather than in the caller.

## The guards, each checked by breaking it

| Guard | Broken how | What happened |
|---|---|---|
| The variant must carry its class's error code | the fixture preamble renamed a field | 2 rows failed with "refused with E0599 but not E0502" |
| The twin must compile | the generator emitted the twin with the bug still in it | 21 rows failed with "the twin was refused too" |
| The mechanism must be the one named | `RawValue`'s seal removed | all 24 raw-pointer variants reported, not one |
| H-3's alias rule | the rule deleted | the lint test's renamed-import row failed |
| Every axis value must appear | — | the generator refuses to write a set that drops one, and the runner re-checks the per-class floor from the manifest |
| A toolchain must be present | — | no `rustc`, `cargo` or `python3` is exit 2, never a pass: without a compiler every variant would "fail to compile" |

## What is in the tree

* `tools/mutants.py` — the axis tables, the covering array, the twins, the manifest.
* `tools/mutants.sh` — compiles each variant with its twin; the `mutants` check in `tools/checks.sh`.
* `crates/hg-heap/tests/ui/` — JS-10 (a)'s hand-written heap cases, 24 refusals and 5 controls, including the two spellings that escaped; `crates/hg-js-runtime/tests/ui/` holds 3 more and 1 control.
* `crates/hg-graph-lint/tests/it/rules.rs` — H-3's lint rows, for the impls that compile and must be reported.

## For the owner

* **The two open classes.** Both are recorded here as open with their trigger rather than filled with variants that would not mean anything. They come due at JS-M1 with concurrent marking.
* **Q-CS-1's crypto rules have the same hole as H-3 had.** `crypto-impl`, `crypto-alias` and `crypto-macro` all read the trait's name, so a macro that takes the trait as an argument gets past them exactly as it got past H-3's triple. Nothing in the tree does it today. The fix is not a fourth text rule, as above; it is the same shape of unavoidable name, and the crypto trait is Izcal's own, so it can carry one. That is a gate-file change to a different target, so it is flagged here rather than made.
* **JS-10 (d), the external review, is not something this session can do.** It needs people outside the engine team, per stage.
