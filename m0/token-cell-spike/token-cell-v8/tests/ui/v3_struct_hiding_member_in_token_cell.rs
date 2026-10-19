// Criterion (3): a `Member` hidden in a struct used as a `TokenCell` value.
// The struct is not `NoTraced`, and `no_traced!` refuses to make it so.
use token_cell::no_traced;
use token_cell_v8::gc_object;
use token_cell_v8::member_cell::DomNode;
use v8::cppgc::Member;

pub struct Plain {
    pub n: u32,
    pub link: Member<DomNode>,
}

no_traced! {
    pub struct Claimed {
        pub n: u32,
        pub link: Member<DomNode>,
    }
}

gc_object! {
    pub struct Bad {
        traced {}
        cells { pub data: Plain }
        plain {}
    }
    name = c"Bad";
}

fn main() {}
