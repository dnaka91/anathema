use std::ops::Deref;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;

use parking_lot::{
    Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard,
};

pub use self::map::Map;
use self::map::MapRef;
use crate::generation::Generation;
use crate::hashmap::{HashMap, IntMap};
use crate::notifier::{Action, Notifier};
use crate::path::Paths;
use crate::scopes::{ScopeValue, Scopes};
use crate::slab::GenerationSlab;
use crate::values::{IntoValue, TryFromValue, TryFromValueMut};
use crate::{Container, Path, PathId, ScopeId, Truthy, ValueRef};

mod map;

struct Values2<T>(RefCell<GenerationSlab<Container<T>>>);

impl<T> Default for Values2<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Values2<T> {
    fn get(&self, value_ref: ValueRef<Container<T>>) -> Option<Ref<'_, Container<T>>> {
        let borrow = self.0.borrow();

        panic!()
        // Some(wrong)
        // panic!()
        // let () = Ref::map(borrow, |slab| slab.get(value_ref.index));
    }

    fn get_mut(&self, value_ref: ValueRef<Container<T>>) -> Option<RefMut<'_, Container<T>>> {
        panic!()
    }

    // Will panic if the value does not exist or isn't a map
    fn get_map(&self, value_ref: ValueRef<Container<T>>) -> Ref<'_, Map<T>> {
        let slab = self.0.borrow();
        Ref::map(slab, |gen| {
            let cont: &Container<_> = &*gen.get(value_ref.index).unwrap();
            match cont {
                Container::Map(map) => map,
                _ => panic!(),
            }
        })
    }

    fn push(&self, value: Container<T>) -> ValueRef<Container<T>> {
        self.0.borrow_mut().push(value)
    }
}

// -----------------------------------------------------------------------------
//   - Store 2 -
// -----------------------------------------------------------------------------
pub struct Store2<T> {
    root: Map<T>,
    values: Values2<T>,
    paths: Paths,
    scopes: Scopes<T>,
    notifier: Notifier<T>,
}

impl<T> Store2<T> {
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            root: Map::new(),
            values: Values2::default(),
            notifier: Notifier::new(sender),
            paths: Paths::empty(),
            scopes: Scopes::new(),
        }
    }

    pub fn get(&self, path: impl Into<Path>) -> Option<ValueRef<Container<T>>> {
        let path_id = self.paths.get(&path.into())?;
        self.root.get(path_id)
    }
}

// -----------------------------------------------------------------------------
//   - Global bucket -
// -----------------------------------------------------------------------------
/// A store contains a collection of `Container`s
pub struct Store<T> {
    values: RwLock<GenerationSlab<Container<T>>>,
    scopes: RwLock<Scopes<T>>,
    paths: RwLock<Paths>,
    notifier: Notifier<T>,
}

impl<T> Store<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            values: RwLock::new(GenerationSlab::with_capacity(cap)),
            scopes: RwLock::new(Scopes::with_capacity(cap)),
            paths: RwLock::new(Paths::empty()),
            notifier: Notifier::new(sender),
        }
    }

    pub fn empty() -> Self {
        Self::with_capacity(0)
    }

    /// Write causes a lock
    pub fn write(&mut self) -> StoreMut<'_, T> {
        StoreMut {
            slab: self.values.write(),
            scopes: self.scopes.write(),
            paths: &self.paths,
            notifier: &self.notifier,
        }
    }

    /// Read casues a lock.
    /// It's okay to have as many read locks as possible as long
    /// as there is no write lock
    pub fn read(&self) -> StoreRef<'_, T> {
        StoreRef {
            values: &self.values,
            paths: &self.paths,
            scopes: &self.scopes,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Bucket ref -
//
//   This is for the node generator
// -----------------------------------------------------------------------------
pub struct StoreRef<'a, T> {
    values: &'a RwLock<GenerationSlab<Container<T>>>,
    paths: &'a RwLock<Paths>,
    scopes: &'a RwLock<Scopes<T>>,
}

impl<'a, T: Truthy> StoreRef<'a, T> {
    pub fn check_true(&self, value_ref: ValueRef<Container<T>>) -> bool {
        self.values
            .read()
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
            .map(|val| val.is_true())
            .unwrap_or(false)
    }
}

impl<'a, T> StoreRef<'a, T> {
    pub fn read(&self) -> ReadOnly<'a, T> {
        ReadOnly {
            inner: self.values.read(),
        }
    }

    pub fn by_path(
        &self,
        path_id: PathId,
        scope: impl Into<Option<ScopeId>>,
    ) -> Option<ScopeValue<T>> {
        self.scopes.read().get(path_id, scope).cloned()
    }

    /// Try to get a value by path.
    /// If there is no value at a given path, insert an
    /// empty value and return the `ValueRef` to that.
    pub fn by_path_or_empty(
        &self,
        path_id: PathId,
        scope: impl Into<Option<ScopeId>>,
    ) -> ScopeValue<T> {
        match self.by_path(path_id, scope).clone() {
            Some(val) => val,
            None => {
                let value_ref = self.values.write().push(Container::Empty);
                ScopeValue::Dyn(value_ref)
            }
        }
    }

    pub fn new_scope(&self, parent: Option<ScopeId>) -> ScopeId {
        self.scopes.write().new_scope(parent)
    }

    pub fn scope_value(
        &self,
        path_id: PathId,
        value: ScopeValue<T>,
        scope: ScopeId,
    ) -> Option<ScopeValue<T>> {
        self.scopes.write().insert(path_id, value, scope)
    }
}

// -----------------------------------------------------------------------------
//   - Read-only values -
// -----------------------------------------------------------------------------
pub struct ReadOnly<'a, T> {
    inner: RwLockReadGuard<'a, GenerationSlab<Container<T>>>,
}

impl<'a, T> ReadOnly<'a, T> {
    pub fn get(&self, value_ref: ValueRef<Container<T>>) -> Option<&Container<T>> {
        self.inner
            .get(value_ref.index)
            .filter(|val| val.compare_generation(value_ref.gen))
            .map(std::ops::Deref::deref)
    }

    // TODO: reconsider this name
    pub fn getv2<V>(&self, value_ref: ValueRef<Container<T>>) -> Option<&V::Output>
    where
        V: TryFromValue<T>,
    {
        V::from_value(self.get(value_ref)?)
    }
}

// -----------------------------------------------------------------------------
//   - Bucket mut -
//
//   This is what is exposed to the user of the runtime
// -----------------------------------------------------------------------------
pub struct StoreMut<'a, T> {
    slab: RwLockWriteGuard<'a, GenerationSlab<Container<T>>>,
    scopes: RwLockWriteGuard<'a, Scopes<T>>,
    paths: &'a RwLock<Paths>,
    notifier: &'a Notifier<T>,
}

impl<'a, T> StoreMut<'a, T> {
    //pub(crate) fn remove(&mut self, value_ref: ValueRef<T>) -> Generation<Container<T>> {
    //    self.slab.remove(value_ref.index)
    //}

    //pub fn push(&mut self, value: T) -> ValueRef<Container<T>> {
    //    self.slab.push(Container::Value(value))
    //}

    pub fn insert_path(&mut self, path: impl Into<Path>) -> PathId {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        path_id
    }

    /// Insert a value at a given path.
    /// This will ensure the path will be created if it doesn't exist.
    ///
    /// This will only insert into the root scope.
    pub fn insert_at_path<V>(&mut self, path: impl Into<Path>, value: V) -> ValueRef<Container<T>>
    where
        V: IntoValue<T>,
    {
        let path = path.into();
        let path_id = self.paths.write().get_or_insert(path);
        self.insert_dyn(path_id, value)
    }

    /// Insert a value at a given path id.
    /// The value is inserted into the root scope,
    /// (A `BucketMut` should never operate on anything other than the root scope.)
    pub fn insert_dyn<V>(&mut self, path_id: PathId, value: V) -> ValueRef<Container<T>>
    where
        V: IntoValue<T>,
    {
        panic!("figure out if it makes sense to replace or just overwrite");
        // let value = value.into_value(&mut *self);
        // TODO: this can't always use Container::Value, it would make no sense!
        // match self.scopes.root.remove_dyn(path_id) {
        //     Some(value_ref) => self.slab.replace(value_ref, Container::Value(value)),
        //     None => {
        //         let value_ref = self.slab.push(Container::Value(value));
        //         self.scopes.insert(path_id, ScopeValue::Dyn(value_ref), None);
        //         value_ref
        //     }
        // }
    }

    //// // TODO: rename this to something more sensible
    //// pub fn getv2<V>(&self, path: impl Into<Path>) -> Option<&V::Output>
    //// where
    ////     V: TryFromValue<T>,
    //// {
    ////     let path = path.into();
    ////     let path_id = self.paths.write().get_or_insert(path);
    ////     self.get(path_id).and_then(|v| V::from_value(v))
    //// }

    //// // TODO: rename this, you know the drill
    //// pub fn getv2_mut<V>(&mut self, path: impl Into<Path>) -> Option<&mut V::Output>
    //// where
    ////     V: TryFromValueMut<T>,
    //// {
    ////     let path = path.into();
    ////     let path_id = self.paths.write().get_or_insert(path);
    ////     self.get_mut(path_id).and_then(|v| V::from_value(v))
    //// }

    //// pub fn get(&self, path_id: PathId) -> Option<&Generation<Container<T>>> {
    ////     let ScopeValue::Dyn(value_ref) = self.scopes.get(path_id, None)? else { return None };
    ////     self.slab
    ////         .get(value_ref.index)
    ////         .filter(|val| val.compare_generation(value_ref.gen))
    //// }

    //// pub fn get_mut(&mut self, path_id: PathId) -> Option<&mut Generation<Container<T>>> {
    ////     let value_ref = self.scopes.get(path_id, None)?;
    ////     self.by_ref_mut(value_ref)
    //// }

    //pub fn by_ref_mut(
    //    &mut self,
    //    value_ref: ValueRef<Container<T>>,
    //) -> Option<&mut Generation<Container<T>>> {
    //    // Notify here
    //    self.notifier.notify(value_ref, Action::Modified);
    //    self.slab
    //        .get_mut(value_ref.index)
    //        .filter(|val| val.compare_generation(value_ref.gen))
    //}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hashmap::HashMap;
    use crate::List;

    #[test]
    fn store_from_map() {
        // let mut store = Store2::<String>::new();
        // let values = store.values();
        // values.new_map();

        // let mut hm = store.new_map();
        // hm.insert("name", Container::Value("Fin".into()));
        // // values.insert("people",

        // // {
        // //     let mut hm = store.new_map();
        // //     hm.insert("name", Container::Value("Fin".into()));
        // //     let mut people = store.new_map();
        // //     people.insert("fin", Container::Map(hm));
        // //     store.root.insert("people", people);
        // // }

        // // let values = store.values();
        // // let people = store.get("people").unwrap();
        // // let people = values.get_map(people).unwrap();
        // // let fin = people.get("fin").unwrap();
        // // let fin = values.get_map(fin).unwrap();

        // // panic!("{fin:#?}");

        // // let value_ref: ValueRef<Container<String>> = store.get("name").unwrap();
        // // let expected = "Fin";
        // // let values = store.values();
        // // let actual: &str = values.get_single(value_ref).unwrap();
        // // assert_eq!(expected, actual);
    }

    #[test]
    fn store_from_nested_maps() {
        // let mut hm = HashMap::new();
        // hm.insert("user".to_string(), Container::Map());
    }
}