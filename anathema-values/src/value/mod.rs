use std::fmt::{self, Display, Debug};
use std::rc::Rc;

use anathema_render::Color;

pub use self::num::Num;
pub use self::owned::Owned;

use crate::map::Map;
use crate::{Collection, List, ValueExpr};

mod num;
mod owned;

// -----------------------------------------------------------------------------
//   - Value ref -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum ValueRef<'a> {
    Str(&'a str),
    Map(&'a dyn Collection),
    List(&'a dyn Collection),
    Expressions(&'a [ValueExpr]),
    Owned(Owned),
}

impl<'a> ValueRef<'a> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Str(s) => s.is_empty(),
            Self::Owned(Owned::Bool(b)) => *b,
            _ => false,
        }
    }
}

impl<'a> PartialEq for ValueRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Str(lhs), Self::Str(rhs)) => lhs == rhs,
            (Self::Owned(lhs), Self::Owned(rhs)) => lhs == rhs,
            // (Self::Map(lhs), Self::Map(rhs)) => lhs.eq(rhs),
            // (Self::List(lhs), Self::List(rhs)) => lhs.eq(rhs),
            // TODO: see panic message
            _ => panic!("need equality for Collection trait"),
        }
    }
}

// -----------------------------------------------------------------------------
//   - From for value ref -
// -----------------------------------------------------------------------------
impl<'a, T: Debug> From<&'a Map<T>> for ValueRef<'a>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'a Map<T>) -> Self {
        Self::Map(value)
    }
}

impl<'a, T: Debug> From<&'a List<T>> for ValueRef<'a>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'a List<T>) -> Self {
        Self::List(value)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    fn from(value: &'a str) -> Self {
        ValueRef::Str(value)
    }
}

impl<'a, T: Into<Owned> + Copy> From<&'a T> for ValueRef<'a> {
    fn from(value: &'a T) -> Self {
        ValueRef::Owned((*value).into())
    }
}

// -----------------------------------------------------------------------------
//   - TryFrom -
// -----------------------------------------------------------------------------
impl<'a> TryFrom<ValueRef<'a>> for u64 {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(Owned::Num(Num::Unsigned(num))) => Ok(num),
            _ => Err(()),
        }
    }
}

impl TryFrom<ValueRef<'_>> for String {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s.to_string()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<ValueRef<'a>> for &'a str {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<ValueRef<'a>> for Color {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            // ValueRef::Str(s) => Ok(s),
            // _ => Err(())
            _ => panic!(),
        }
    }
}
