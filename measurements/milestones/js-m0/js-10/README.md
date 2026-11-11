# JS-10: the heap's planted bugs, their variants, and the in-cage corruption harness

10 November 2026. JS-M0 plan Phase 2 item 8; JS spec 8.2 JS-10, 6.1 and 6.7; heap design review H-1, H-2, H-3. Nothing here cites or derives from any third-party source code (M-1 part 3).

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

**1. A renamed import walked past H-3's rule.** `heap-object-impl` (#86) matches the trait path's *text*, so

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

## JS-10 (c): the in-cage corruption harness

(c) asks that **the in-cage corruption harness (Section 6.1) runs on every release**, and the pass is **0 out-of-cage faults**.

### The premise

Security scope ATK-2 assumes an attacker who reads and writes their own process's memory, the cage included, so a type rule about what Rust *can* store in the cage is necessary and not sufficient. What has to hold is H-2: a cage holding arbitrary words gives **wrong answers inside the cage**, and never an access to memory outside the cage, its side tables or a checked table entry. Section 6.1 says that claim is "tested rather than asserted". This is the test.

### The API

`Mutator::corrupt()`, in a test build only: `cfg(test)`, or the `testing` feature, which only a dev-dependency enables — that is how JS-M1's fuzzer will drive it from outside `hg-heap`, with script running. It grants arbitrary 32- and 64-bit reads and writes at any offset, a `fill` for the sweep an attacker with a write primitive does, and `confine`, which folds an offset into the chunks of the heap that owns them — for the reason the finding below gives.

It is deliberately **not** a way out of the cage. Every access it makes goes through the same masked, aligned, atomic accessors of the `cage` module that the collector uses, so what it grants is an arbitrary *value* at an arbitrary *in-cage offset*: the attacker of the premise and nothing more. The masking is itself part of what is measured.

### A negative claim needs detectors

"No access outside the cage" cannot be asserted at the end of a run, so the harness carries four detectors, and each was checked by breaking something.

| Detector | What it catches | Where it runs | Checked by |
|---|---|---|---|
| **The guard ring**: a canary either side of every `TestMemory` reservation — the cage's and the trusted region's — read back after every run. A chunk's worth, or two blocks under Miri, which interprets the scan | a write that left the cage, or a side table, by up to a chunk, and says how far past the end it reached | every `cargo test`, on all three OSes | a write made one block past the end on purpose. It is the only out-of-cage write in the tree, because no path in `hg-heap` can produce one; the ring reports it at exactly that offset (`cage::tests`) |
| **Miri** | a write that left the reservation altogether, and a race between the threads of the two-heap test | the `miri` job (`--lib`, so this harness is in it) | it needed no planted bug: on the first run it reported a real one, below |
| **A panic is a fault**, caught and classified with its seed | a corrupted value reaching a bounds check, an `unwrap` or an overflow — in a renderer, an abort | every run | the type index used unchecked: `self.types[ty]` for `self.types.get(ty)?` in `State::shape`. **142 of 144** runs reported it, naming the index — `0xDEADBEEF` and `u32::MAX` among them, the harness's own patterns coming back |
| **A 30 s budget per run**, on its own thread | a cycle that does not finish: a renderer that stops answering | every non-Miri run | a cycle replaced by `while black_box(true) {}`. **3 of 3** runs reported `TimedOut` |

**The collector's own bookkeeping needs no fifth detector.** Section 6.1's claim covers the cage's *side tables* too, and the heap's are block states, line marks and mark bits, which live in ordinary Rust data outside the cage (`space.rs`). Every index into them is bounds-checked by safe Rust, so a cage-derived index cannot write silently into one: it panics, which detector 3 catches with its seed, or it is a provenance error, which Miri catches. The one place that could write outside a Rust allocation is the `cage` module's raw base pointer, which is what the guard ring and Miri watch. The trusted region's reservation is guarded as well, so a write into *it* is reported the same way as a write past the cage.

The heap verifier is **off** during corruption, and only during it, for the reason `verify.rs` states in its own header: it assumes the cage holds what the collector and the mutator put there, which is the assumption this harness exists to break. A corrupted slot legitimately names a dead object; the verifier would call that a collector bug. The two check different properties and must not be run against each other.

### What it found: the cage has two access widths

This is the harness's first result, and it came from the detector that needed no planted bug.

The first shape of the concurrent half pointed a second thread at a live heap's cage — a `Send` handle on the corruption API — so that an attacker could write while the collector marked and evacuated, which is what "while script runs" asks for. Miri reported:

```
error: Undefined Behavior: Race condition detected between (1) 4-byte atomic store
on thread `unnamed-2` and (2) 8-byte atomic store on thread `corrupt::harnes`
  = help: overlapping unsynchronized atomic accesses must use the same access size
```

It is right. The `cage` module reaches cage memory at **two widths**: 32 bits for slots, headers, length words and tails; 64 bits for raw words, node slots, and the `copy` and `fill` that evacuation and the sweep perform over whole objects, lines and blocks — so the 64-bit accesses cover the same bytes the 32-bit ones do. Overlapping unsynchronised atomic accesses of *different* sizes are undefined whatever their ordering, so **atomicity is not what makes the cage safe to share between threads**. What makes it safe is that no two threads reach the same cage word: an agent heap owns the chunks it was handed, a `Heap` is `!Send`, a `Mutator` is `!Send`/`!Sync`, and collection stops the world.

Three things followed.

1. **The comment on `unsafe impl Send for Cage` was wrong about why it was safe.** It gave atomicity as the reason — "every access through it is atomic … so agent heaps on different threads may share the cage". The reason is disjointness. The comment now says so, and names the two widths.
2. **The `Send` handle was withdrawn rather than documented around.** A fuzzer cannot know which width a given word is reached at, so there was no sound way to use it; an API whose only correct use is "don't" is a trap. What replaces it is `Corrupt::confine` and a test in the shape the design does license: **two agent heaps in one cage, on two threads, each corrupting only its own chunks**, which is exactly the claim the `unsafe impl` makes, and which Miri now reads for races in the chunk pool and the sharing.

   Withdrawing it needed more than deleting a type. `Corrupt`'s two fields are both `Sync`, so the handle was still `Send` and `Sync` by inference: the prose said withdrawn and the compiler said otherwise, which is the gap this milestone exists to close. It now carries a `PhantomData<*const ()>`, with a `const` assertion that the marker is still there — so losing it fails the **build**, not a test run. Checked like the four detectors, by breaking it: with the marker made inert, `hg-heap` does not compile, and the `E0080` carries the consequence rather than a type name. The crate's `ui` fixtures would be the usual home for that check, but they compile `hg-heap` without the `testing` feature, where this module does not exist, so the fixture would be refused for the wrong reason and still read as a pass.
3. **The constraint binds any concurrent collector**, which is the part that matters beyond this harness. A concurrent *marker* is unaffected: a slot is reached at 32 bits by whoever reaches it. A concurrent *evacuator* is not: `copy` writes 64-bit words over bytes the mutator reads as 32-bit slots. JS-M0 stops the world, so nothing is owed now.

Nothing about the shipping engine changed, because nothing in it shares a cage word across threads. What changed is that the reason is now written down where the `unsafe` lives, and the question is on the owner's list before the collector becomes concurrent.

### What a run does

A run is a seeded script of operations against one heap in a 4 MiB cage — four chunks, so it fills, frees, reuses and runs out of room for evacuation copies within one run; one chunk under Miri, where the sweep's poison is interpreted store by store: allocate, link, set slots, raw words, tails and bytes, read every field back, create and append and detach nodes, root and unroot them, drain the release list, charge, safepoint, collect, keep alive, take the cleared holders, drop a root, walk the whole heap for a digest, grow the trusted region and end entries in it — and corrupt.

Nothing is checked against an expected answer. Under arbitrary cage contents the answers are meaningless by design: H-2 promises they are wrong, not that they are anything in particular. What a run is checked for is the four faults above.

The corrupting write is aimed four ways, because a uniformly drawn offset into a cage almost never lands on a live object — a fuzzer that drew only uniformly would test almost nothing.

| Aim | Share | Where it writes |
|---|---|---|
| `Word` | 50% | a word of a live object: its header, its length word, a slot, a raw word, a tail element |
| `Near` | 20% | within a block either side of a live object: its neighbours, the line and block edges, the space between objects |
| `Chunk` | 20% | anywhere in a chunk this heap holds |
| `Wild` | 10% | anywhere in the cage, committed or not |

and with patterns that include zero, all ones, the sweep's poison, `1` (a reference to cage offset 0), `u32::MAX` (the highest offset a slot can name) and **a forged reference to a real live object**, which is the shape an attacker writes to make the collector follow something.

### The figures

48 runs of 400 operations per collector mode, so 144 runs, plus the two-heap test's 2. Measured 10 November 2026 on hueyj (Linux x86_64, rust 1.99.1):

| | Runs | Corrupting writes | Objects | Cycles | Objects moved |
|---|---|---|---|---|---|
| In-cage corruption | 144 | 5,858 | 12,699 | 8,198 | 297,302 |
| Two agent heaps, two threads | 2 | 43 | 86 | 65 | 997 |

**5.08 s** for the whole `hg-heap` lib suite, 50 tests. **0** faults of any of the four kinds.

What the corruption actually reached, in those 144 runs:

* **1,665 field accesses refused** because the object's header no longer named its type — the checked cast declining to answer, which is H-2 behaviour and the clearest sign the writes landed where they were aimed.
* **655 trusted-region lookups answered and 1,952 refused** by the bounds, kind or generation check, through a code-table index held in a cage word. Every answer was checked to be a live entry of the kind asked for.
* **297,302 objects moved**: evacuation ran, repeatedly, over corrupted contents.

The census exists because a harness whose writes stop landing would otherwise pass by doing nothing. It caught exactly that on the day: `Holder` was added to the object kinds and the allocator still drew from `rng.below(6)` over a seven-entry table, so the trusted-region path reported **0 reached and 0 refused** until the draw came from the table's own length.

### Two properties that needed their own tests

Both are in `corrupt.rs` beside the fuzzer, because the fuzzer is the wrong instrument for them.

1. **A corrupted length word is clamped to the object's block.** The size a cycle traces comes from the layout and the length word, and the length word is cage data, so `State::shape` clamps it to the room the block leaves. `Array`'s layout alone would make a length of `u32::MAX` mean about 16 GiB of tail slots, in a 4 MiB cage. The failure mode is not a fault any detector catches: an unclamped size has the tracer build a slot list of billions of entries and the process dies of an allocation failure, which looks like a crashed test runner rather than a finding with a seed.
2. **A trusted index held in the cage cannot outlive its entry.** Live, it reaches its entry and only under its own kind; ended, it reaches nothing; and when its index is reused under a new generation, the old bits do not reach the new occupant. Measured with the liveness check deleted from `TrustedRegion::get`: the fuzzer reported **4 of the 48** runs it then had, and this test reported every time. A property with that narrow a window belongs in a test that names it.

### Where it runs

| | Runs per mode | Operations | Modes | Cost |
|---|---|---|---|---|
| `cargo test -p hg-heap`, three OSes, and CI | 48 | 400 | all three | 5 s |
| The `miri` job | 1 | 30 | the moving test collector only | 106 s |

Miri's 106 s is the fuzzer at 50.4 s, the two-heap test at 36.7 s and the trusted index at 18.5 s, measured on a Linux x86_64 host; the clamp test is ignored there, because a bound over cage data is not an unsafe-code or a concurrency property. That is below the existing suite's per-test average under Miri. The budget goes to 80 minutes in the same change, for a reason the harness did not create: over the twelve runs before it, the job took **30.9 to 54.9 minutes, median 43.8** — a 24-minute spread on an unchanged suite, the worst run at 92% of a 60-minute cap. A job that varies by that much against a hard cap fails on an unlucky runner and says nothing about the code when it does. The run that merged this PR took 40.9 minutes *with* the three new tests, which is the same point from the other side.

**The 106 s is the harness after it was sized for Miri.** As first written it measured **3,109 s** — 52 minutes — on the same host, against a job whose worst run was already 54.9 minutes: no budget here would have absorbed it, and the honest alternative to sizing it was not running it under Miri at all. Three changes account for the difference: Miri sees one run of 30 operations rather than two of 60, the guard ring is two blocks rather than a chunk, and the ring is read as `u64` words rather than bytes. Only the ring's size was measured on its own (the table below); the run count and the word-wise scan were not, so the 29× is reported whole rather than apportioned to any one of them.

Two guesses at what made it slow, both measured rather than assumed, with opposite answers:

| Guess | Measured | Kept? |
|---|---|---|
| The sweep's poison — the cage's size in interpreted 8-byte stores, every cycle — so give Miri a one-chunk cage | 117 s against 118 s | **No.** The `cfg` bought nothing and was taken out |
| The guard ring's read-back, a chunk of bytes per reservation after every run | 106 s against **257 s** with a chunk-sized ring, the trusted-index test alone going 18.5 s to 78.9 s | **Yes.** Two blocks under Miri, and the scan reads eight bytes at a time |

The second is why the ring is read as `u64` words rather than bytes, and why its size has a `cfg` while the cage's does not.

JS-10 names (c) as "nightly in T2". There is no nightly tier yet, so it runs in **every** CI run instead, which is more often, not less; when T2 exists, the longer sweep belongs there and this one stays where it is.

One size, unlike (b)'s variants, which are sampled locally and run in full in CI. That split buys something when the full set costs 23.6 s of compilation; this suite costs 5 s, so a local run gets the coverage CI gets. It also keeps the harness out of `std::env`, which G3 bans in engine code.

### Limits, stated

* `TestMemory` backs the cage with an ordinary allocation, so a write to an uncommitted page **inside** the cage does not fault as it would on a real reservation. On a real one that is an in-cage fault, which H-2 allows and JS-10 does not count. The harness tests confinement, not the page protections.
* There is no script at JS-M0, so "a fuzzer gets arbitrary read and write inside the cage while script runs" is here "while the heap allocates, mutates, collects, evacuates, clears weak slots and traces a node space". JS-50's corpus run at JS-M1 adds the script half, and the `testing` feature is what lets the fuzzer live outside `hg-heap` by then.
* Of Section 6.1's six mechanisms, the harness exercises 1 (no raw pointer representable), 2 (everything from the cage re-checked), 3 and 4 (the trusted region, and typed entries reached by a checked index). Mechanism 5 (buffer bounds from the buffer table) and 6 (machine code addressing through base registers) have nothing in the tree to exercise yet.

## For the owner

* **The two open classes.** Both are recorded here as open with their trigger rather than filled with variants that would not mean anything. They come due at JS-M1 with concurrent marking.
* **Q-CS-1's crypto rules have the same hole as H-3 had.** `crypto-impl`, `crypto-alias` and `crypto-macro` all read the trait's name, so a macro that takes the trait as an argument gets past them exactly as it got past H-3's triple. Nothing in the tree does it today. The fix is not a fourth text rule, as above; it is the same shape of unavoidable name, and the crypto trait is Izcal's own, so it can carry one. That is a gate-file change to a different target, so it is flagged here rather than made.
* **The cage's two access widths, before the collector becomes concurrent.** Recorded above and in the design review's unsafe-code section. A concurrent evacuator's 64-bit `copy` over words the mutator reads as 32-bit slots is a mixed-size overlapping access, which is undefined whatever the ordering. The ways out are a single cage access width — every 64-bit value read and written as two 32-bit halves, which H-2 already allows to tear — or a width-matched copy. The first costs an extra load on every raw-word read, every heap double among them, so it is a decision with an interpreter cost rather than a bug fix, and it is the owner's. Nothing is owed at JS-M0.
* **JS-10 (d), the external review, is not something this session can do.** It needs people outside the engine team, per stage.
