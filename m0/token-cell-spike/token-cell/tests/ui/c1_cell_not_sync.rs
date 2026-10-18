// A TokenCell cannot be reached from another thread (that thread has its own token).
use token_cell::{TokenCell, enter};

fn main() {
    let c = TokenCell::new(0u32);
    std::thread::scope(|s| {
        s.spawn(|| enter(|t| *c.borrow_mut(t) += 1));
        enter(|t| *c.borrow_mut(t) += 1);
    });
}
