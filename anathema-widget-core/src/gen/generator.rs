use super::scope::Scope;
use super::store::Values;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::node::{Action, Node, NodeCache};
use crate::template::Template;

struct CacheIndex(usize);

// -----------------------------------------------------------------------------
//   - Direction -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Forward,
    Backward,
}

// -----------------------------------------------------------------------------
//   - Generator -
// -----------------------------------------------------------------------------
pub(crate) struct Generator<'parent> {
    scope: Scope<'parent>,
}

impl<'parent> Generator<'parent> {
    pub fn new(ctx: &LayoutCtx<'parent, 'parent>, templates: &'parent [Template]) -> Self {
        Self {
            scope: Scope::new(
                ctx.parent_id,
                templates,
                ctx.values,
                Direction::Forward,
                0,
            ),
        }
    }

    /// Reverse the generator from its current position
    pub fn reverse(&mut self) {
        self.scope.reverse();
    }

    /// Flip the generator to start from the end and change direction.
    pub fn flip(&mut self) {
        self.scope.flip();
    }

    pub fn next(&mut self, values: &mut Values<'parent>, node_cache: &mut NodeCache) -> Option<Result<Action>> {
        self.scope.next(values, node_cache)
    }
}
