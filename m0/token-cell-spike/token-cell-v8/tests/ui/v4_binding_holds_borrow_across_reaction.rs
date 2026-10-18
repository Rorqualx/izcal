// A generated-binding body that keeps node state borrowed while it calls a JS
// observer (a custom-element reaction / observer delivery).
use token_cell::ScriptToken;
use token_cell::dom::Document;
use token_cell_v8::{binding, hg_script};
use v8::{FunctionCallbackArguments, PinScope, ReturnValue, Value};

thread_local! { static DOC: Document = Document::new(); }

fn body<'s>(
    tok: &mut ScriptToken,
    scope: &mut PinScope<'s, '_>,
    args: &FunctionCallbackArguments<'s>,
    _rv: &mut ReturnValue<'s, Value>,
) {
    DOC.with(|d| {
        let node = d.node(0, tok);
        let data = node.data.borrow_mut(tok);
        let obs = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
        let recv = v8::undefined(scope).into();
        hg_script::call(tok, scope, obs, recv, &[]);
        data.child_count += 1;
    });
}
binding!(fn cb = body);

fn main() {}
