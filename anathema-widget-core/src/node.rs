use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::template::Template;
use crate::WidgetContainer;

pub struct NodeId(Vec<usize>);

impl NodeId {
    pub fn next(parent: &[usize]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);

        let mut inner = Vec::with_capacity(parent.len() + 1);
        inner.extend(parent);
        inner.push(NEXT.fetch_add(1, Ordering::Relaxed));
        Self(inner)
    }
}

pub enum NodeKind {
    Single(WidgetContainer),
    Collection(Vec<WidgetContainer>),
}

pub struct Node {
    id: NodeId,
    kind: NodeKind,
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

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WidgetContainer> + '_ {
        self.inner.iter_mut().map(|node| {
            let ret: Box<dyn Iterator<Item = &mut WidgetContainer>> = match node.kind {
                NodeKind::Single(ref mut widget) => Box::new(std::iter::once(widget)),
                NodeKind::Collection(ref mut widgets) => Box::new(widgets.iter_mut()),
            };
            ret
        }).flatten()
    }
}
