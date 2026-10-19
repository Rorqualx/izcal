// Criterion (3): a `TracedReference` in a plain (untraced) field.
use token_cell_v8::gc_object;

gc_object! {
    pub struct Bad {
        traced {}
        cells {}
        plain { pub wrapper: v8::TracedReference<v8::Object> }
    }
    name = c"Bad";
}

fn main() {}
