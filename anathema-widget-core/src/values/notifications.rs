use std::collections::BTreeSet;
use std::sync::OnceLock;

use parking_lot::Mutex;

use crate::node::NodeId;
use crate::Value;

static NOTIFICATIONS: OnceLock<Mutex<Vec<(Change, NodeId)>>> = OnceLock::new();

#[derive(Debug)]
pub enum Change {
    Changed,
    Add,
    Remove(usize),
    Swap(usize, usize),
}

#[derive(Debug)]
pub(crate) struct ValueWrapper {
    pub(crate) value: Value,
    subscribers: Mutex<BTreeSet<NodeId>>,
}

impl ValueWrapper {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            subscribers: Mutex::new(BTreeSet::new()),
        }
    }

    pub fn sub(&self, node_id: &NodeId) {
        self.subscribers.lock().insert(node_id.clone());
    }
}

impl Clone for ValueWrapper {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: Mutex::new(BTreeSet::new()),
        }
    }
}

impl PartialEq for ValueWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> From<T> for ValueWrapper
where
    Value: From<T>,
{
    fn from(val: T) -> Self {
        Self::new(val.into())
    }
}
