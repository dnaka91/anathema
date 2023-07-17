use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::template::Template;
use crate::{Values, WidgetContainer};

static ROOT: NodeId = NodeId(Vec::new());

pub enum Action {
    Cached(usize), // usize = cache index
    Add(Node),
    StartCollection,
    EndCollection,
}

// TODO: try a comparison in performance using Arc for the vec.
//       NodeId(Arc<[usize]>)
#[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Default)]
pub struct NodeId(pub Vec<usize>);

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

impl PartialEq<[usize]> for NodeId {
    fn eq(&self, rhs: &[usize]) -> bool {
        let max = self.0.len().min(rhs.len());
        self.0[..max].eq(&rhs[..max])
    }
}

#[derive(Debug)]
pub enum Node {
    Single(WidgetContainer),
    Collection(Vec<WidgetContainer>),
}

impl Node {
    fn count(&self) -> usize {
        match self {
            Self::Single(widget) => 1 + widget.children.count(),
            Self::Collection(widgets) => widgets.iter().map(|n| n.children.count()).sum(),
        }
    }

    fn by_id(&mut self, id: &[usize]) -> Option<&mut WidgetContainer> {
        match self {
            Self::Single(wc) => {
                if !wc.id.eq(id) {
                    None
                } else if wc.id.0.len() == id.len() {
                    Some(wc)
                } else {
                    wc.children.by_id(id)
                }
            }
            Self::Collection(widgets) => {
                for widget in widgets {
                    if !widget.id.eq(id) {
                        continue;
                    } else {
                        return  widget.children.by_id(id);
                    }
                }
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Nodes {
    cache: NodeCache,
    pub(crate) templates: Arc<[Template]>,
}

impl Nodes {
    pub fn new(templates: impl Into<Arc<[Template]>>) -> Self {
        Self {
            templates: templates.into(),
            cache: NodeCache { inner: vec![] },
        }
    }

    pub fn clear(&mut self) {
        self.cache.inner.clear();
    }

    pub fn first_mut(&mut self) -> Option<&mut WidgetContainer> {
        self.iter_mut().next()
    }

    pub fn by_id(&mut self, id: &[usize]) -> Option<&mut WidgetContainer> {
        self.cache.by_id(id)
    }

    /// Iterate over the first layer of widget containers.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetContainer> + '_ {
        self.cache
            .inner
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
            cache: &mut self.cache,
            gen: Generator::new(&ctx, &self.templates),
            next: vec![0],
        }
    }

    /// Count **all** the widget containers in the entire tree.
    pub fn count(&self) -> usize {
        self.cache.inner.iter().map(|node| node.count()).sum()
    }
}

// -----------------------------------------------------------------------------
//   - Node cache -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct NodeCache {
    inner: Vec<Node>,
}

impl NodeCache {
    fn by_id(&mut self, id: &[usize]) -> Option<&mut WidgetContainer> {
        for node in &mut self.inner {
            match node.by_id(id) {
                Some(wc) => return Some(wc),
                None => {}
            }
        }
        None
    }

    pub(crate) fn get_mut(&mut self, mut index: usize) -> Option<&mut WidgetContainer> {
        for node in &mut self.inner {
            match node {
                Node::Single(_) if index > 0 => index -= 1,
                Node::Single(wc) => return Some(wc),
                Node::Collection(widget_containers) => {
                    for wc in widget_containers {
                        if index == 0 {
                            return Some(wc);
                        }
                        index -= 1;
                    }
                }
            }
        }

        None
    }

    fn push(&mut self, node: Node) {
        self.inner.push(node);
    }

    fn last_mut(&mut self) -> Option<&mut Node> {
        self.inner.last_mut()
    }

    fn pop(&mut self) -> Option<WidgetContainer> {
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

// -----------------------------------------------------------------------------
//   - Node evaluator -
// -----------------------------------------------------------------------------
pub struct NodeEval<'a> {
    cache: &'a mut NodeCache,
    gen: Generator<'a>,
    next: Vec<usize>,
}

impl<'a> NodeEval<'a> {
    pub fn next(&mut self, values: &mut Values<'a>) -> Option<Result<&mut WidgetContainer>> {
        let action = match self.gen.next(values, &mut self.cache)? {
            Ok(n) => n,
            Err(e) => return Some(Err(e)),
        };

        match action {
            Action::Cached(index) => Ok(self.cache.get_mut(index)).transpose(),
            Action::Add(node) => {
                self.cache.push(node);
                match self.cache.last_mut() {
                    Some(Node::Single(el)) => Some(Ok(el)),
                    _ => unreachable!(),
                }
            }
            Action::StartCollection => self.next(values),
            Action::EndCollection => self.next(values),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetContainer> + '_ {
        self.cache
            .inner
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
        self.cache.pop()
    }
}
