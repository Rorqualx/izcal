// A nested callback cannot mint a second token by construction.
use token_cell::{ScriptToken, TokenCell, lend, reenter};

fn main() {
    let c = TokenCell::new(0u32);
    token_cell::enter(|t| {
        lend(t, || {
            reenter(|_inner| {
                let mut forged = ScriptToken {
                    _not_send_sync: std::marker::PhantomData,
                };
                *c.borrow_mut(&mut forged) += 1;
            })
        });
    });
}
