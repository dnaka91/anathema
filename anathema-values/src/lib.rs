use std::cell::RefCell;

pub use self::id::NodeId;
pub use self::list::List;
pub use self::map::Map;
pub use self::path::Path;
pub use self::scope::{Collection, Context, Num, Owned, Scope, ScopeValue};
pub use self::slab::Slab;
pub use self::state::{Change, State, StateValue};
pub use self::value_expr::ValueExpr;
pub use self::{Value, ValueRef};

pub mod hashmap;
mod path;

mod id;
mod list;
mod map;
mod scope;
mod slab;
mod state;
mod value;
mod value_expr;

thread_local! {
    static DIRTY_NODES: RefCell<Vec<(NodeId, Change)>> = Default::default();
    static REMOVED_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<(NodeId, Change)> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}

pub fn remove_node(node: NodeId) {
    REMOVED_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

// #[cfg(testing)]
pub mod testing;
