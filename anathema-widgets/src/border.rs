use anathema_render::{Size, Style};
use unicode_width::UnicodeWidthChar;

use super::{LocalPos, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::lookup::WidgetFactory;
use crate::values::{
    Layout, SomeThing, BORDER_EDGE_BOTTOM, BORDER_EDGE_BOTTOM_LEFT, BORDER_EDGE_BOTTOM_RIGHT,
    BORDER_EDGE_LEFT, BORDER_EDGE_RIGHT, BORDER_EDGE_TOP, BORDER_EDGE_TOP_LEFT,
    BORDER_EDGE_TOP_RIGHT, DEFAULT_SLIM_EDGES, DEFAULT_THICK_EDGES,
};
use crate::{AnyWidget, Attributes, BorderStyle, DataCtx, Sides, TextPath};

/// Draw a border around an element.
///
/// The border will size it self around the child if it has one.
///
/// If a width and / or a height is provided then the border will produce tight constraints
/// for the child.
///
/// If a border has no size (width and height) and no child then nothing will be rendered.
///
/// To render a border with no child provide a width and a height.
#[derive(Debug)]
pub struct Border {
    /// Which sides of the border should be rendered.
    pub sides: Sides,
    /// All the characters for the border, starting from the top left moving clockwise.
    /// This means the top-left corner is `edges[0]`, the top if `edges[1]` and the top right is
    /// `edges[2]` etc.
    pub edges: [char; 8],
    /// The width of the border. This will make the constraints tight for the width.
    pub width: Option<usize>,
    /// The height of the border. This will make the constraints tight for the height.
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
    /// The style of the border.
    pub style: Style,
}

impl Border {
    /// The name of the element
    pub const KIND: &'static str = "Border";

    /// Create a new instance of a border
    ///
    ///```
    /// use anathema_widgets::{Border, BorderStyle, Sides};
    /// let border_style = BorderStyle::Thin;
    /// let border = Border::new(&border_style, Sides::ALL, None, None);
    /// ```
    pub fn new(
        style: &BorderStyle,
        sides: Sides,
        width: impl Into<Option<usize>>,
        height: impl Into<Option<usize>>,
    ) -> Self {
        let width = width.into();
        let height = height.into();

        let edges = style.edges();
        Self {
            sides,
            edges,
            width,
            height,
            min_width: None,
            min_height: None,
            style: Style::new(),
        }
    }

    /// Create a "thin" border with an optional width and height
    pub fn thin(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(&BorderStyle::Thin, Sides::ALL, width, height)
    }

    /// Create a "thick" border with an optional width and height
    pub fn thick(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(&BorderStyle::Thick, Sides::ALL, width, height)
    }

    fn border_size(&self) -> Size {
        // Get the size of the border (thickness).
        // This is NOT including the child.
        let mut border_width = 0;
        if self.sides.contains(Sides::LEFT) {
            let mut width = self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        if self.sides.contains(Sides::RIGHT) {
            let mut width = self.edges[BORDER_EDGE_RIGHT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        // Set the height of the border it self (thickness)
        let mut border_height = 0;
        if self.sides.contains(Sides::TOP) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        if self.sides.contains(Sides::BOTTOM) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        Size::new(border_width, border_height)
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::new(&BorderStyle::Thin, Sides::ALL, None, None)
    }
}

impl Widget for Border {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'tpl, 'parent>(&mut self, mut layout: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        // If there is a min width / height, make sure the minimum constraints
        // are matching these
        if let Some(min_width) = self.min_width {
            layout.constraints.min_width = layout.constraints.min_width.max(min_width);
        }

        if let Some(min_height) = self.min_height {
            layout.constraints.min_height = layout.constraints.min_height.max(min_height);
        }

        // If there is a width / height then make the constraints tight
        // around the size. This will modify the size to fit within the
        // constraints first.
        if let Some(width) = self.width {
            layout.constraints.make_width_tight(width);
        }

        if let Some(height) = self.height {
            layout.constraints.make_height_tight(height);
        }

        let border_size = self.border_size();

        let mut values = layout.values.next();
        let mut gen = Generator::new(layout.templates, layout.lookup, &mut values);

        let size = match gen.next(&mut values).transpose()? {
            Some(mut widget) => {
                let mut constraints = layout.padded_constraints();

                // Shrink the constraint for the child to fit inside the border
                constraints.max_width = constraints.max_width.saturating_sub(border_size.width);
                if constraints.max_width == 0 {
                    return Ok(Size::ZERO);
                }

                if constraints.min_width > constraints.max_width {
                    constraints.min_width = constraints.max_width;
                }

                constraints.max_height = constraints.max_height.saturating_sub(border_size.height);
                if constraints.max_height == 0 {
                    return Ok(Size::ZERO);
                }
                if constraints.min_height > constraints.max_height {
                    constraints.min_height = constraints.max_height;
                }

                let values = values.next();
                let mut size = widget.layout(constraints, &values, layout.lookup)?
                    + border_size
                    + layout.padding_size();

                layout.children.push(widget);

                if let Some(min_width) = self.min_width {
                    size.width = size.width.max(min_width);
                }

                if let Some(min_height) = self.min_height {
                    size.height = size.height.max(min_height);
                }

                if layout.constraints.is_width_tight() {
                    size.width = layout.constraints.max_width;
                }
                if layout.constraints.is_height_tight() {
                    size.height = layout.constraints.max_height;
                }

                Size {
                    width: size.width.min(layout.constraints.max_width),
                    height: size.height.min(layout.constraints.max_height),
                }
            }
            None => {
                let mut size =
                    Size::new(layout.constraints.min_width, layout.constraints.min_height);
                if layout.constraints.is_width_tight() {
                    size.width = layout.constraints.max_width;
                }
                if layout.constraints.is_height_tight() {
                    size.height = layout.constraints.max_height;
                }
                size
            }
        };

        Ok(size)
    }

    fn position<'gen, 'ctx>(
        &mut self,
        mut ctx: PositionCtx,
        children: &mut [WidgetContainer<'gen>],
    ) {
        let child = match children.first_mut() {
            Some(child) => child,
            None => return,
        };

        if self.sides.contains(Sides::TOP) {
            ctx.pos.y += 1;
        }

        if self.sides.contains(Sides::LEFT) {
            ctx.pos.x += self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0) as i32;
        }

        child.position(ctx.padded_position());
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        // Draw the child
        let border_size = self.border_size();
        if let Some(child) = children.first_mut() {
            let mut clipping_region = ctx.create_region();
            clipping_region.to.x -= border_size.width as i32 / 2;
            clipping_region.to.y -= border_size.height as i32 / 2;

            let child_ctx = ctx.sub_context(Some(&clipping_region));

            child.paint(child_ctx);
        }

        // Draw the border
        let width = ctx.local_size.width;
        let height = ctx.local_size.height;

        // Only draw corners if there are connecting sides:
        // e.g Sides::Left | Sides::Top
        //
        // Don't draw corners if there are no connecting sides:
        // e.g Sides::Top | Sides::Bottom

        // Top left
        let pos = LocalPos::ZERO;
        if self.sides.contains(Sides::LEFT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
        }

        // Top right
        let pos = LocalPos::new(width.saturating_sub(1), 0);
        if self.sides.contains(Sides::RIGHT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
        }

        // Bottom left
        let pos = LocalPos::new(0, height.saturating_sub(1));
        if self.sides.contains(Sides::LEFT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
        }

        // Bottom right
        let pos = LocalPos::new(width.saturating_sub(1), height.saturating_sub(1));
        if self.sides.contains(Sides::RIGHT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
        }

        // Top
        if self.sides.contains(Sides::TOP) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, 0);
                ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
            }
        }

        // Bottom
        if self.sides.contains(Sides::BOTTOM) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, height.saturating_sub(1));
                ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
            }
        }

        // Left
        if self.sides.contains(Sides::LEFT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(0, i);
                ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
            }
        }

        // Right
        if self.sides.contains(Sides::RIGHT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(width.saturating_sub(1), i);
                ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
            }
        }
    }

    // fn update(&mut self, ctx: UpdateCtx) {
    //     ctx.attributes.update_style(&mut self.style);
    //     for (k, _) in &ctx.attributes {
    //         match k.as_str() {
    //             fields::WIDTH => self.width = ctx.attributes.width(),
    //             fields::HEIGHT => self.height = ctx.attributes.height(),
    //             fields::BORDER_STYLE => self.edges = ctx.attributes.border_style().edges(),
    //             fields::SIDES => self.sides = ctx.attributes.sides(),
    //             _ => {}
    //         }
    //     }
    // }
}

pub(crate) struct BorderFactory;

impl WidgetFactory for BorderFactory {
    fn make(
        &self,
        values: SomeThing<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let border_style = values.border_style();
        let sides = values.sides();
        let width = values.width();
        let height = values.height();

        let mut widget = Border::new(&*border_style, sides, width, height);
        widget.min_width = values.min_width();
        widget.min_height = values.min_height();
        widget.style = values.style();
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::test_widget;
    use crate::Lookup;

    fn test_border(sides: Sides, expected: &str) {
        let lookup = Lookup::default();
        let border = Border::new(&BorderStyle::Thin, sides, None, None);
        let border = WidgetContainer::new(Box::new(border), &[]);
        test_widget(border, &lookup, expected);
    }

    #[test]
    fn border() {
        test_border(
            Sides::ALL,
            r#"
            ┌───┐
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn border_top() {
        test_border(
            Sides::TOP,
            r#"
            ─────
            "#,
        );
    }

    #[test]
    fn border_top_bottom() {
        test_border(
            Sides::TOP | Sides::BOTTOM,
            r#"
            ─────
            ─────
            "#,
        );
    }

    #[test]
    fn border_left() {
        test_border(
            Sides::LEFT,
            r#"
            │
            │
            "#,
        );
    }

    #[test]
    fn border_right() {
        test_border(
            Sides::RIGHT,
            r#"
                │
                │
            "#,
        );
    }

    #[test]
    fn border_bottom_right() {
        test_border(
            Sides::TOP | Sides::LEFT,
            r#"
            ┌───
            │   
            │   
            "#,
        );
    }

    // #[test]
    // fn style_changes_via_attributes() {
    //     let mut border = Border::thick(10, 3);
    //     border.update(Attributes::new("italic", true));
    //     assert!(border.to_mut::<Border>().style.attributes.contains(anathema_render::Attributes::ITALIC));
    // }

    //     #[test]
    //     fn min_width_height() {
    //         let mut border = Border::thick(10, 3);
    //         border.min_width = Some(10);
    //         border.min_height = Some(3);
    //         let lookup = WidgetLookup::default();
    //         let ctx = Context::new();
    //         let mut border = WidgetContainer::new(Box::new(border), &[]);
    //         border.layout(Constraints::unbounded(), &ctx, &lookup);
    //         let actual = border.size();
    //         assert_eq!(Size::new(10, 3), actual);
    //     }
}