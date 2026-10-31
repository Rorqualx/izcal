# JS-M0 operation-level oracle: the core set against engine262

Measured 31 October 2026, on macOS (Huey the Sixth). This is the JS-M0 plan's Phase 1 operation-level oracle (ORC-1): `hg-js-oracle ops` (`crates/hg-js-oracle`).

## Method

- **The cases.** 28 core operations that compute on values:
  - 10 unary: `ToBoolean`, `ToNumber`, `ToNumeric`, `ToString`, `Not`, `TypeOf`, `Neg`, `BitNot`, `Inc`, `Dec`;
  - 18 binary: arithmetic, bitwise, shifts, `==`, `===` and the four relational operators.

  Each runs on every operand, or ordered pair, from 74 edge values, for 99,308 cases. The values:
  - `undefined`, `null`, the Booleans, and a fresh `Object.create(null)`, on which ToPrimitive throws;
  - 34 Numbers: ±0, ±2^30, ±2^31, 2^32, 2^53, NaN, the infinities, subnormals, the largest double, and the 1e21 and 1e-7 switches of Number::toString;
  - 35 Strings for StringToNumber's grammar, plus code units outside the BMP and a lone surrogate.
- **Izcal's side.** Each case is assembled into a unit and run as a job by the generated reference interpreter. It runs once on an ordinary heap and once under the moving test collector, and the two answers must agree.
- **engine262's side.** The same expression is evaluated by engine262 at the pin in `tools/test-data.toml` (`d553e7b`, npm package), run black-box under Node 26.13.0 through its documented CLI. It is never linked, and its source is never read.
- **Comparison.** Both sides print one encoding: the type; a Number's 64 bits (NaN as one value); a String's code units; or a thrown error's constructor name.
- **Two rules on top of exact equality:**
  - **Engine262 crashes.** A case engine262 crashes on is decided by a second engine, Node's own, run black-box the same way, and listed.
  - **`Exp` within one ulp.** ECMA-262 leaves Number::exponentiate's general case implementation-approximated ([step 13](https://tc39.es/ecma262/#sec-numeric-types-number-exponentiate)). Izcal's interim `pow` promises at most 1 ulp, and correct rounding belongs to `hg-math` (JS spec 6.9). So an `Exp` answer one ulp from engine262's is accepted and listed. Every other operation must agree exactly.

## Results

`hg-js-oracle ops: 99308 cases, 28 operations, 0 mismatched, 3 within 1 ulp, 4 decided by Node (engine262 crashed)` (exit 0, 8 s with 6 jobs).

- **Within 1 ulp:** `Number.MAX_VALUE ** 0.5`, with `0.5`, `".5"` and `"+.5"` as the exponent. Izcal gives 2^512 (`0x5ff0000000000000`) and engine262 gives `0x5fefffffffffffff`, the correctly rounded square root. The exact value lies about 2^-110 (relative) from the midpoint between the two doubles. That is closer than double-double resolves, which is the case `pow`'s documentation names.
- **engine262 crashed:** `true == o`, `false == o`, `o == true` and `o == false` with `o = Object.create(null)`. Izcal and Node both throw a TypeError. engine262 stops on an internal assertion.

  The cause is in ECMA-262 itself, unchanged in [ES2025](https://tc39.es/ecma262/2025/#sec-islooselyequal), [ES2026](https://tc39.es/ecma262/2026/#sec-islooselyequal) and the [current draft](https://tc39.es/ecma262/#sec-islooselyequal). The Boolean steps read "If x is a Boolean, return **!** IsLooselyEqual(! ToNumber(x), y)" (and the same for y), but the inner call can throw: for an Object `y` it reaches "return ! IsLooselyEqual(x, **?** ToPrimitive(y))". The `!` asserts something false, and engine262 implements the assertion. The thrown TypeError is the intended answer.

  No tc39/ecma262 issue for it was found on 31 Oct 2026. The open editorial PR [#3918](https://github.com/tc39/ecma262/pull/3918) refactors these steps; it is not checked here whether it fixes them.
- **Sensitivity.** With one planted error in the oracle's table (`Sar` computed as `>>>`), the run reports 657 `Sar` mismatches and exits 1.

## Open

- **CI.** The CI job needs Node and the pinned engine262 package from `tools/fetch-test-data.sh`, which lands with `hg-test262`. The job comes with or after that PR.
- **Coverage.** Loads, jumps, `Return`, property get and set, and the intrinsics are not in the oracle; the interpreter's own tests and the probe cover them. Objects with `valueOf` or `toString` are not in it either: the core set has no functions defined in script, so ToPrimitive reaching user code is checked through host functions in `hg-js-runtime`'s tests.
