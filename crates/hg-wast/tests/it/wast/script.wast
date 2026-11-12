;; A script in the shape the suite uses, written for these tests.
;; It exercises every command the reader recognises.
(module $first
  (func (export "answer") (result i32) (i32.const 42))
  (global (export "g") i32 (i32.const 7))
)

(register "first" $first)

(invoke $first "answer")
(get $first "g")

(assert_return (invoke "answer") (i32.const 42))
(assert_return (invoke "answer"))
(assert_return (invoke "f") (f64.const nan:canonical) (ref.extern 1))
(assert_return (invoke "f") (either (i32.const 1) (i32.const 2)))
(assert_return (invoke "f") (v128.const i32x4 1 2 3 4))

(assert_trap (invoke "boom") "unreachable")
(assert_trap (module (func (export "_start") (unreachable))) "unreachable")
(assert_exhaustion (invoke "deep") "call stack exhausted")
(assert_exception (invoke "throws"))

(assert_malformed (module quote "(module (func) ") "unexpected end")
(assert_invalid (module (func (result i32))) "type mismatch")
(assert_unlinkable (module (import "no" "such" (func))) "unknown import")

(module definition $d (func))
(module instance $i $d)

(module binary "\00asm" "\01\00\00\00")
