// A scoped multi-borrow (parent, child, reference) held across script.
use token_cell::{TokenCell, borrow_mut_3, enter, lend};

fn main() {
    let (p, c, r) = (
        TokenCell::new(0u32),
        TokenCell::new(0u32),
        TokenCell::new(0u32),
    );
    enter(|t| {
        let (pp, cc, rr) = borrow_mut_3(&p, &c, &r, t);
        *pp += 1;
        lend(t, || ()); // custom-element reaction runs here
        *cc += 1;
        *rr += 1;
    });
}
