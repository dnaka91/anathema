use std::fmt::Display;

use crate::scope::Num;
use crate::{Context, NodeId, Path, ScopeValue, StaticValue};

#[derive(Debug)]
pub enum ValueExpr {
    Ident(String),
    Value(Box<ScopeValue>),
    Num(Num),
    Lookup(Box<ValueExpr>),
    Key(Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),
    Dot(Box<ValueExpr>, Box<ValueExpr>),
    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    Bool(Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),
}

impl Display for ValueExpr  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(s) => write!(f, "{s}"),
            Self::Num(n) => write!(f, "{n}"),
            Self::Lookup(n) => write!(f, "{n}"),
            Self::Key(n) => write!(f, "{n}"),
            Self::Index(lhs, idx) => write!(f, "{lhs}[{idx}]"),
            Self::Dot(lhs, rhs) => write!(f, "{lhs}.{rhs}"),
            _ => panic!("{self:#?}")
        }
    }
}

// ItemState {
//     name: Value<String>,
//     age: Value<usize>,
// }
//
// RootState {
//    collection: List<ItemState>,
//    root_num: Value<u32>,
// }
//
// Template
// --------
//
// // scope value `item` from collection, subscribe for-loop to `collection`
// for item in collection
//     ValueExpr::Add(
//         ValueExpr::Val(Dyn("item", "age")), ValueExpr::Sub(
//             Dyn("root_num"),
//             Static(1)
//         )
//     )
//     text "{{ item.age + root_num - 1 }}"

impl ValueExpr {
    // fn eval_num(&self, scope: &Scope, state: &mut impl State, node_id: Option<&NodeId>) -> ?? {
    // }

    fn eval<T>(&self, context: &Context, node_id: Option<&NodeId>) -> &ScopeValue {
        match self {
            Self::Value(val) => val,
            Self::Add(lhs, rhs) => {
                panic!()
                // let lhs = lhs.eval(scope, state, node_id).to_num() or return Invalid;
                // let rhs = rhs.eval(scope, state, node_id).to_num() or return Invalid;

                // match (lhs, rhs) {

                //     (Num::Unsigned(lhs), Num::Signed(rhs @ 0..=i64::MAX)) => lhs + rhs as u64,
                //     (Num::Unsigned(lhs), Num::Signed(rhs) => {
                //         let rhs = rhs.abs();
                //         if lhs > rhs {
                //             lhs + rhs
                //         }
                //     }
                //     (Num::Unsigned(lhs), Num::Signed(rhs) => as i64,
                //     (Num::Unsigned(lhs), Num::Float(rhs) => lhs as f64 + rhs

                //     (Num::Unsigned(lhs), Num::Unsigned(rhs)) => lhs + rhs,
                //     (Num::Signed(lhs), Num::Signed(rhs)) => lhs + rhs,
                //     (Num::Float(lhs), Num::Float(rhs)) => lhs + rhs,
                // }
            }
            _ => &ScopeValue::Invalid,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolve_something() {
        panic!("oh my")
    }
}