use anathema_render::Size;
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Layouts;
use anathema_widget_core::node::{NodeEval, Nodes};
use anathema_widget_core::{
    AnyWidget, TextPath, ValuesAttributes, Widget, WidgetFactory,
};

use crate::layout::stacked::Stacked;

/// Unlike the [`HStack`](crate::HStack) or the [`VStack`](crate::VStack) the [`ZStack`] draws the
/// children on top of each other.
///
/// This makes it possible to draw widgets on top of other widgets.
///
/// An example adding a title to a border
/// ```ignore
/// use anathema_widgets::{ZStack, Position, Border, Text, Widget, NodeId, HorzEdge, VertEdge};
///
/// let mut zstack = ZStack::new(None, None).into_container(NodeId::anon());
///
/// // Border
/// let mut border = Border::thin(20, 5).into_container(NodeId::anon());
/// border.add_child(Text::with_text("Here is some text").into_container(NodeId::anon()));
/// zstack.add_child(border);
///
/// // Title
/// let mut position = Position::new(HorzEdge::Left(1), VertEdge::Top(0)).into_container(NodeId::anon());
/// position.add_child(Text::with_text("] Title [").into_container(NodeId::anon()));
/// zstack.add_child(position);
/// ```
/// output
/// ```text
/// ┌] Title [─────────┐
/// │Here is some text │
/// │                  │
/// │                  │
/// └──────────────────┘
/// ```
///
/// Note that widgets are drawn in the order they are inserted.
/// To make something like a dialogue box appear on top it would have to be the last child of the
/// `ZStack`.
#[derive(Debug, PartialEq)]
pub struct ZStack {
    /// Width
    pub width: Option<usize>,
    /// Height
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
}

impl ZStack {
    /// Create a new instance of a `ZStack`
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            min_width: None,
            min_height: None,
        }
    }
}

impl Widget for ZStack {
    fn kind(&self) -> &'static str {
        "ZStack"
    }

    fn layout<'widget, 'parent>(
        &mut self,
        mut ctx: LayoutCtx<'widget, 'parent>,
        nodes: NodeEval<'widget>,
    ) -> Result<Size> {
        if let Some(min_width) = self.min_width {
            ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        }
        if let Some(width) = self.width {
            ctx.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height {
            ctx.constraints.make_height_tight(height);
        }

        Layouts::new(Stacked, &mut ctx).layout(nodes)?.size()
    }

    fn position(&mut self, ctx: PositionCtx, nodes: &mut Nodes) {
        for widget in nodes.iter_mut() {
            widget.position(ctx.pos);
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>, nodes: &mut Nodes) {
        for child in nodes.iter_mut() {
            let ctx = ctx.sub_context(None);
            child.paint(ctx);
        }
    }
}

pub(crate) struct ZStackFactory;

impl WidgetFactory for ZStackFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        _: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let width = values.width();
        let height = values.height();
        let mut widget = ZStack::new(width, height);
        widget.min_width = values.min_width();
        widget.min_height = values.min_height();
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::{template, template_text};
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn border_title() {
        let zstack = ZStack::new(None, None);
        let body = [
            template("border", (), [template("expand", (), [])]),
            template("position", [("left", 2)], [template_text(" [title] ")]),
        ];

        test_widget(
            zstack,
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║┌─ [title] ────────┐║
            ║│                  │║
            ║│                  │║
            ║│                  │║
            ║└──────────────────┘║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn place_on_top() {
        let zstack = ZStack::new(None, None);
        let body = [
            template_text("000"),
            template_text("11"),
            template_text("2"),
        ];

        test_widget(
            zstack,
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║210                 ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }
}
