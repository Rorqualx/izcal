// Criterion (1): only the event loop may mint the outermost token.
fn main() {
    let isolate = &mut v8::Isolate::new(Default::default());
    v8::scope!(let hs, isolate);
    let context = v8::Context::new(hs, Default::default());
    let scope = &mut v8::ContextScope::new(hs, context);
    token_cell_v8::hg_script::enter(scope, |_scope, _tok| ());
}
