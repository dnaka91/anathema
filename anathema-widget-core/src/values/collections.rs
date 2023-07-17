use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

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
}

impl Collection {
    pub fn new(values: Vec<Value>) -> Self {
        Self {
            values: values.into_iter().map(ValueWrapper::new).collect(),
        }
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.values
            .get_mut(index)
            .map(|v| v.deref_mut())?
            .try_into()
            .ok()
    }

    pub fn get_ref<T: 'static>(&self, index: usize) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.values.get(index).map(|v| v.deref())?.try_into().ok()
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.values.push(ValueWrapper::new(value.into()))
    }

    pub fn remove(&mut self, index: usize) -> Value {
        // notify subscribers
        self.values.remove(index).into()
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
            .map(|v| v.into())
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Value>
    where
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.remove(key).map(|v| v.into())
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
        self.values.get(key).map(|v| v.deref())
    }

    pub fn get_ref<T: 'static, Q>(&self, key: &Q) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
        String: Borrow<Q>,
        Q: ?Sized,
        Q: std::hash::Hash + PartialEq + Eq,
    {
        self.values.get(key).map(|v| v.deref())?.try_into().ok()
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
            .map(|v| v.deref_mut())?
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
