// The token cannot move to another thread (tokens are per thread).
use token_cell::enter;

fn main() {
    enter(|t| {
        std::thread::scope(|s| {
            s.spawn(|| {
                let _t = &*t;
            });
        });
    });
}
