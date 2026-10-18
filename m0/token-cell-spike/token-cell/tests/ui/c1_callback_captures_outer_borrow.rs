// The "script" closure tries to use a borrow made under the outer token.
use token_cell::{TokenCell, enter, lend};

fn main() {
    let c = TokenCell::new(0u32);
    enter(|t| {
        let r = c.borrow_mut(t);
        lend(t, || *r += 1);
    });
}
