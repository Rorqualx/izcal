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

Filed as **html5lib/html5lib-tests#202** on 14 November 2026, on the owner's
answer to IZ-5, disclosing AI assistance. It reports defect 1 and the seven
duplicates. It **understates defect 2**: case 500 and the eighth PI case were
measured afterwards, by the lint above. Upstream issue #125 ("Logic of
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

**Either way, two factual corrections are owed to ST-1** and are independent
of the choice: the count is **6,824 at the pin, not 6,827**, and a count is
not a coverage claim while 18 of `test3.test`'s cases are redundant.

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
