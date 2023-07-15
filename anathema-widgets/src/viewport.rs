use anathema_render::Size;
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Direction, Layouts};
use anathema_widget_core::node::{NodeEval, Nodes};
use anathema_widget_core::{
    AnyWidget, TextPath, ValuesAttributes, Widget, WidgetContainer, WidgetFactory,
};

use crate::layout::many::Many;

/// A viewport where the children can be rendered with an offset.
#[derive(Debug, PartialEq)]
pub struct Viewport {
    /// Line / cell offset
    pub offset: i32,
    /// Clamp the vertical space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_vertical: bool,
    /// Clamp the horizontal space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_horizontal: bool,
    /// Layout direction.
    /// `Direction::Forward` is the default, and keeps the scroll position on the first child.
    /// `Direction::Backward` keeps the scroll position on the last child.
    pub direction: Direction,
    /// Vertical or horizontal
    pub axis: Axis,
}

impl Viewport {
    const KIND: &'static str = "Viewport";

    /// Create a new instance of a [`Viewport`]
    pub fn new(direction: Direction, axis: Axis, offset: impl Into<Option<i32>>) -> Self {
        Self {
            offset: offset.into().unwrap_or(0),
            clamp_horizontal: true,
            clamp_vertical: true,
            direction,
            axis,
        }
    }
}

impl Widget for Viewport {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'widget, 'parent>(
        &mut self,
        mut ctx: LayoutCtx<'widget, 'parent>,
        nodes: NodeEval<'widget>,
    ) -> Result<Size> {
        let many = Many::new(self.direction, self.axis, self.offset, true);
        let mut layout = Layouts::new(many, &mut ctx);
        layout.layout(nodes)?;
        self.offset = layout.layout.offset();
        layout.size()
    }

    fn position(&mut self, mut ctx: PositionCtx, nodes: &mut Nodes) {
        let mut pos = ctx.pos;
        if let Direction::Backward = self.direction {
            match self.axis {
                Axis::Horizontal => pos.x += ctx.inner_size.width as i32,
                Axis::Vertical => pos.y += ctx.inner_size.height as i32,
            }
        }

        let offset = match self.direction {
            Direction::Forward => -self.offset,
            Direction::Backward => self.offset,
        };

        match self.axis {
            Axis::Horizontal => pos.x += offset,
            Axis::Vertical => pos.y += offset,
        }

        for widget in nodes.iter_mut() {
            if let Direction::Forward = self.direction {
                widget.position(pos);
            }

            match self.direction {
                Direction::Forward => match self.axis {
                    Axis::Horizontal => pos.x += widget.outer_size().width as i32,
                    Axis::Vertical => pos.y += widget.outer_size().height as i32,
                },
                Direction::Backward => match self.axis {
                    Axis::Horizontal => pos.x -= widget.outer_size().width as i32,
                    Axis::Vertical => pos.y -= widget.outer_size().height as i32,
                },
            }

            if let Direction::Backward = self.direction {
                widget.position(pos);
            }
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>, nodes: &mut Nodes) {
        let region = ctx.create_region();
        for child in nodes.iter_mut() {
            let ctx = ctx.sub_context(Some(&region));
            child.paint(ctx);
        }
    }
}

pub(crate) struct ViewportFactory;

impl WidgetFactory for ViewportFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        _: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let direction = values.direction().unwrap_or(Direction::Forward);
        let axis = values.axis().unwrap_or(Axis::Vertical);
        let offset = values.offset();
        let widget = Viewport::new(direction, axis, offset);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::{template, template_text, Template};
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    fn children(count: usize) -> Vec<Template> {
        (0..count)
            .map(|i| template("border", (), vec![template_text(i.to_string())]))
            .collect()
    }

    #[test]
    fn vertical_viewport() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Forward, Axis::Vertical, 0),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│0│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn horizontal_viewport() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Forward, Axis::Horizontal, 0),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐┌─┐┌─┐┌─┐┌─┐║
            ║│0││1││2││3││4│║
            ║└─┘└─┘└─┘└─┘└─┘║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn vertical_viewport_reversed() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Backward, Axis::Vertical, 0),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐            ║
            ║│8│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│9│            ║
            ║└─┘            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn horizontal_viewport_reversed() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Backward, Axis::Horizontal, 0),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┌─┐┌─┐┌─┐┌─┐┌─┐║
            ║│5││6││7││8││9│║
            ║└─┘└─┘└─┘└─┘└─┘║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn vertical_forward_offset() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Forward, Axis::Vertical, 2),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║└─┘            ║
            ║┌─┐            ║
            ║│1│            ║
            ║└─┘            ║
            ║┌─┐            ║
            ║│2│            ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn horizontal_forward_offset() {
        let body = children(10);
        test_widget(
            Viewport::new(Direction::Forward, Axis::Horizontal, 2),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║┐┌─┐┌─┐┌─┐┌─┐┌─║
            ║││1││2││3││4││5║
            ║┘└─┘└─┘└─┘└─┘└─║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
