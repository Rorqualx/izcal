// Holding a &str into node text across a reaction; the observer could clear it.
use token_cell::dom::Document;
use token_cell::enter;

fn main() {
    let doc = Document::new();
    enter(|t| {
        let root = doc.create_node("root", t);
        let x = doc.create_node("x", t);
        let node = doc.node(x, t);
        let text: &str = &node.data.borrow(t).text;
        doc.insert_before(root, x, None, t).unwrap();
        println!("{text}");
    });
}
