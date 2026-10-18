// A TokenCell borrow held across hg_script::call (runs JS).
use token_cell::{ScriptToken, TokenCell};
use token_cell_v8::hg_script;

fn f<'s>(
    tok: &mut ScriptToken,
    scope: &v8::PinScope<'s, '_>,
    js: v8::Local<v8::Function>,
    cell: &TokenCell<u32>,
) {
    let r = cell.borrow_mut(tok);
    let recv = v8::undefined(scope).into();
    hg_script::call(tok, scope, js, recv, &[]);
    *r += 1;
}

fn main() {}
