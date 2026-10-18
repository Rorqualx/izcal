// The "script" closure tries to use the outer token while it is lent.
use token_cell::{TokenCell, enter, lend};

fn main() {
    let c = TokenCell::new(0u32);
    enter(|t| {
        lend(t, || *c.borrow_mut(t) += 1);
    });
}
