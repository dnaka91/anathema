use std::collections::{BTreeSet, HashMap};

use parking_lot::Mutex;

use crate::node::NodeId;
use crate::values::Notifier;
use crate::views::ViewCollection;
use crate::Value;

#[derive(Debug, Default)]
pub struct DataCtx {
    data: HashMap<String, Value>,
    pub views: ViewCollection,
    notifier: Notifier,
}

impl DataCtx {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        let mut value = value.into();
        if let Value::List(col) = &mut value {
            col.notifier.replace(self.notifier);
        }
        self.data.insert(key.into(), value);
    }

    pub fn by_key(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>
    where
        for<'a> &'a mut Value: TryInto<&'a mut T>,
    {
        self.data.get_mut(key)?.try_into().ok()
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.data.get(key)?.try_into().ok()
    }
}
