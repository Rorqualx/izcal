// insertBefore-style code that keeps a node's data borrowed while it runs a
// reaction that re-enters script.
use std::rc::Rc;
use token_cell::dom::{Document, MutationRecord};
use token_cell::{ScriptToken, enter, lend, reenter};

fn main() {
    let doc = Document::new();
    enter(|t| {
        let root = doc.create_node("root", t);
        let x = doc.create_node("x", t);
        doc.observe(
            Rc::new(|d: &Document, r: &MutationRecord, t: &mut ScriptToken| {
                lend(t, || reenter(|t| d.set_text(r.added, "", t)))
            }),
            t,
        );
        let node = doc.node(x, t);
        let data = node.data.borrow_mut(t); // e.g. caching child's state
        doc.insert_before(root, x, None, t).unwrap(); // runs the observer
        data.connected = true;
    });
}
