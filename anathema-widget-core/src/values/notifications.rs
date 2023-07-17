use std::collections::BTreeSet;
use std::mem::take;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

use parking_lot::Mutex;

use crate::node::NodeId;
use crate::Value;

static NOTIFICATIONS: OnceLock<Mutex<Vec<(Change, NodeId)>>> = OnceLock::new();

fn drain_notifications() -> Vec<(Change, NodeId)> {
    let v: &mut Vec<_> = &mut *NOTIFICATIONS.get_or_init(Default::default).lock();
    take(v)
}

fn push_notifications(change: Change, nodes: &BTreeSet<NodeId>) {
    let changes: &mut Vec<_> = &mut *NOTIFICATIONS.get_or_init(Default::default).lock();
    changes.extend(nodes.iter().map(|n| (change, n.clone())));
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    Modified,
    Add,
    Remove(usize),
    Swap(usize, usize),
}

#[derive(Debug)]
pub(crate) struct ValueWrapper {
    value: Value,
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

impl Deref for ValueWrapper {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for ValueWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.value {
            Value::Map(_) => {}
            Value::List(_) => {}
            _ => push_notifications(Change::Modified, &self.subscribers.lock()),
        }
        &mut self.value
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

impl Into<Value> for ValueWrapper {
    fn into(self) -> Value {
        self.value
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn register_change() {
        let mut value = ValueWrapper::from(1);
        value.sub(NodeId::root());
        assert!(drain_notifications().is_empty());
        value.deref_mut();
        assert_eq!(drain_notifications().remove(0), (Change::Modified, NodeId::root().clone()));
    }

    #[test]
    fn add_to_collection() {
        let mut value = ValueWrapper::from(Vec::<Value>::new());
        value.sub(NodeId::root());
        let Value::List(list) = value.deref_mut() else { panic!() };
        list.push(1);
        assert_eq!(drain_notifications().remove(0), (Change::Add, NodeId::root().clone()));
    }
}
