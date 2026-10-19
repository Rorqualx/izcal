// Criterion (3): a `WeakMember` nested inside a `TokenCell` field's value.
use token_cell_v8::gc_object;
use token_cell_v8::member_cell::DomNode;
use v8::cppgc::WeakMember;

gc_object! {
    pub struct Bad {
        traced {}
        cells { pub links: Vec<Option<WeakMember<DomNode>>> }
        plain {}
    }
    name = c"Bad";
}

fn main() {}
