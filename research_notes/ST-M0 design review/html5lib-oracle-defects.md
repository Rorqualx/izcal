# ST-1's oracle has defects of its own, measured

*15 November 2026. Measured against the pinned suite, commit
`c5495c539ca8a79c3c5107f5f14010bcff9a35d5` (`tools/test-data.toml`,
`[html5lib]`). Opens **ST-D21**. Nothing here changes a target; the
decision is the owner's.*

## Why this note exists

**ST-1 is a ship blocker** and asks for "**100%** of the 6,827 html5lib
tokenizer tests, by both the production tokenizer and the reference stepper,
including **error codes and positions**". `hg-html5lib` already records that
it cannot reach 100% and says why in its own module docs: 16 cases expect
`eof-in-processing-instruction` at column 4 whatever follows the target, and
Izcal reports the end of the file where the end of the file is.

So a **ship blocker's pass condition is unreachable by a correct tokenizer**,
and the measurement that proves it sits in a crate while the target still
says 100%. That gap is the thing to decide. The runner's choice — report it
rather than work around it — was right; what is missing is the target's own
answer to it.

This note is the measurement, so the decision rests on numbers rather than
on the paraphrase of a paraphrase.

## What the pinned suite actually holds

| Measured | Value |
|---|---|
| Files in `tokenizer/` | 14 |
| Cases (`tests` + `xmlViolationTests`) | **6,824** |
| Cases `hg-html5lib` judges | 6,820 (`RUNS` = 14,092 over two back ends) |
| Recorded passes | 14,060 of 14,092 runs, **99.8%**, 0 token mismatches |
| Cases in `test3.test` | 1,606 |

**ST-1's "6,827" is stale.** That was the count at `c777c40` (1 October
2026), which the phase-8 fact-check verified. The pin moved to `c5495c53`
("Fix incorrect error order and remove obsolete tests", 5 November 2026),
which holds 6,824. `tools/test-data.toml` records the discrepancy and the
runner reports what it finds; **only the target still names the old number.**

## Defect 1: 16 expectations contradict the file's own convention

`test3.test` carries 25 `eof-in-processing-instruction` expectations. Nine
agree with the rest of the suite's convention — the column is the end of the
input, `len + 1` — and sixteen are fixed at 4 whatever the input's length:

| Cases | Inputs | Expected column | End of input |
|---|---|---|---|
| 1158 | `<?` | 3 | 3 ✓ |
| 1159–1166 | `<?a` … `<?Z` | 4 | 4 ✓ |
| 1167–1173, 1175–1182 | `<?a a` … `<?Z Z` | **4** | 6 ✗ |
| 1174 | `<?Z Z?` | **4** | 7 ✗ |

The nine correct ones are the proof: the convention is the file's own, and in
the longer cases the column simply never advanced past the target. A correct
tokenizer therefore fails **16 cases, 32 runs** — which is exactly the gap
between the recorded 14,060 and 14,092. **14,060 is the ceiling**, not a
shortfall.

## Defect 2: nine inputs lost a character their descriptions still promise

In `test3.test` the description *is* the input, escaping non-printables as
`\uXXXX`. A twelve-line lint — unescape the description, join surrogate
pairs, compare to the input — finds **12 cases where it does not hold**, of
which **3 are a second legitimate convention** (`<!----!LF>`, `<!----!CR>`,
`<!----!CRLF>` at 193–195 spell control characters by name) and **9 are
defects**:

| Case | Description promises | Input holds |
|---|---|---|
| 500 | `<!DOCTYPE a SYSTEM x\u0000` | `<!DOCTYPE a SYSTEM \u0000` — no `x` |
| 1175–1182 | `<?a a?` … `<?Z Z?` | `<?a a` … `<?Z Z` — no `?` |

Two consequences:

- **Seven of the eight PI cases are now byte-identical duplicates** of cases
  1167–1173, in input, output and errors. `test3.test` has 17 duplicate
  groups and 18 redundant cases in all; seven of those groups exist only
  because of this defect. The other nine are benign permutation repeats
  (`<a>` at 1214 and 1556, and so on). **The suite's case count is not a
  coverage measure.**
- **Seven promised inputs are tested nowhere**: `<?a a?`, `<?b b?`,
  `<?y y?`, `<?z z?`, `<?A A?`, `<?B B?`, `<?Y Y?`. The eighth, `<?Z Z?`,
  *does* exist — as case 1174's input — which is the evidence that the `?`
  was intended in all eight.
- **The class predates the processing-instruction tests.** Case 500 is the
  same defect — a description promising a character the input lacks — and it
  is nowhere near the 1 October 2026 PI additions. A one-off would not
  recur; this is a suite without a description-denotes-input check.

## Reported upstream

Filed as **html5lib/html5lib-tests#202** on 14 November 2026, disclosing AI
assistance, on the owner's answer of that date — "yes, file it" — relayed
through the decision queue. *Cited by its content and date rather than by the
queue's id, because that queue renumbered one of its ids the next day: an
identifier that still resolves while pointing at something it no longer means
is this note's own subject, in its crudest form.* The issue reports defect 1
and the seven duplicates, and **understates defect 2**: case 500 and the
eighth PI case were measured afterwards, by the lint above, and posted as a
correction on 15 November 2026. Upstream issue #125 ("Logic of
eof-in-comment error positions", open since 2019, no comments) reports the
same class for comments, so the convention has been ambiguous upstream for
seven years and a fix is not imminent.

## ST-D21, for the owner: a ship blocker whose oracle is defective

**(a) Recommended — 100% of the pinned cases *except* a recorded exception
list.** Each entry carries the case index, the contradiction, the upstream
issue and the date it was recorded. The property that makes it honest is the
one the milestone gate already uses for `[[refuted]]`: **an entry that the
pinned data no longer contradicts is a finding**, not a silent pass. So the
list cannot outlive the defect, and a re-pin that fixes the 16 cases forces
them back under the 100%. ST-1 stays a ship blocker and stays reachable.

**(b) 100% literally — Izcal matches the suite, wrong columns included.**
Refused: it makes a wrong error position a shipping requirement, and the same
tolerance would hide a real position bug behind an expectation nobody
re-derives.

**(c) ST-1 held until upstream fixes it.** A ship blocker gated on a third
party's tracker, where the sibling issue has been open since 2019.

**(d) Correct the oracle.** Not among the options first put to the owner, and
the one they wrote: a committed table of patches, applied as each case is
read.

**Either way, two factual corrections are owed to ST-1** and are independent
of the choice: the count is **6,824 at the pin, not 6,827**, and a count is
not a coverage claim while 18 of `test3.test`'s cases are redundant (10 with
(d) applied).

## Decided: (d), and what it measured

The owner's answer of 15 November 2026, relayed through the decision queue,
was ***"fork and add patches"***. The patches are not a pushed fork, because
a second remote is the owner's own to ask for; they are
`crates/hg-html5lib/src/corrections.rs`, a table applied at load. The pin
does not move and the fetched tree stays byte-identical to upstream, so there
is still one digest in `tools/test-data.toml`.

**28 corrections**, each naming the exact value it replaces and the class of
evidence it rests on:

| Class | Entries | What it rests on |
|---|---|---|
| lint-proven | 9 | The file contradicts itself: `description == input` for 1,247 of `test3.test`'s 1,606 cases, and these nine break it. |
| spec-and-suite | 17 | 13.2.5.75 and 13.2.5.76 emit only an end-of-file token at EOF, so nothing further is consumed. The *number* is not the standard’s: it does not specify error columns at all, so it comes from the nine uncontradicted cases 1158–1166. |
| series-inferred | 2 | Case 1174's stray `?`. An argument about intent, gated so it can be dropped alone. |

**The weak hunk got stronger while being written down.** Case 1174 is series 1's
only member carrying a trailing `?` and case 1182 is series 2's only member
lacking one, so applying both hunks **moves one character between two rows** —
a transposition, which also removes the file's last duplicate pair rather than
creating one. That is a far better account of the evidence than eight
characters lost in the same place. It is still the weakest of the three
classes, because nothing in the file contradicts itself, so it stays gated and
labelled.

**The result: 14,092 of 14,092 runs, 100.0%.** The tokenizer did not change,
so the entire 32-run gap was the oracle's and none of it Izcal's.

### What the 100% is evidence of, and what it is not

I first wrote that the corrections and the tokenizer are two independent
derivations, so landing exactly on 14,092 is evidence the table is right. That
is half true and the false half is the half that matters, and a peer session
caught it by holding me to a residual I had named myself before any of this
was built: *a patch could be wrong in the same direction as Izcal and look
green.*

**Strong evidence against a patch set wrong at random.** The table changes 9
inputs and 17 columns; it leaves **30 expectation fields** on those nine cases
untouched, and every one still had to be satisfied. Case 500 is the sharp one:
its first error, `missing-quote-before-doctype-system-identifier` at **column
20**, is an expectation the table does not touch, and after restoring the `x`
it describes a *different character* at that column. It stays satisfiable only
if exactly one character was restored at exactly that point. Insert the `x`
elsewhere, or insert two, and that untouched expectation breaks. The fix is
pinned, not merely permitted.

**Not independent of the tokenizer, and the correlation is total.** The
standard **does not specify error columns at all** — this crate's own docs say
so, and say the convention was *calibrated against the suite*: a state that
consumed a character reports that character's position, and nothing moves the
position on at EOF. The sixteen column corrections rest on the same nine
uncontradicted cases that convention was calibrated from, so correcting
1167–1182 to the end of the input and then observing that the tokenizer agrees
is **one calibration applied twice**. If the convention is wrong, the sixteen
entries are wrong with it and the suite still reports 100%.

**But "one calibration applied twice" was my own overcorrection, and the
number that bounds it is 2,560.** The convention is not calibrated from nine
cases. The suite holds **2,577 error expectations this crate judges**, across
twelve files, and the table touches **17** of them. The other **2,560 are
untouched third-party expectations that all pass** — every error position in
`entities.test`, `numericEntities.test`, `test1`, `test2`, `test4` and the
2,127 in `test3.test` the table never goes near. So the rule that forces the
sixteen values is not a free choice confirmed by itself; it is a rule with
2,560 independent confirmations in the same corpus, extrapolated to sixteen
cases where the corpus contradicts itself. That is ordinary induction, and it
is strong. What it is *not* is a second opinion: induction on a convention
cannot tell you the convention is right, only that it is the one the suite
uses.

**And on a convention there may be nothing to be right about — which is the
real residual, and it is dated.** A peer session's framing: where the standard
fixes no value, "wrong" collapses into "inconsistent", and consistency is
exactly what 2,560 passing expectations evidence. That holds only while
nothing outside this crate and its tokenizer consumes a column *as a value*.
Checked, 15 November 2026: **no other Izcal spec mentions HTML parse errors at
all** — the DevTools spec has zero mentions, and the style spec's two are
ST-1's own "error codes and positions". So the condition holds today. **The
first external consumer of an error position — a DevTools issue, a diagnostic,
anything compared against another implementation — gives the convention an
external truth it currently lacks and turns these sixteen entries from a
convention into a claim.** That is the thing to re-check, and it expires the
same mechanical way the rest of this design does: it becomes false the first
time a column leaves the crate. Recorded as ST-D21's reversal trigger.

**It also tells us what kind of claim #202 is.** The upstream issue tells a
third party their columns are wrong, which would be unfounded if columns were
pure preference. It is not, because the argument filed was **self-inconsistency
inside one file**: `test3.test` puts the error at the end of the input for
`<?` and `<?a` and at column 4 for everything longer. A convention owes
self-consistency, so inconsistency is precisely the defect one may report
against it — and the only one.

Rule 1 — cite the standard's step, never the tokenizer's output — holds the
correlated risk down and does not close it. The evidence class is named
`SpecAndSuite` and not `SpecDerived`, because the label is what a reader in
six months uses to decide how much to trust a hunk, and `SpecDerived` would
have claimed the standard fixes a number it never mentions.

**The two halves are evidenced differently, and both are evidenced.** The nine
input corrections are pinned by 30 expectations the table never touched, one
of which fixes an insertion point to a single position. The sixteen column
corrections are forced by a rule 2,560 untouched expectations confirm. Neither
is unevidenced; they are strong in different ways, and only the second rests
on something that could turn out to have no fact of the matter.

**This is tonight's own symmetry landing on its author.** A measurement became
a target, the target was hit, and the hit was being read as proof. The same
sentence I insisted on everywhere else — *unexercised is not harmless*,
*14,060 is a ceiling, not a shortfall* — applies to the number I was pleased
with.

**The expiry is zero-fuzz, and that is the point.** Every entry must find the
value it names, or the load fails with a message naming the case. A patch set
whose hunks silently stop applying is how a fork rots: upstream fixes the
case, the patch does nothing, and the tree goes on claiming to correct
something. Here the next re-pin reports it. Options (a) and (d) share that
property — it is the half of the recommendation that survived the owner
choosing differently.

**Two counts corrected while implementing**, both mine and both found by
measuring rather than by re-reading: the lost-character defect makes **eight**
cases byte-identical duplicates, not seven (case 500 duplicates 499 and was
left out of the first count), and `test3.test`'s redundant cases fall from 18
to **10**.

## The general shape, because it is not about html5lib

The gate's `[[refuted]]` state answers a *refusal* that has quietly become
true, because the gate re-runs the clause anyway. This is the harder
neighbour: **a recorded number whose evidence still re-derives exactly, while
what it means has changed underneath.** `14,060 of 14,092` re-derives
byte-for-byte on every run and will keep doing so after upstream fixes the 16
cases — at which point the same number means "32 runs are now failing for a
real reason". The ratchet would not notice, because the number did not move.

What stops it here is that the exception list names *which* cases and
re-checks the contradiction, not just the total. The rule that generalises:
**re-derive the question, not only the number.** Three rungs, in order of
strength — an exit status; a recorded number (milestone clause 3); a number
whose inputs and whose *claim* are both re-derived. Only the third catches
meaning drift, and only where the claim is written down precisely enough to
re-check.

## Reproducing this

The lint and the counts are a short script over the pinned tree; nothing is
committed, as `tools/test-data.toml` requires. With the suite fetched by
`tools/fetch-test-data.sh`:

```text
for each file in tokenizer/*.test:   count tests + xmlViolationTests   -> 6,824
in test3.test:
  errors with code eof-in-processing-instruction                       -> 25
    column == len(input) + 1                                           -> 9
    column == 4 with len(input) > 3                                    -> 16
  unescape(description) != input                                       -> 12
    minus the LF/CR/CRLF spellings at 193-195                          -> 9
  groups identical in (input, output, errors, initialStates)           -> 17 groups,
                                                                          18 redundant
```
