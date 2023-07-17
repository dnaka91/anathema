use std::borrow::Cow;

use super::expressions::Expression;
use super::generator::Direction;
use super::index::Index;
use super::store::Values;
use super::ValueRef;
use crate::error::Result;
use crate::node::{Action, Node, NodeId};
use crate::template::Template;
use crate::values::notifications::ValueWrapper;
use crate::{Factory};

enum State<'parent> {
    Block,
    Loop {
        body: &'parent [Template],
        binding: &'parent str,
        collection: &'parent [ValueWrapper],
        value_index: Index,
    },
}

// -----------------------------------------------------------------------------
//   - Scope -
// -----------------------------------------------------------------------------
pub struct Scope<'parent> {
    pub(crate) expressions: Vec<Expression<'parent>>,
    parent_id: &'parent NodeId,
    state: State<'parent>,
    inner: Option<Box<Scope<'parent>>>,
    index: Index,
}

impl<'parent> Scope<'parent> {
    pub(crate) fn new(
        parent_id: &'parent NodeId,
        templates: &'parent [Template],
        values: &Values<'parent>,
        dir: Direction,
    ) -> Self {
        let expressions = templates
            .iter()
            .map(|t| t.to_expression(values))
            .collect::<Vec<_>>();

        Self {
            parent_id,
            index: Index::new(dir, templates.len()),
            expressions,
            inner: None,
            state: State::Block,
        }
    }

    pub(super) fn reverse(&mut self) {
        self.index.reverse();

        if let State::Loop {
            value_index: value, ..
        } = &mut self.state
        {
            value.reverse();
        }

        if let Some(scope) = &mut self.inner {
            scope.reverse();
        }
    }

    pub(super) fn flip(&mut self) {
        self.index.flip();

        if let State::Loop {
            value_index: value, ..
        } = &mut self.state
        {
            value.flip();
        }

        if let Some(scope) = &mut self.inner {
            scope.flip();
        }
    }

    pub(crate) fn next(&mut self, values: &mut Values<'parent>) -> Option<Result<Action>> {
        loop {
            match self.inner.as_mut().and_then(|scope| scope.next(values)) {
                next @ Some(_) => break next,
                None => self.inner = None,
            }

            match &mut self.state {
                State::Block => {
                    let index = self.index.next()?;
                    let expr = &self.expressions[index];

                    match expr {
                        Expression::Node(template) => {
                            let container =
                                match Factory::exec(self.parent_id.next(), template, values) {
                                    Ok(container) => container,
                                    Err(e) => break Some(Err(e)),
                                };

                            let node = Node::Single(container);
                            break Some(Ok(Action::Add(node)));
                        }
                        Expression::View(id) => {
                            let view = match values.root.views.get(&*id) {
                                Some(view) => view,
                                None => continue,
                            };
                            let templates = view.templates();
                            let scope =
                                Scope::new(self.parent_id, &templates, values, self.index.dir);
                            self.inner = Some(Box::new(scope));
                        }
                        Expression::For {
                            body,
                            binding,
                            collection,
                        } => {
                            self.state = State::Loop {
                                body,
                                collection,
                                binding,
                                value_index: Index::new(self.index.dir, collection.len()),
                            };
                            break Some(Ok(Action::StartCollection));
                        }
                        Expression::Block(templates) => {
                            let scope =
                                Scope::new(self.parent_id, templates, values, self.index.dir);
                            self.inner = Some(Box::new(scope));
                        }
                    }
                }
                State::Loop {
                    body,
                    binding,
                    collection,
                    value_index: value,
                } => {
                    let value = match value.next() {
                        Some(idx) => &collection[idx],
                        None => {
                            self.state = State::Block;
                            break Some(Ok(Action::EndCollection));
                        }
                    };

                    values.set(Cow::Borrowed(binding), ValueRef::Borrowed(&value));

                    let scope = Scope::new(self.parent_id, body, values, self.index.dir);
                    self.inner = Some(Box::new(scope));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    // use std::iter::zip;

    // use crate::gen::testing::*;
    // use crate::template::*;
    // use crate::TextPath;

    // fn for_loop(size: usize) -> (Vec<String>, TestSetup) {
    //     let text = crate::TextPath::fragment("x");
    //     let values = (0..size).map(|v| v.to_string()).collect::<Vec<_>>();
    //     let for_loop = template_for("x", values.clone(), [test_template(text)]);
    //     let setup = TestSetup::new().template(for_loop);
    //     (values, setup)
    // }

    // #[test]
    // fn empty_scope() {
    //     let mut setup = TestSetup::new();
    //     let mut scope = setup.scope();
    //     assert!(scope.next().is_none());
    // }

    // #[test]
    // fn generate_single_widget() {
    //     let text = TextPath::fragment("beverage");

    //     let mut setup = TestSetup::with_templates([test_template(text)]).set("beverage", "tea");
    //     let mut scope = setup.scope();

    //     let text = scope.next_unchecked();
    //     let text = text.to_ref::<TestWidget>();
    //     assert_eq!(&text.0, "tea");
    // }

    // #[test]
    // fn generate_loop() {
    //     let (values, mut setup) = for_loop(5);
    //     let scope = setup.scope();

    //     for (a, b) in zip(values, scope) {
    //         assert_eq!(a, b.to_ref::<TestWidget>().0);
    //     }
    // }

    // #[test]
    // fn flip_loop() {
    //     let (values, mut setup) = for_loop(5);
    //     let mut scope = setup.scope();
    //     scope.inner.flip();

    //     for (a, b) in zip(values.into_iter().rev(), scope) {
    //         assert_eq!(a, b.to_ref::<TestWidget>().0);
    //     }
    // }

    // #[test]
    // fn reverse_loop() {
    //     let (_values, mut setup) = for_loop(2);
    //     let mut scope = setup.scope();

    //     assert_eq!("0", scope.next_assume_text());
    //     assert_eq!("1", scope.next_assume_text());
    //     scope.inner.reverse();
    //     assert_eq!("0", scope.next_assume_text());
    // }
}
