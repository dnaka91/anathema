use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layout};
use anathema_widget_core::WidgetContainer;
use anathema_widget_core::node::NodeEval;

use super::many::Many;

pub struct Horizontal(Many);

impl Horizontal {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Horizontal, 0, false);
        Self(many)
    }
}

impl Layout for Horizontal {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        nodes: NodeEval<'widget>,
        size: &mut Size,
    ) -> Result<()> {
        self.0.layout(ctx, nodes, size)
    }
}
