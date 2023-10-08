use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{Collection, NodeId, Path, ValueRef};

mod value;

pub trait State {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>;
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        self.deref().get(key, node_id)
    }
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        None
    }
}