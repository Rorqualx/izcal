# JS-M0 `hg-wast`: the WebAssembly suite read, and read correctly

Measured 11 November 2026, on macOS (Huey the Sixth). This is the JS-M0 plan's Phase 1 (ORC-1): `hg-wast` (`crates/hg-wast`).

## What is being claimed, and what is not

JS-M0's exit asks for `hg-wast` running. Izcal has no WebAssembly engine yet, so the crate is built in the order that can be checked, and this note measures two claims of different strength.

1. **Every script in the pinned suite is read into its commands**, and every module those commands name is located exactly (`hg-wast parse`). This needs no engine.
2. **The reading is correct**, in the only sense that can be tested without an engine of our own: a script written back out from what was parsed is still the same script to the WebAssembly reference interpreter, and stops being one the moment a single expected value is changed (`hg-wast check --mutate`).

What is **not** claimed: that any engine conforms to anything. Izcal has no WebAssembly engine, and the interpreter here is an oracle, not a subject. When Izcal has one it implements the same `ScriptEngine` trait and is judged by the same runs.

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

## The differential

`hg-wast check --suite … --engine … --mutate`, against the reference interpreter built from the same pinned tree (`make -C interpreter`, black-box: never linked, never vendored, its source never read).

```
engine wasm wasm 3.0.0 reference interpreter
files 259, agreed 259, disagreed 0, engine refuses the original 0
mutations caught 211, uncaught 0, nothing to change 48
```

17 s for the suite.

- **Agreement is not the result.** The interpreter passes all 259 original files, so a runner that dropped every assertion would agree too. The result is the second line: in 211 of the files one expected value was changed and the interpreter then **refused** the rendering, every time. The remaining 48 expect no exact number anywhere — they are files of `assert_invalid` and `assert_malformed` — and are reported as such rather than counted as passes.
- **Why floats are rendered in hexadecimal.** A hexadecimal literal names a binary value exactly, so nothing rounds on the way out and back. The decimal alternative was measured, not assumed: rendering through the host's shortest round-trip formatter, with NaN written as bare `nan`, still agrees on **251 of 259** files. The 8 it breaks are `address.wast`, `conversions.wast`, `float_memory.wast`, `local_tee.wast`, `memory64/address64.wast`, `memory64/float_memory64.wast`, `multi-memory/float_memory0.wast` and `simd/simd_splat.wast` — every one a NaN payload collapsed onto the canonical `0x400000`, losing `0x500001`, `0x7fffff`, `0x200000`, `0xf1e2`. A 97%-correct rendering is exactly the kind that ships.
- **A hang found in the oracle.** `wasm -v` prints its banner and then falls into the interactive loop, because the documented rule is that a run with nothing left to do lands in the REPL. It exited here only because standard input happened to be at EOF; on a terminal it waits for ever. Standard input is now closed for every engine call. A tool that can hang is one that will, on some machine, at some hour.
- **The toolchain.** The interpreter's own files declare `ocaml >= 4.14`, `dune >= 2.9`, `menhir >= 20220210` (its README says 4.12, which disagrees with its `dune-project`). It builds clean on OCaml 5.5.2 here; CI uses noble's 4.14.1, 3.14.0 and 20231231.

## Sensitivity

A reader that quietly stopped reading would still report success, so each mechanism was measured with an error planted in it.

| Planted error | What the run reports |
| --- | --- |
| Block comments stop nesting (`(;` inside one no longer counts) | 1 file fails: `comments.wast`. |
| Strings are not lexed (a quotation mark is an ordinary character) | 253 of 259 files fail; the command count falls from 65,353 to 20. |
| Every module span ends one byte early | 7,199 of 7,200 spans are reported; 258 of 259 files fail. |
| A keyword's floor is set above its count | `FLOOR register: 80, at least 99 expected`, exit 1. |
| Floats rendered as shortest decimal, NaN as bare `nan` | 8 of 259 files disagree with the interpreter. |
| The last mantissa hexadecimal digit dropped when rendering | 28 of 259 files disagree with the interpreter. |

The first line is the one worth keeping: **nesting is checked by a single file in the whole suite.** The corpus is wide but not uniformly deep, so the crate's own tests carry that case too rather than relying on the suite to keep carrying it.

## Notes

- The floors in the `wast` CI job are the counts of this pin. A pin moves in its own commit, which is where they move with it.
- Telling a reserved run of identifier characters (`0$x`) from a keyword or a number needs the number grammar, which is applied where a number is actually wanted, not in the lexer. A run that is neither is refused there or by the command parser. Inside a module body nothing is checked at all, by design: the engine is the judge of a module.
- A file that does not open with a command keyword is an inline module whatever follows it, so a misspelt command at the very top of a file reads as a module rather than as an error. This is the same design decision, seen from the other side, and it costs nothing an engine would not catch.
