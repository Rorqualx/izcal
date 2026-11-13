# The state-source table (ST §6.7.1)

Drafted 13 November 2026 for the ST-M0 design review, which accepts or
amends it. §6.7.1 names it as a deliverable in its own right:

> About 35 pseudo-classes depend on state outside the DOM tree: user
> action, location, media, display state, form state, `:defined`,
> `:state()`, `:dir()` and `:lang()`. Each has a named **state
> source**. [...] The table of sources, with each writer and B1 input,
> is an ST-M0 deliverable.

## Why the table exists

A pseudo-class that reads state outside the tree is a hole in
determinism unless three things are written down: **who** writes the
bit, **when** in the frame, and **what recorded input** the write comes
from. Without all three, a replay can produce a different style for the
same recording and nothing says which step diverged.

§6.7.1 fixes the shape of every answer:

> Every source writes through `hg-dom`'s API **on the script thread**,
> either as a posted task or at a named step of "update the rendering",
> and logs a mutation with the old bits; the bit lives in JS-D4's
> element-state column if it is common, otherwise in rare data.

and, for the sources that do not begin on the script thread:

> Sources outside script (media playback, fullscreen and the top layer,
> `:target` after navigation, `:autofill` from the shell) post a task
> whose input is a recorded B1 event.

Two are singled out and are the reason the "when" column is not
decoration:

> **`:hover` and `:active`** are recomputed at a named rendering step
> **after layout**, as a function of layout output and the recorded
> pointer position, so content moving under a still pointer changes
> hover deterministically.

> The heuristic of `:focus-visible` runs in the focus code, so its
> result is an **input**, not a style-time decision.

## How to read the table

- **Bit** — the name in `hg_tree_traits::ElementState`, for the 32 that
  are one bit. A `—` means the state is not a bit: it is read from the
  tree or from rare data, and the matcher asks a different question.
- **Writer** — the component that calls `hg-dom`'s API. Every row's
  write happens on the script thread; the column says what decides the
  value.
- **When** — a posted task, or a named step of "update the rendering".
- **B1 input** — the recorded input the write is a function of. A `—`
  means the write is a pure function of state already recorded (the
  tree, a prior bit), so it needs no input of its own.

---

## User action

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:hover` | `HOVER` | the hit-test over layout output | **a named step after layout**, every rendering update | the recorded pointer position (and the layout it is tested against) |
| `:active` | `ACTIVE` | the same step | **a named step after layout** | pointer button state, with the pointer position |
| `:focus` | `FOCUS` | the focus code | posted task, on a focus change | the input event that moved focus, or a script call |
| `:focus-visible` | `FOCUS_VISIBLE` | the focus code's heuristic | posted task, with `:focus` | **the heuristic's result**, recorded — never recomputed at style time |
| `:focus-within` | `FOCUS_WITHIN` | the focus code | posted task, with `:focus` | — (a function of `:focus` and the tree) |

**Open for the review.** `:hover` and `:active` are written *after*
layout and read by the *next* style pass, so a frame's hover is one
frame behind its own layout. That is what makes it deterministic; the
review records it as intended rather than leaving it to be discovered.

## Location

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:link` | `LINK` | the DOM, on an `href` change | with the mutation | — (a function of the attribute) |
| `:visited` | `VISITED` | the history store | posted task | the visited-set answer **for this container** (ST-D15: keyed by container; never matches in a nonce partition) |
| `:target` | `TARGET` | the navigation code | posted task, after navigation | the recorded navigation |
| `:any-link` | — | — | — | read as `LINK ∪ VISITED` |
| `:local-link`, `:scope` | — | matcher | — | a function of the document URL and the match's scoping root |

**Open for the review.** `:visited` is the one row whose *write* is a
privacy decision as well as a determinism one. ST-D15 keys it by
container; the review checks that the key reaches the bit and not only
the match.

## Display state

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:fullscreen` | `FULLSCREEN` | the fullscreen code | posted task | the recorded fullscreen transition (a B1 event: it begins outside script) |
| `:modal` | `MODAL` | the top-layer code | posted task | the dialog or popover call, or a B1 event |
| `:popover-open` | `POPOVER_OPEN` | the popover code | posted task | as `:modal` |
| `:open` | `OPEN` | the DOM, on `open` | with the mutation | — |
| `:picture-in-picture` | `PICTURE_IN_PICTURE` | the media code | posted task | a B1 event |

## Media

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:playing` | `PLAYING` | the media process, through the renderer | posted task | a recorded playback event |
| `:paused` | — | — | — | read as the absence of `PLAYING` on a media element |
| `:seeking` | `SEEKING` | the media process | posted task | a recorded playback event |
| `:muted` | `MUTED` | the media element | posted task or mutation | a script call, or a recorded user action |
| `:buffering`, `:stalled` | — | the media process | posted task | recorded playback events; **not bits yet** — the review decides whether they earn one or live in rare data |

## Form state

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:enabled` | `ENABLED` | the DOM | with the mutation | — |
| `:disabled` | `DISABLED` | the DOM | with the mutation | — |
| `:checked` | `CHECKED` | the form code | posted task or mutation | a recorded user action, or a script call |
| `:indeterminate` | `INDETERMINATE` | the form code | as `:checked` | — |
| `:default` | `DEFAULT` | the form code | on form association | — |
| `:required` | `REQUIRED` | the DOM | with the mutation | — |
| `:optional` | `OPTIONAL` | the DOM | with the mutation | — |
| `:valid` | `VALID` | constraint validation | at its named step | — (a function of the value and the constraints) |
| `:invalid` | `INVALID` | constraint validation | as `:valid` | — |
| `:in-range` | `IN_RANGE` | constraint validation | as `:valid` | — |
| `:out-of-range` | `OUT_OF_RANGE` | constraint validation | as `:valid` | — |
| `:read-only` | `READ_ONLY` | the DOM | with the mutation | — |
| `:read-write` | `READ_WRITE` | the DOM | with the mutation | — |
| `:placeholder-shown` | `PLACEHOLDER_SHOWN` | the form code | on a value change | — |
| `:autofill` | `AUTOFILL` | **the shell** | posted task | a B1 event from the shell — the one row whose source is outside the renderer entirely |
| `:user-valid`, `:user-invalid` | — | constraint validation | as `:valid`, once the user has interacted | a recorded user interaction; **not bits yet** |

## Custom elements and internationalization

| Pseudo-class | Bit | Writer | When | B1 input |
|---|---|---|---|---|
| `:defined` | `DEFINED` | the custom-element registry | posted task, at the upgrade | — (a function of the registry and the tree) |
| `:state()` | — | `ElementInternals` | with the mutation | — ; the value is a set of names, so it lives in **rare data**, not a bit |
| `:dir()` | — | the bidi resolution | at a named step | — (a function of the tree, the `dir` attribute and the text) |
| `:lang()` | — | the matcher | — | — (read from the nearest `lang`, which is in the tree) |

**Open for the review.** `:dir()` is the only internationalization row
whose input is *text*, so it changes when a text node changes, not only
when an attribute does. The invalidation index's `:dir`/`:lang` feature
must therefore be driven by the text-change log entry as well as the
attribute one; §6.8 lists text changes with "the old emptiness", which
is not enough to tell whether direction changed. **This is a gap the
review must close**, and it is the clearest finding this table
produced.

---

## Counts

| | |
|---|---|
| Rows with a bit in `ElementState` | 32 |
| Rows read from the tree or rare data | 10 |
| Rows the review must decide a home for | 4 (`:buffering`, `:stalled`, `:user-valid`, `:user-invalid`) |
| Rows whose B1 input begins outside the renderer | 6 (`:visited`, `:fullscreen`, `:picture-in-picture`, `:playing`, `:seeking`, `:autofill`) |

The spec says "about 35"; this table has 46 rows because it counts the
read-from-the-tree cases that §6.7.1's sentence does not. The review
decides whether the table's scope is "needs a source" (32 to 36 rows)
or "a matcher asks about it" (46).

## What is implemented today

`hg_tree_traits::ElementState` holds the 32 bits named above, and
`hg-style-reference` reads them and never computes them — which is the
half of §6.7.1 a reference engine can check. The writers do not exist
yet: `hg-dom`, the focus code, constraint validation and the shell are
all later tracks. So this table is a **contract to be met**, and the
only part of it testable at ST-M0 is that the matcher is a pure
function of the bits.
