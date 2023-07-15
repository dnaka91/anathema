use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::node::NodeEval;
use anathema_widget_core::WidgetContainer;

pub struct Stacked;

impl Layout for Stacked {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        mut nodes: NodeEval<'widget>,
        size: &mut Size,
    ) -> Result<()> {
        let mut width = 0;
        let mut height = 0;

        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();

        while let Some(widget) = nodes.next(&mut values).transpose()? {
            let size = match widget.layout(ctx.parent_id, constraints, &values) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            width = width.max(size.width);
            height = height.max(size.height);
        }

        size.width = size.width.max(width);
        size.height = size.height.max(height);

        Ok(())
    }
}
