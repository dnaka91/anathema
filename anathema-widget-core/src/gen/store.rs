use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::Deref;

use super::ValueRef;
use crate::contexts::DataCtx;
use crate::node::NodeId;
use crate::values::notifications::ValueWrapper;
use crate::views::View;
use crate::{Fragment, Path, TextPath, Value};

// -----------------------------------------------------------------------------
//   - Layout -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct ScopedValues<'parent>(HashMap<Cow<'parent, str>, ValueRef<'parent>>);

impl<'parent> ScopedValues<'parent> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: Cow<'parent, str>, value: ValueRef<'parent>) {
        self.0.insert(key, value);
    }

    pub fn by_key(&self, key: &str) -> Option<&ValueRef<'parent>> {
        self.0.get(key)
    }

    pub fn set(&mut self, key: Cow<'parent, str>, val: ValueRef<'parent>) {
        self.0.insert(key, val);
    }
}

// -----------------------------------------------------------------------------
//   - Values -
// -----------------------------------------------------------------------------
pub struct Values<'parent> {
    root: &'parent DataCtx,
    parent: Option<&'parent Values<'parent>>,
    inner: ScopedValues<'parent>,
}

impl<'parent> Values<'parent> {
    pub(crate) fn sneaky_log_rename_me(&self, path: &Path) {
        #[cfg(feature = "logging")]
        {
            log::info!("path: {path:?}");
        }
        self.root.log(path.clone());
    }

    pub(crate) fn get_view(&self, key: &str) -> Option<&'parent dyn View> {
        self.root.get_view(key)
    }

    // TODO: this function should probably not get a node id
    pub fn text_to_string(&self, text: &'parent TextPath, node_id: &NodeId) -> Cow<'parent, str> {
        match text {
            TextPath::Fragments(fragments) => {
                let mut output = String::new();
                for fragment in fragments {
                    match fragment {
                        Fragment::String(s) => output.push_str(s),
                        Fragment::Data(path) => {
                            match path.lookup_value(self) {
                                Some(val) => {
                                    write!(&mut output, "{}", val.deref());
                                    val.sub(node_id);
                                }
                                None => {
                                    panic!();
                                    self.sneaky_log_rename_me(path);
                                }
                            };
                        }
                    }
                }
                Cow::Owned(output)
            }
            TextPath::String(s) => Cow::from(s),
        }
    }

    pub fn new(root: &'parent DataCtx) -> Self {
        Self {
            root,
            parent: None,
            inner: ScopedValues::new(),
        }
    }

    pub fn next(&self) -> Values<'_> {
        Values {
            root: self.root,
            parent: Some(&self),
            inner: ScopedValues::new(),
        }
    }

    pub fn get_ref<T: 'static>(&self, key: &str) -> Option<&T>
    where
        for<'a> &'a Value: TryInto<&'a T>,
    {
        self.get_value(key).map(|v| v.deref())?.try_into().ok()
    }

    pub(crate) fn get_value(&self, key: &str) -> Option<&ValueWrapper> {
        self.inner
            .by_key(key)
            .and_then(|v| v.value())
            .or_else(|| self.parent.and_then(|p| p.get_value(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub(crate) fn get_borrowed_value(&self, key: &str) -> Option<&'parent ValueWrapper> {
        self.inner
            .by_key(key)
            .and_then(|v| v.borrowed())
            .or_else(|| self.parent.and_then(|p| p.get_value(key)))
            .or_else(|| self.root.by_key(key))
    }

    pub(crate) fn set(&mut self, key: Cow<'parent, str>, val: ValueRef<'parent>) {
        self.inner.set(key, val);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn root() -> DataCtx {
        let mut root = DataCtx::default();
        root.insert("key".to_string(), 1);
        root
    }

    #[test]
    fn get_nested_values() {
        let root = root();
        let values = Values::new(&root);
        assert_eq!(*values.get_ref::<i64>("key").unwrap(), 1);

        let mut values = values.next();
        values.set("key2".into(), ValueRef::Owned(2.into()));
        let value = values.get_ref::<i64>("key2").unwrap();
        assert_eq!(*value, 2);

        let value_1 = values.get_value("key").unwrap();
        let mut values = values.next();
        values.set("key2".into(), ValueRef::Borrowed(value_1));
        assert_eq!(*values.get_ref::<i64>("key2").unwrap(), 1);
    }
}
