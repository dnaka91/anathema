use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::node::NodeEval;


pub struct Single;

impl Layout for Single {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        mut nodes: NodeEval<'widget>,
        size: &mut Size,
    ) -> Result<()> {
        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();

        if let Some(widget) = nodes.next(&mut values).transpose()? {
            *size = match widget.layout(ctx.parent_id, constraints, &values) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => return Ok(()),
                err @ Err(_) => err?,
            };
        }

        Ok(())
    }
}
