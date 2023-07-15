use super::scope::Scope;
use super::store::Values;
use crate::error::Result;
use crate::node::Nodes;
use crate::template::Template;
use crate::WidgetContainer;

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
pub struct Generator<'parent> {
    scope: Scope<'parent>,
}

impl<'parent> Generator<'parent> {
    pub fn new(nodes: &'parent Nodes, values: &mut Values<'parent>) -> Self {
        Self {
            scope: Scope::new(&nodes.templates, values, Direction::Forward),
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

    pub fn next(&mut self, values: &mut Values<'parent>) -> Option<Result<WidgetContainer>> {
        self.scope.next(values)
    }
}
