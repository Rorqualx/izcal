// The token cannot be duplicated.
use token_cell::enter;

fn main() {
    enter(|t| {
        let _second = t.clone();
    });
}
