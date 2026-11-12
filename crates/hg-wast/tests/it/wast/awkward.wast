;; The lexical cases a span-keeping reader has to survive, written for
;; these tests: comments that nest, that hold quotation marks and
;; unbalanced parentheses, strings that hold parentheses, annotations
;; before a command's keyword, and control characters inside a comment.
(;a (;nested (;deeper;) nested;) a;)
;; a line comment with (; and ;) in it
((@where) module (@is $the) $keyword
  (; a comment with a " and ((( and ))) in it ;)
  (data ")" "(" "\00\ff")
  (func $"a quoted name with ) in it")
)

(assert_malformed (module quote "(@a \00)") "illegal character")

(@an annotation between commands)

(assert_return (invoke "f" (i32.const -1) (f32.const 0x1p-149)) (i64.const 0xffff_ffff_ffff_ffff))
