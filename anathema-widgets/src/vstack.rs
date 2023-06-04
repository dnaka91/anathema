use anathema_render::Size;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::layout::vertical;
use crate::lookup::WidgetFactory;
use crate::template::Template;
use crate::values::SomeThing;
use crate::{AnyWidget, TextPath, Widget, PositionCtx, WidgetContainer, PaintCtx, WithSize};

/// A widget that lays out its children vertically.
/// ```text
/// ┌─┐
/// │1│
/// └─┘
/// ┌─┐
/// │2│
/// └─┘
/// ┌─┐
/// │3│
/// └─┘
/// ```
///
/// ```ignore
/// use anathema_widgets::{VStack, Text, Widget, NodeId};
/// let mut vstack = VStack::new(None, None);
/// vstack.children.push(Text::with_text("1").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("2").into_container(NodeId::anon()));
/// vstack.children.push(Text::with_text("3").into_container(NodeId::anon()));
/// ```
/// output:
/// ```text
/// 1
/// 2
/// 3
/// ```
#[derive(Debug)]
pub struct VStack {
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Option<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Option<usize>,
    /// The minimum width. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
}

impl VStack {
    /// Creates a new instance of a `VStack`
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            min_width: None,
            min_height: None,
        }
    }
}

impl Widget for VStack {
    fn kind(&self) -> &'static str {
        "VStack"
    }

    fn layout<'tpl, 'parent>(&mut self, mut layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        if let Some(width) = self.width {
            layout.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height {
            layout.constraints.make_height_tight(height);
        }
        if let Some(min_width) = self.min_width {
            layout.constraints.min_width = layout.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            layout.constraints.min_height = layout.constraints.min_height.max(min_height);
        }

        panic!()
        // vertical::layout()
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        panic!()
        // vertical::position(ctx, children)
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        for child in children {
            let ctx = ctx.sub_context(None);
            child.paint(ctx);
        }
    }

    // fn update(&mut self, ctx: UpdateCtx) {
    //     if let Some(width) = ctx.attributes.width() {
    //         self.width = Some(width);
    //     }
    //     if let Some(height) = ctx.attributes.height() {
    //         self.height = Some(height);
    //     }
    // }
}

pub(crate) struct VstackFactory;

impl WidgetFactory for VstackFactory {
    fn make(
        &self,
        values: SomeThing<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let width = values.width();
        let height = values.height();
        let mut widget = VStack::new(width, height);
        widget.min_width = values.min_width();
        widget.min_height = values.min_height();
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::testing::test_widget;
    // use crate::{Border, BorderStyle, Sides, Text};

    // fn test_vstack(col: impl Widget, expected: &str) {
    //     let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
    //     border.child = Some(col.into_container(NodeId::Auto(0)));
    //     test_widget(border, expected);
    // }

    // #[test]
    // fn only_vstack() {
    //     let mut vstack = VStack::new(None, None);
    //     vstack.add_child(Text::with_text("0").into_container(NodeId::Auto(0)));
    //     vstack.add_child(Text::with_text("1").into_container(NodeId::Auto(1)));
    //     vstack.add_child(Text::with_text("2").into_container(NodeId::Auto(2)));
    //     test_vstack(
    //         vstack,
    //         r#"
    //         ┌───────┐
    //         │0      │
    //         │1      │
    //         │2      │
    //         └───────┘
    //         "#,
    //     );
    // }

    // // #[test]
    // // fn fixed_height_stack() {
    // //     let mut vstack = VStack::new(None, 2);
    // //     vstack.add_child(Text::with_text("0").into_container(NodeId::Auto(0)));
    // //     vstack.add_child(Text::with_text("1").into_container(NodeId::Auto(1)));
    // //     vstack.add_child(Text::with_text("2").into_container(NodeId::Auto(2)));
    // //     test_vstack(
    // //         vstack,
    // //         r#"
    // //         ┌───────┐
    // //         │0      │
    // //         │1      │
    // //         │       │
    // //         └───────┘
    // //         "#,
    // //     );
    // // }
}