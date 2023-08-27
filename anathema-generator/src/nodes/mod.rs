use std::ops::DerefMut;
use std::rc::Rc;

use anathema_values::{State, Path};

pub use self::id::NodeId;
use crate::expressions::Expression;
use crate::{IntoWidget, Value};

mod id;

#[derive(Debug)]
pub struct Node<Widget: IntoWidget> {
    pub node_id: NodeId,
    pub kind: NodeKind<Widget>,
}

#[cfg(test)]
impl<Widget: IntoWidget> Node<Widget> {
    pub(crate) fn single(&mut self) -> (&mut Widget, &mut Nodes<Widget>) {
        match &mut self.kind {
            NodeKind::Single(inner, nodes) => (inner, nodes),
            _ => panic!()
        }
    }
}

#[derive(Debug)]
pub enum NodeKind<Widget: IntoWidget> {
    Single(Widget, Nodes<Widget>),
    Loop {
        body: Nodes<Widget>,
        binding: Path,
        collection: Box<[Value]>,
        value_index: usize,
    },
}

#[derive(Debug)]
// TODO: possibly optimise this by making nodes optional on the node
pub struct Nodes<Widget: IntoWidget> {
    expressions: Rc<[Expression<Widget>]>,
    inner: Vec<Node<Widget>>,
    active_loop: Option<Box<Node<Widget>>>,
    expr_index: usize,
    next_id: NodeId,
}

impl<Widget: IntoWidget> Nodes<Widget> {
    pub(crate) fn new(expressions: Rc<[Expression<Widget>]>, next_id: NodeId) -> Self {
        Self {
            expressions,
            inner: vec![],
            active_loop: None,
            expr_index: 0,
            next_id,
        }
    }

    fn reset(&mut self) {
        self.expr_index = 0;
    }

    fn eval_active_loop(&mut self, state: &mut Widget::State) -> Option<Result<(), Widget::Err>> {
        if let Some(active_loop) = self.active_loop.as_mut() {
            let Node {
                kind:
                    NodeKind::Loop {
                        body,
                        loop_repr,
                        value_index,
                    },
                node_id: parent_id,
            } = active_loop.deref_mut()
            else { unreachable!() };

            match body.next(state) {
                result @ Some(_) => return result,
                None => {
                    *value_index += 1;
                    if *value_index == loop_repr.collection.len() {
                        self.inner.push(*self.active_loop.take().expect(""));
                    } else {
                        // Scope the value
                        body.reset();
                    }
                }
            }
        }

        self.next(state)
    }

    pub fn next(&mut self, state: &mut Widget::State) -> Option<Result<(), Widget::Err>> {
        if let ret @ Some(_) = self.eval_active_loop(state) {
            return ret;
        }

        let expr = self.expressions.get(self.expr_index)?;
        let node = match expr.eval(state, self.next_id.clone()) {
            Ok(node) => node,
            Err(e) => return Some(Err(e)),
        };
        match node.kind {
            NodeKind::Loop { .. } => {
                self.active_loop = Some(node.into());
                self.next(state)
            }
            NodeKind::Single(element, node) => {
                self.expr_index += 1;
                Some(Ok(()))
            }
        }
    }
}
