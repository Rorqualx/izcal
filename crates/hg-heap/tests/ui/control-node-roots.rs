//! Positive control: the node-space API (node roots, a node scope and a
//! node field) compiles under the same brand.
//!
//! EXPECT: compiles.
use hg_heap::{Heap, NodeRef};

hg_heap::object! {
    /// A wrapper for a node.
    pub Wrapper {
        node OWNER,
        tail none,
    }
}

pub fn case(a: &mut Heap) {
    a.enter(|m| {
        m.scope(|m, s| {
            let w = m.alloc::<Wrapper>(s, 0).expect("alloc");
            let node = NodeRef::new(1, 1);
            w.get(m).set_node(m, Wrapper::OWNER, Some(node));
            let id = m.add_node_root(node);
            m.remove_node_root(id);
            m.node_scope(|m, ns| {
                let _ = ns.root(m, node);
            });
        });
    });
}
