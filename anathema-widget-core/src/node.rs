use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::template::Template;
use crate::{Values, WidgetContainer};

static ROOT: NodeId = NodeId(Vec::new());

pub enum Action {
    Add(Node),
    StartCollection,
    EndCollection,
}

#[derive(Debug, Clone)]
pub struct NodeId(Vec<usize>);

impl NodeId {
    pub fn next(&self) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);

        let mut inner = Vec::with_capacity(self.0.len() + 1);
        inner.extend(&self.0);
        inner.push(NEXT.fetch_add(1, Ordering::Relaxed));
        Self(inner)
    }

    pub fn root() -> &'static NodeId {
        &ROOT
    }
}

pub enum Node {
    Single(WidgetContainer),
    Collection(Vec<WidgetContainer>),
}

impl Node {
    fn count(&self) -> usize {
        match self {
            Self::Single(node) => 1 + node.children.count(),
            Self::Collection(nodes) => nodes.iter().map(|n| n.children.count()).sum(),
        }
    }
}

pub struct Nodes {
    inner: Vec<Node>,
    pub(crate) templates: Arc<[Template]>,
}

impl Nodes {
    pub fn new(templates: impl Into<Arc<[Template]>>) -> Self {
        Self {
            templates: templates.into(),
            inner: vec![],
        }
    }

    pub fn first_mut(&mut self) -> Option<&mut WidgetContainer> {
        self.iter_mut().next()
    }

    /// Iterate over the first layer of widget containers.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetContainer> + '_ {
        self.inner
            .iter_mut()
            .map(|node| {
                let ret: Box<dyn Iterator<Item = &mut WidgetContainer>> = match node {
                    Node::Single(ref mut widget) => Box::new(std::iter::once(widget)),
                    Node::Collection(ref mut widgets) => Box::new(widgets.iter_mut()),
                };
                ret
            })
            .flatten()
    }

    pub fn gen<'a>(&'a mut self, ctx: LayoutCtx<'a, '_>) -> NodeEval<'a> {
        NodeEval {
            inner: &mut self.inner,
            gen: Generator::new(&ctx, &self.templates),
        }
    }

    /// Count **all** the widget containers in the entire tree.
    pub fn count(&self) -> usize {
        self.inner.iter().map(|node| node.count()).sum()
    }
}

pub struct NodeEval<'a> {
    inner: &'a mut Vec<Node>,
    gen: Generator<'a>,
}

impl<'a> NodeEval<'a> {
    pub fn next(&mut self, values: &mut Values<'a>) -> Option<Result<&mut WidgetContainer>> {
        let action = match self.gen.next(values)? {
            Ok(n) => n,
            Err(e) => return Some(Err(e)),
        };

        match action {
            Action::Add(node) => {
                self.inner.push(node);
                match self.inner.last_mut() {
                    Some(Node::Single(el)) => Some(Ok(el)),
                    _ => unreachable!(),
                }
            }
            Action::StartCollection => self.next(values),
            Action::EndCollection => self.next(values),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetContainer> + '_ {
        self.inner
            .iter_mut()
            .map(|node| {
                let ret: Box<dyn Iterator<Item = &mut WidgetContainer>> = match node {
                    Node::Single(ref mut widget) => Box::new(std::iter::once(widget)),
                    Node::Collection(ref mut widgets) => Box::new(widgets.iter_mut()),
                };
                ret
            })
            .flatten()
    }

    pub fn flip(&mut self) {
        self.gen.flip()
    }

    pub fn reverse(&mut self) {
        self.gen.reverse()
    }

    pub fn pop(&mut self) -> Option<WidgetContainer> {
        let last = self.inner.pop()?;
        match last {
            Node::Single(wc) => Some(wc),
            Node::Collection(mut nodes) => {
                let wc = nodes.pop()?;
                self.inner.push(Node::Collection(nodes));
                Some(wc)
            }
        }
    }
}
