// A shared borrow held across a call that may run script (script could mutate).
use token_cell::{TokenCell, enter, lend};

fn main() {
    let c = TokenCell::new(String::new());
    enter(|t| {
        let s: &String = c.borrow(t);
        lend(t, || ());
        println!("{s}");
    });
}
