use std::collections::{BTreeSet, HashMap};

use parking_lot::Mutex;

use crate::node::NodeId;
use crate::values::notifications::ValueWrapper;
use crate::views::ViewCollection;
use crate::Value;

#[derive(Debug, Default)]
pub struct DataCtx {
    data: HashMap<String, ValueWrapper>,
    pub views: ViewCollection,
}

impl DataCtx {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        let mut value = value.into();
        self.data.insert(key.into(), ValueWrapper::new(value.into()));
    }

    pub(crate) fn by_key(&self, key: &str) -> Option<&ValueWrapper> {
        self.data.get(key)
    }

    pub fn get_mut_or<T: 'static>(&mut self, key: &str, val: T) -> &mut T
    where
        for<'a> &'a mut T: TryFrom<&'a mut Value, Error = ()>,
        T: Into<Value>,
    {
        let v = self.data
            .entry(key.into())
            .or_insert(ValueWrapper::new(val.into()));

        let v = &mut v.value;
        v.try_into().expect("values was just added")
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.data.get_mut(key).map(|v| &mut v.value)?.try_into().ok()
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.data.get(key).map(|v| &v.value)?.try_into().ok()
    }
}
