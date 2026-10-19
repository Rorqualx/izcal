// Criterion (3): a traced `Member` inside a GC object's `TokenCell` field.
use token_cell_v8::gc_object;
use token_cell_v8::member_cell::DomNode;
use v8::cppgc::Member;

gc_object! {
    pub struct Bad {
        traced {}
        cells { pub link: Member<DomNode> }
        plain {}
    }
    name = c"Bad";
}

fn main() {}
