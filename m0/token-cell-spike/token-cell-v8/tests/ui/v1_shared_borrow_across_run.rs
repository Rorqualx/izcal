// A &str into DOM text held across hg_script::run.
use token_cell::{ScriptToken, TokenCell};
use token_cell_v8::hg_script;

fn f(tok: &mut ScriptToken, scope: &v8::PinScope<'_, '_>, text: &TokenCell<String>) -> usize {
    let s: &str = text.borrow(tok);
    hg_script::run(tok, scope, "document.body.textContent = ''");
    s.len()
}

fn main() {}
