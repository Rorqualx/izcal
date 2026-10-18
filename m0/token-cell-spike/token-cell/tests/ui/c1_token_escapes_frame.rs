// A callback's token cannot escape its frame (so it cannot outlive the lend).
use token_cell::{ScriptToken, enter, lend, reenter};

fn main() {
    let mut stash: Option<&mut ScriptToken> = None;
    enter(|t| {
        lend(t, || reenter(|inner| stash = Some(inner)));
    });
    let _ = stash;
}
