# The five M0 starts: the evidence, clause by clause

13 November 2026. Measured against the head of the M1-X1 stack,
`js/nw-m0-foundations`. Nothing here cites or derives from any
third-party source code (M-1 part 3).

Clause 4 of M1's exit (Stage 1 §8.1) reads *"ST-M0, TX-M0, LY-M0, GR-M0
and NW-M0 start."* `measurements/milestones/m1-exit/README.md` recorded
it as the one clause of seven that did not hold, and said why the
clause could not be settled even in principle: **the row never said what
"start" means.** Stage 1 §8.1a now defines it, in five clauses that
turn on artefacts in the tree, and `hg-graph-lint --started` evaluates
them on every pull request.

This document is the evidence for that rule's answer, and it is also
**clause 5's own artefact** — the interlock table, in one place, is
§3.

## The verdict, up front

| Track | 1 crates | 2 generator | 3 oracle | 4 review | 5 interlock | Started |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **TX-M0** | + | + | + | + | + | **yes** |
| **ST-M0** | + | + | + | + | + | **yes** |
| **LY-M0** | + | + | + | + | + | **yes** |
| **GR-M0** | + | + | + | + | + | **yes** |
| **NW-M0** | + | + | + | + | + | **yes** |

**All five have started, so clause 4 of M1's exit holds.** It is
checked by `hg-graph-lint --started` on every pull request, not by this
document; what this document does is show the artefact behind each `+`
so that the rule can be audited rather than trusted.

Two things are worth stating before the evidence, because they are the
easiest things to misread in it.

- **"Started" is not "nearly done".** TX-M0's own exit has four clauses
  that need a type-design program of about 45† staff, and LY-M0's needs
  a named graphics owner. Both are **D-S8**, which Stage 1 §13.1
  records as deferred by the owner on 14 October 2026. Each track's
  review document says so in its own §5; §4 here collects it.
- **Two of the three oracles report a classification rather than a pass
  rate**, on purpose. `wpt-layout` says 0 of 9,244 tests are runnable,
  because there is no HTML tree builder yet. A pass rate of zero
  reported as a result would be evidence of nothing, so what is held
  against regression is the classification.

---

## 1. The definition, and how it is checked

Stage 1 §8.1a's five clauses, and the artefact each turns on:

| # | Clause | Artefact | Checked by |
|---|---|---|---|
| 1 | **Crates exist** | every crate the M0 row names is a workspace member with a `tools/tiers.toml` row | `hg-graph-lint --started`, against the resolved graph |
| 2 | **Its generator or contract type compiles** | the crate the rest of the track generates from or builds on | membership, plus the generator's own freshness test (G29) |
| 3 | **One oracle runs in CI** | a named job in `.github/workflows/checks.yml`, on data pinned by SHA-256 in `tools/test-data.toml` | the job's presence, and the recorded number in the crate |
| 4 | **Its design review is open with its agenda fixed** | `research_notes/<track>-M0 design review/README.md` | the file exists |
| 5 | **Its cross-track obligations are recorded** | one interlock table | the file exists — §3 of this document |

The table the rule reads is `tools/graph-rules.toml`'s `[[started]]`
rows. A row is **checked** only when `claimed = true`; an unclaimed
track is reported and never failed, so the rule cannot block the work
that satisfies it. Setting `claimed` is a gate-file change (CS-GA-1)
and it fails unless all five clauses hold.

What the rule does **not** check is whether the artefacts are any good.
Clause 3 is the only one with a number in it, and that number is in the
crate rather than in the rule. §2 gives each.

---

## 2. Per track

### 2.1 TX-M0 — fonts and text

| # | Evidence |
|---|---|
| 1 | `hg-ucd`, `hg-unicode`, `hg-unicode-build`, `hg-fontgen`, `hg-fontfmt`, `hg-fontc`, each with a `tiers.toml` row. `hg-graph-lint --locked` clean on three triples, both feature modes. |
| 2 | **`hg-fontgen`**: the `.hgfont` language, and five outputs — the zero-copy reader, a Kani harness per accessor (TX-11), the structure-aware fuzz grammar, the anchor index, and `hg-fontc`'s writers. `cargo run -p hg-fontgen -- check` is a CI step; a hand edit of any output fails it (G29), proved by the generator's own test planting the edit. |
| 3 | job **`tx-1`**, on the UCD pinned at 18.0.0 |
| 4 | `research_notes/TX-M0 design review/README.md` — built around the two freezes TX-M0 owes, §2.1 (the layout contract, with LY) and §2.2 (the metrics query, with ST) |
| 5 | §3 below |

**What the oracle reported** (13 November 2026, this tree):

```
hg-unicode: IdnaTestV2 6394 rows (2 skipped for an unpaired surrogate),
            19182 checks, 0 differences
TX-1 NormalizationTest: 403420 checks over 20171 lines, 0 failed
TX-1 invariance: 1094910 code points outside @Part1, 0 changed
```

and, for the writers that landed with this plan, 16 tests over seven
described tables: every table written is read back by `hg-fontfmt` and
compared field by field, with the byte expectations for `hhea`, `head`
and `maxp` 0.5 transcribed from the OpenType prose rather than from the
description. **Measured:** swapping `descender` and `line_gap` in
`hhea.hgfont` and regenerating leaves every round-trip test green and
fails the byte test; the same tree without the plant passes 16 of 16.
A round-trip alone would have agreed with itself.

TX-1 names nine conformance files; **two of them run**. The other seven
(BidiTest, BidiCharacterTest, LineBreakTest, GraphemeBreakTest,
WordBreakTest, SentenceBreakTest, emoji-test) are TX-M0's to finish and
are listed in §4.

### 2.2 ST-M0 — HTML and CSS parsing and style

| # | Evidence |
|---|---|
| 1 | `hg-htmlgen`, `hg-html-syntax`, `hg-html5lib`, `hg-cssgen`, `hg-css-props`, `hg-style-reference`, `hg-encoding`, `hg-url`, each with a `tiers.toml` row. `hg-tree-traits`, `hg-encoding-build` and `hg-wpt-url` landed with them and have rows of their own. |
| 2 | **`hg-htmlgen`**: the `.hgtok` description language, and the production tokenizer, the reference stepper and the coverage table it emits into `hg-html-syntax`, on `hg-opgen`'s pattern. Freshness checked in CI (G29). |
| 3 | job **`html5lib`**, with `url`, `cssgen` and `encoding` beside it |
| 4 | `research_notes/ST-M0 design review/README.md` — the six-item agenda, with the 46-row state-source table beside it |
| 5 | §3 below |

**What the oracles reported:**

| Job | Pinned at | Result |
|---|---|---|
| `html5lib` | html5lib-tests `c5495c53` | **14,060 of 14,092 runs, 99.8%**, by both back ends, **0** token mismatches; the named-character-reference table generated from WHATWG's pinned `entities.json`; encoding cases 75 of 82 |
| `url` | WPT `33926aca` | urltestdata **904/904**, toascii **87/87**, percent-encoding **16/16** |
| `cssgen` | `@webref/css` 8.7.9 | 758 properties, 64 legacy aliases, 169 computed-value texts in 12 kinds; **0 disagreements** with the pinned webref, and a planted disagreement in each field in turn is **refused**, against a twin with no plant that passes |
| `encoding` | the Encoding Standard's `encodings.json` and index files | the generated tables fresh (G29), every index entry decoded and re-encoded |

ST-1 — a ship blocker — asks for 100% of the html5lib tokenizer tests
"including error codes and positions". 99.8% is not 100%, and the 32
runs that do not pass are not a mystery: 16 of them are the cases whose
expected `eof-in-processing-instruction` column contradicts the same
file's own correct values for shorter inputs. That is a defect in the
pinned suite, not in the tokenizer, and reporting it upstream is listed
in §4.

### 2.3 LY-M0 — layout

| # | Evidence |
|---|---|
| 1 | `hg-layout-num`, `hg-box-tree`, `hg-layout-reference`, `hg-reftest-raster`, `hg-wpt-layout`, `hg-paint-order`, each with a `tiers.toml` row. `hg-fragment` and `hg-layout` landed with them and have their own rows; `hg-types` gained the geometry its existing row already reserved. |
| 2 | **`hg-layout-num`**: the computed-length numeric type, 24 fractional bits, with `f32` and `f64` confined to its two boundary functions. G39 is enforced by two `hg-graph-lint` rules — a type rule and a literal rule, because the type rule alone misses `let k = 0.5;`. |
| 3 | job **`wpt-layout`**, on WPT `css/CSS2` pinned at `33926aca` |
| 4 | `research_notes/LY-M0 design review/README.md` — §6 divided into what is reviewed, the central item (LY-D1), six items to decide, four cross-track items, and §5's block |
| 5 | §3 below |

**What the oracle reported:**

```
hg-wpt-layout: runnable 0 / 9244 tests, baseline 0
  the first 13 become runnable with an HTML tree builder (ST-M1)
```

over 12,955 files: 13 with machine-readable `data-expected-*` layout
expectations, 6,235 reftests, 66 testharness, 2,930 self-describing and
3,711 support files. **This is a reachability report, not a pass
rate**, and what CI holds against regression is the classification: a
change that stopped recognising reftests would fail the job even though
the runnable count never moved.

The contract crates beside it have their own job, `layout-contract`,
which runs `hg-types`' geometry (Writing Modes 4 §6.4's mapping table),
`hg-layout-num`'s boundary, the fragment store (LY-D18, G45), the
layout contract and its central diff (LY-D1), the box tree (LY-D2), the
reference engine (LY-D5, G41) and Appendix E's paint order.

### 2.4 GR-M0 — graphics and rendering

| # | Evidence |
|---|---|
| 1 | `hg-gpu-hal` (L5, P), `hg-raster` (L2g, P), `hg-raster-ref` (tools, T), `hg-display-list` (L2, P), each with a `tiers.toml` row |
| 2 | **`hg-display-list`**, generated by `hg-ipc-codegen` from one `.hgidl` (GR-D6), with the standard freshness test and its twin: the same comparison against a hand-edited copy must fail |
| 3 | job **`raster-ref`**, on **x86-64 and AArch64**, which is the form GR-M0's row asks for; it needs no pinned data because the reference is exact and computes its own answers |
| 4 | `research_notes/GR-M0 design review/README.md` — the HAL's five properties against three back ends as the central item, three measured findings in its §3, and §5's two blocks |
| 5 | §3 below |

**What the oracle reported**, on the first accuracy cases:

```
hg-raster vs hg-raster-ref: 18 of 720 pixels differ, worst 1 level
axis-aligned shapes:        agree exactly
hg-raster: canonical hash   0xb0047df6484f657c
```

GR-6 asks for ≤ 1/255 per channel on 99.9% of edge pixels and ≤ 2/255
on all. Every differing pixel is at 1/255, so both halves hold — and
they hold against an oracle with **no rounding of its own**, because
`hg-raster-ref` is built on exact `i128` rationals rather than the
float64 §6.26's crate table names. That deviation is recorded in the
review's §3.3 rather than taken silently, together with two other
measurements the work produced:

- **a maximal chunk is 7,516,291,092 B against a 65,536 B message
  cap**, about 114,000×, so GR-D6's "generated, typed IPC format"
  needs a stated transport (review §3.1);
- **`hg-raster` accumulates in `SUB`² units, not §6.4's 1/256 px²**,
  because exact trapezoid areas need the finer denominator — "exact
  area coverage" and "1/256 px² units" cannot both hold (review §3.2).

**Clause 2 is met by the format, not by the HAL**, and the review says
so: GR-M0 is the one M0 whose "Needed by" is another track's milestone
(LY-M0), `hg-paint-order`'s contract is **co-signed at LY-M0's
review**, which needs a named graphics owner who does not exist, and
GR-D28's trusted-prompt layer needs a named security owner. So two of
GR-M0's items wait on D-S8 — and only two, which is what §§1–3 of its
review are for. `hg-gpu-alloc` is **not built**: §6.6 names the
allocator, the review fixes its interface, and a placeholder crate
would have met the letter of clause 1 and told the review nothing.

### 2.5 NW-M0 — networking

| # | Evidence |
|---|---|
| 1 | `hg-net-wire`, `hg-netsim`, `hg-crypto`, `hg-crypto-check`, `hg-tls`, each with a `tiers.toml` row. `hg-net-wire` and `hg-crypto` are L2s, tier P, because renderers link them. |
| 2 | **`hg-net-wire`**: the budget every state machine is charged against, the eleven named limits (NW-15), and four checked kernels — QUIC varints, packet numbers, ACK ranges and flow control — each proved total by Kani for every input (G33). Added to **both** `tools/kani.sh`'s `KANI_PACKAGES` and the `kani:` matrix, which the `kanipkgs` gate compares. |
| 3 | job **`wycheproof`**, on the Wycheproof vectors pinned at `65ca4947`, with `net-contract` beside it |
| 4 | `research_notes/NW-M0 design review/README.md` — thirteen state-machine interfaces, the partition key type as the central item, and §5's block |
| 5 | §3 below |

**What the oracle reported:**

```
HMACSHA256:    174/174 pass, 0 acceptable
HKDF-SHA-256:   86/86  pass, 0 acceptable
```

over 359 files, 113 algorithms and 145,424 cases, with the other 111
algorithms **classified rather than counted as failures** — the same
shape as `wpt-layout`, and for the same reason. `hg-crypto` implements
three primitives (SHA-256, HMAC, HKDF), each written from its
published standard, and `pins.toml` resolves the other ten with the
licence each will ship under.

Beside it, NW-50's timing test, which is a **gate before the code it
gates** (§9.1: secret-dependent code merges only once this passes).
What it asserts is the plant: measured here, a planted
short-circuiting comparison comes out at ratio **1,539.6** and
`ct::eq` at **0.0**, against a threshold of 0.25. A timing test that
could not find a short-circuit would have passed over `hg-crypto` for
the rest of the project.

And `hg-tls`'s transition table: 9 states against 9 messages is 81
pairs, of which **10 are transitions and 71 are alerts**, every one
checked. A state machine written as `if` chains is one where the
alerts are the lines somebody has to remember.

**The Q-CS-1 question the plan raised, answered by measurement.** The
plan required `[q-cs-1] crypto-implementers` and the four crypto rules
in `hg-graph-lint` to be revisited *before* `hg-crypto` landed, on the
reading that it would be a third legitimate implementer. It is not:
that list is about the cryptographic-**randomness** trait, and
`hg-crypto` consumes randomness rather than producing it — it holds no
implementation of `CryptoRandom` and names it nowhere. The list is
unchanged, and the hole `m1-exit` §1.3 measured is not widened.

The network process skeleton exists today (`crates/hg-net-service`),
and its own doc says what it is: "Sockets, DNS and the protocol stack
are NW-M0's own plan."

---

## 3. The interlock, in one place

This is clause 5's artefact. Each row names the counterpart and the
direction; no row is a two-way "coordinate with".

### 3.1 Between the five M0 tracks

| From | To | What | Clause |
|---|---|---|---|
| **TX** | **ST** | normalization and the UTS 46 data | TX-M0: "first, by month 6, for `hg-url` at the style spec's ST-M0" — **discharged in the tree** |
| **TX** | **ST** | the metrics query, as exact ratios | TX §6.8a; "which the style spec's ST-M1 needs" |
| **TX** | **LY** | shaped segments, and the flags `break_reshapes`, `join_reshapes`, `kashida_point` | TX §6.8 |
| **TX** | **LY** | break opportunities with cumulative positions | TX §6.8; LY decides which one fits (LY §6.5.2) |
| **TX** | **LY**, the semantic plane | per-glyph ink bounds; per-character bounds | TX §6.8 |
| **ST** | **TX** | the computed-length type that multiplies the ratio | ST §6.9; binary64, fixed by the SVG spec's R5 |
| **ST** | **LY** | the re-entry protocol's agreement | at ST-M0's design review |
| **ST** | **LY** | the "affects" mask | ST §6.9 |
| **LY** | **TX** | one conversion into `LayoutUnit`, and only one | LY-9, LY-D3 |
| **LY** | **ST** | container re-entry's **scheduling** | "the layout spec owns the scheduling" (ST §6.10) |
| **LY** | **ST** | `hg-layout-reference`, with size containment | ST-M2 tests re-entry against it (LY-M1) |
| **LY** | **GR** | `hg-paint-order`'s **requirements** | LY §6.14; GR §6.24 owns the implementation |
| **LY** | **GR**, DevTools, the GPU service | the fragment store's read API; `hg-types`' geometry | G38 |
| **GR** | **LY** | the paint budget's other half | GR-D23's 160 B†, provisional |
| **GR** | **LY** | `hg-paint-order`'s contract, **co-signed** | GR-M0's row; blocked, §4 |
| **NW** | **ST**, **TX**, **GR** | the partition key type, and its `(T, C, X, n)` projection | NW §6.8; P18 — one type, projected in one place |
| **ST** | **NW** | `hg-url`, the browser's one URL parser | NW §6.2, G31 — built, 904/904 |
| **JS** | **NW** | a jitless engine with no DOM, for PAC | NW §6.13, IPC-R10; JS-M1 |

### 3.2 Out to tracks that are not M0s

| From | To | What | Clause |
|---|---|---|---|
| **TX** | **ED** | composition rendering, segmentation and bidi data | TX §6.8a as amended 20 October 2026 |
| **ED** | **TX** | the broker-side IME relay, caret and selection movement | TX §9.3's counting rule, applied 20 October 2026 |
| **TX** | **SH** | the UI face, per OS | Q-TX-8, answered; the Linux face is TX-W1's |
| **ST** | the provenance reviewer | reviewer independence | ST-M0's review; recorded in its agenda |
| **LY** | the provenance reviewer | a reviewer who has not read the engine source | LY-M0's review §6 |

### 3.3 In from D-S8, which is deferred

| To | What | Where it is recorded |
|---|---|---|
| **TX** | a staffed type-design program **with its leads** | TX review §5 |
| **LY** | a **named graphics owner**, "who must exist before this review" | LY review §5 |
| **GR** | the security owner, and LY-M0 first | GR-M0's row |
| **NW** | the root program's charter and its lead | NW-M0's row |

Four of the five M0 rows name a human role as a precondition. The
definition of "start" was written to turn on artefacts precisely so
that this table could be filled in while they are open — but filling it
in does not close any of them.

---

## 4. What each M0 still owes before *its* exit

Starting is not exiting. Each row below is from the track's own M0 row
or its review agenda, and none is satisfied by anything in §2.

### TX-M0

| Owes | Blocked by |
|---|---|
| The other seven TX-1 conformance files at 100% (BidiTest, BidiCharacterTest, LineBreakTest untailored, GraphemeBreakTest, WordBreakTest, SentenceBreakTest, emoji-test) | nothing — work |
| The shaping specification's engine model, and the default, Arabic-type and USE engines drafted | nothing — work |
| The fuel model; both rasterizer families spiked; TX-D9's oracle harness if allowed | nothing — work; the licence question in TX-D9 is the owner's |
| OS-family and interim manifests; the build-time session-image part | three local surveys (Q-TX-5, Q-TX-12, Q-TX-17), each needing a machine of each kind |
| Kani proofs for `glyf`, `GDEF`, `GSUB`, `GPOS` and the session-image format | nothing — work; `cmap`, `loca` and `hmtx` are described, `glyf` is not |
| The type-design program staffed; the wave plan with its bill of materials; the alias widths measured; the last-resort face generated | **D-S8** |
| The two freezes signed | their counterparts' M0 reviews |
| The month-6 measured-velocity review | **D-S8** — a velocity measure needs a denominator |

### ST-M0

| Owes | Blocked by |
|---|---|
| ST-1 at **100%**, and the 16 contradictory `eof-in-processing-instruction` expectations reported upstream | an upstream report, which is an outward-facing action |
| The tree builder: it is what makes `wpt-layout`'s 13 machine-readable layout tests runnable at all, and a precondition for the 6,301 reftests and testharness tests beyond them | nothing — ST-M1 |
| The six-item agenda decided: style groups, match keys, the invalidation index, the substitution interpreter, container re-entry with LY, the `:has()` engine | the LY counterpart for one of the six |
| The provenance review, by a reviewer who has not read the engine source | a reviewer who is not the author |

### LY-M0

| Owes | Blocked by |
|---|---|
| LY-D1 settled: is the constraint space the complete input? | nothing — the review |
| Six items decided in their own right; four cross-track items agreed | ST and TX counterparts |
| `hg-paint-order`'s contract | **D-S8** — a named graphics owner, "who must exist before this review" |
| Q-LY-1, Q-LY-2, Q-LY-3, Q-LY-4, Q-LY-6, Q-LY-15 | nothing — black-box measurements |

### NW-M0

| Owes | Blocked by |
|---|---|
| Twelve more state-machine interfaces fixed, against `hg-tls`'s table as the worked example | nothing — the review |
| The partition key type, agreed with ST, TX and GR | three counterpart tracks |
| The ACVP runner | the first primitive it covers; its vector sets cover none that exist |
| NW-50's **binary taint check**, per ISA | nothing — work; the timing half is built |
| Q-NW-1, Q-NW-2, Q-NW-9, Q-NW-12, Q-NW-13, and the Linux GRO measurement | three need a machine, not a person |
| The root program's charter and its lead | **D-S8** — but NW-D24's trigger already covers the gap: a release before NW-53 passes ships on the OS trust stores and says so |

### GR-M0

| Owes | Distance |
|---|---|
| `hg-gpu-alloc`'s interface, then the allocator | nothing — the review decides the interface; GR-M1 implements it |
| `compositor_frame.hgidl` and `canvas_command.hgidl` | nothing — work (review §3.4) |
| The transport decision for a chunk too large to be a message | nothing — a decision (review §3.1) |
| §6.4's accumulator unit, and `hg-raster-ref`'s exactness | nothing — two spec amendments or two reverts (review §3.2, §3.3) |
| The canonical image, E16 and integer composition, specified | nothing — work, and the precondition for GR-50 and GR-51 |
| The three back ends themselves | the HAL contract is written; no back end exists |
| GR-S1, GR-S2, GR-S6 (first pass), GR-S9 done; GR-S3, GR-S4 started | four need hardware or a running engine (review §6) |
| `hg-paint-order`'s contract, co-signed | **D-S8** — a named graphics owner, through LY-M0's review |
| GR-D28's trusted-prompt layer | **D-S8** — a named security owner |

---

## 5. What this document does not show

- **It does not show that M1 exits.** Clause 4 of seven now holds; the
  other six were checked against `d11deee` in
  `measurements/milestones/m1-exit/README.md`, and that document's §6.5
  records what still blocks the exit as a whole.
- **It does not show that any M0 exits.** §4 is the distance, and four
  of the five tracks have at least one item in it that no amount of
  work closes.
- **It does not show that the oracles are strict enough.** Three of
  the numbers in §2 are below the ship-blocking threshold the specs
  set: ST-1 wants 100% and has 99.8%; TX-1 wants nine conformance
  files and runs two; `wpt-layout` has no pass rate at all. A fourth,
  Wycheproof, is 100% — of two algorithms out of 113. Each is recorded
  as a number that can only go up, which is what makes the gap visible
  rather than what closes it.
- **It does not replace the M0 waiver.** FI-D1 requires M0's own start
  conditions — D-S8 funding, M1–M2 staffing — to be "formally waived or
  met". The record is drafted as **D-S13** (Stage 1 §13.1b) and is
  open; its recommendation is to waive both **for M1 and M2 only**,
  with the waiver lapsing at M3. M1 depends on M0, so until the owner
  takes it that blocks M1's exit independently of clause 4.
