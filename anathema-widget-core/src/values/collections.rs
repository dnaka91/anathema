use std::borrow::Borrow;
use std::collections::HashMap;

use parking_lot::Mutex;

use super::Value;
use crate::node::NodeId;
use crate::values::notifications::ValueWrapper;

// -----------------------------------------------------------------------------
//   - Collection -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Collection {
    values: Vec<ValueWrapper>,
    subscribers: Mutex<Vec<NodeId>>,
}

impl Collection {
    pub fn new(values: Vec<Value>) -> Self {
        Self {
            values: values.into_iter().map(ValueWrapper::new).collect(),
            subscribers: Mutex::new(vec![]),
        }
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.values
            .get_mut(index)
            .map(|v| &mut v.value)?
            .try_into()
            .ok()
    }

    pub fn get_ref<T: 'static>(&self, index: usize) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.values.get(index).map(|v| &v.value)?.try_into().ok()
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(ValueWrapper::new(value))
    }

    pub fn remove(&mut self, index: usize) -> Value {
        // notify subscribers
        self.values.remove(index).value
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        // notify subscribers
        self.values.swap(a, b)
    }

    pub(crate) fn as_slice(&self) -> &[ValueWrapper] {
        self.values.as_slice()
    }
}

impl PartialEq for Collection {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Clone for Collection {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            subscribers: Mutex::new(vec![]),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Map -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Map {
    pub(crate) values: HashMap<String, ValueWrapper>,
}

impl Map {
    pub(crate) fn new(values: HashMap<String, ValueWrapper>) -> Self {
        Self { values }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> Option<Value> {
        // notify subscribers
        self.values
            .insert(key.into(), ValueWrapper::new(value.into()))
            .map(|v| v.value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Value>
    where
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.remove(key).map(|v| v.value)
    }

    pub(crate) fn get_wrapper<Q>(&self, key: &Q) -> Option<&ValueWrapper>
    where
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.get(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Value>
    where
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.get(key).map(|v| &v.value)
    }

    pub fn get_ref<T: 'static, Q>(&self, key: &Q) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.get(key).map(|v| &v.value)?.try_into().ok()
    }

    pub fn get_mut<T: 'static, Q>(&mut self, key: &Q) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values
            .get_mut(key)
            .map(|v| &mut v.value)?
            .try_into()
            .ok()
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Clone for Map {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}
