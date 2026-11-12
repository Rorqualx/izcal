# JS-M0 `hg-wast`: the WebAssembly script grammar against the whole suite

Measured 11 November 2026, on macOS (Huey the Sixth). This is the JS-M0 plan's Phase 1 (ORC-1): `hg-wast parse` (`crates/hg-wast`).

## What is being claimed, and what is not

JS-M0's exit asks for `hg-wast` running. Izcal has no WebAssembly engine yet, so the crate is built in the order that can be checked: first the part that needs no engine — reading a `.wast` script — and then the part that needs one.

This note measures the first part only. **No script is run here.** The claim is narrow and testable: every script in the pinned suite is read into its commands, and every module those commands name is located exactly.

The second part — an engine adapter, and the differential against the reference interpreter that would validate the runner the way engine262 validates `hg-test262` — is **not done**. It needs an OCaml toolchain (the interpreter's README: OCaml ≥ 4.12, dune and Menhir, then `make`), which is a toolchain decision for the owner, not a thing to install unasked. Until then this crate is a reader with a census, not a conformance result, and nothing here says an engine passes anything.

## Method

- **The data.** `WebAssembly/spec` at the pin in `tools/test-data.toml` (`943c5df`, 10 Nov 2026), fetched by `tools/fetch-test-data.sh wasm-spec` with its SHA-256 checked. Its `test/` and `interpreter/` directories are Apache-2.0; test-only, never linked, never committed, never shipped.
- **The grammar.** Read from the specification's prose under `document/core/text/` and the reference interpreter's README, never from the interpreter's code (M-1 part 3). Three readings decided the lexer:
  - `token ::= keyword | uN | sN | fN | string | id | '(' | ')' | reserved` and `reserved ::= (idchar | string | ',' | ';' | '[' | ']' | '{' | '}')+`, under a longest-match rule, so `0$x` and `"a""b"` are each **one** token;
  - "In a source file, the toplevel `( module … )` surrounding the module body may be omitted" — a whole file may be one module, which `inline-module.wast` is;
  - an annotation `(@…)` may stand anywhere a token may, which `annotations.wast` exercises by putting one **before** a command's own keyword: `((@a) module (@a) $m …)`.
- **The design under test.** A module's body is never parsed. It is kept as the byte range it occupies and handed to an engine verbatim. That is what lets a script reader exist before an engine does — and it moves the whole weight of correctness onto the span's two ends.
- **The run.** `hg-wast parse --suite <tree>/test/core --quiet --verify`, with a `--floor` for each command keyword.

## Results

```
files 259, parsed 259, failed 0
commands 65353
module spans re-parsed 7200
  assert_return    52748     assert_malformed  1945     assert_exception    18
  assert_trap       4984     invoke             359     assert_exhaustion   15
  assert_invalid    2732     assert_unlinkable  200     module instance      3
  module            2269     register            80
```

0.7 s for the whole suite. 64 unit and integration tests pass alongside.

- **Every file parses**, including the three the grammar is hardest in: `comments.wast` (a NUL byte and every other control character inside a block comment, comments nested three deep, quotation marks and unbalanced parentheses inside comments, `(;` and `;)` inside line comments), `annotations.wast` and `inline-module.wast`.
- **Every module span re-parses to the same module.** All 7,200 of them: 2,269 top-level modules, 1,945 under `assert_malformed`, 2,732 under `assert_invalid`, 200 under `assert_unlinkable` and 54 under `assert_trap`. The span is what an engine would be handed, so this is the claim the design rests on, and parsing each one again on its own is what makes its boundary testable at all.
- **The census is independent.** The counts above were cross-checked against a `grep` over the same tree, counting command keywords at the start of a line: 4,984 `assert_trap`, 2,732 `assert_invalid`, 359 `invoke`, 80 `register`, 18 `assert_exception`, 15 `assert_exhaustion` — equal in every case, with the parser finding more only where a command does not begin its line.

## Sensitivity

A reader that quietly stopped reading would still report success, so each mechanism was measured with an error planted in it.

| Planted error | What the run reports |
| --- | --- |
| Block comments stop nesting (`(;` inside one no longer counts) | 1 file fails: `comments.wast`. |
| Strings are not lexed (a quotation mark is an ordinary character) | 253 of 259 files fail; the command count falls from 65,353 to 20. |
| Every module span ends one byte early | 7,199 of 7,200 spans are reported; 258 of 259 files fail. |
| A keyword's floor is set above its count | `FLOOR register: 80, at least 99 expected`, exit 1. |

The first line is the one worth keeping: **nesting is checked by a single file in the whole suite.** The corpus is wide but not uniformly deep, so the crate's own tests carry that case too rather than relying on the suite to keep carrying it.

## Notes

- The floors in the `wast` CI job are the counts of this pin. A pin moves in its own commit, which is where they move with it.
- Telling a reserved run of identifier characters (`0$x`) from a keyword or a number needs the number grammar, which is applied where a number is actually wanted, not in the lexer. A run that is neither is refused there or by the command parser. Inside a module body nothing is checked at all, by design: the engine is the judge of a module.
- A file that does not open with a command keyword is an inline module whatever follows it, so a misspelt command at the very top of a file reads as a module rather than as an error. This is the same design decision, seen from the other side, and it costs nothing an engine would not catch.
