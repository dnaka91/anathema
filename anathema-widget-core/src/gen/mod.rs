use crate::values::notifications::ValueWrapper;


pub(crate) mod expressions;
pub(crate) mod generator;
pub(crate) mod index;
mod scope;
pub(crate) mod store;

#[cfg(test)]
pub mod testing;

#[derive(Debug)]
pub(crate) enum ValueRef<'parent> {
    Owned(ValueWrapper),
    Borrowed(&'parent ValueWrapper),
}

impl<'parent> ValueRef<'parent> {
    pub fn value(&self) -> Option<&ValueWrapper> {
        match self {
            Self::Borrowed(val) => Some(val),
            Self::Owned(val) => Some(val),
        }
    }

    pub fn borrowed(&self) -> Option<&'parent ValueWrapper> {
        match self {
            Self::Borrowed(val) => Some(val),
            Self::Owned(_) => None,
        }
    }
}

impl<'parent> From<&'parent ValueWrapper> for ValueRef<'parent> {
    fn from(val: &'parent ValueWrapper) -> Self {
        ValueRef::Borrowed(val)
    }
}
