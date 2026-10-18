// A mutable borrow held across a call that may run script.
use token_cell::{TokenCell, enter, lend};

fn main() {
    let c = TokenCell::new(0u32);
    enter(|t| {
        let r = c.borrow_mut(t);
        lend(t, || ()); // runs script
        *r += 1;
    });
}
