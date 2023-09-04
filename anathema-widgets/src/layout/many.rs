use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::{Axis, Constraints, Direction, Layout};
use anathema_widget_core::{Nodes, WidgetContainer};

use super::{expand, spacers};
use crate::{Expand, Spacer};

struct SizeMod {
    inner: Size,
    max_size: Size,
    axis: Axis,
}

impl SizeMod {
    const fn new(max_size: Size, axis: Axis) -> Self {
        Self {
            inner: Size::ZERO,
            max_size,
            axis,
        }
    }

    fn apply(&mut self, size: Size) {
        match self.axis {
            Axis::Vertical => {
                self.inner.width = self.inner.width.max(size.width);
                self.inner.height = (self.inner.height + size.height).min(self.max_size.height);
            }
            Axis::Horizontal => {
                self.inner.height = self.inner.height.max(size.height);
                self.inner.width = (self.inner.width + size.width).min(self.max_size.width);
            }
        }
    }

    fn no_space_left(&self) -> bool {
        match self.axis {
            Axis::Horizontal => self.inner.width >= self.max_size.width,
            Axis::Vertical => self.inner.height >= self.max_size.height,
        }
    }

    fn to_constraints(&self) -> Constraints {
        match self.axis {
            Axis::Horizontal => {
                Constraints::new(self.max_size.width - self.inner.width, self.max_size.height)
            }
            Axis::Vertical => Constraints::new(
                self.max_size.width,
                self.max_size.height - self.inner.height,
            ),
        }
    }
}

#[derive(Debug)]
struct Offset {
    axis: Axis,
    inner: i32,
    enabled: bool,
}

impl Offset {
    fn skip(&mut self, size: &mut Size) -> bool {
        let height = size.height as i32;
        let width = size.width as i32;
        match self.axis {
            Axis::Vertical if self.enabled && self.inner >= height => {
                self.inner -= height;
                true
            }
            Axis::Vertical if self.enabled => {
                self.enabled = false;
                size.height = (size.height as i32 - self.inner) as usize;
                false
            }
            Axis::Horizontal if self.enabled && self.inner >= width => {
                self.inner -= width;
                true
            }
            Axis::Horizontal if self.enabled => {
                self.enabled = false;
                size.width = (size.width as i32 - self.inner) as usize;
                false
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Many {
    pub direction: Direction,
    pub axis: Axis,
    offset: Offset,
    unconstrained: bool,
}

impl Many {
    pub fn new(direction: Direction, axis: Axis, offset: i32, unconstrained: bool) -> Self {
        Self {
            direction,
            axis,
            offset: Offset {
                axis,
                inner: offset,
                enabled: true,
            },
            unconstrained,
        }
    }

    pub fn offset(&self) -> i32 {
        self.offset.inner
    }
}

impl Layout for Many {
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut Nodes,
        data: Context<'_, '_>,
    ) -> Result<Size> {
        let max_constraints = layout.padded_constraints();

        let mut used_size = SizeMod::new(
            Size::new(max_constraints.max_width, max_constraints.max_height),
            self.axis,
        );

        if let Direction::Backward = self.direction {
            panic!("this has to be done at some point...");
            // gen.flip();
        }

        let mut size = Size::ZERO;
        children.for_each(
            data.state,
            data.scope,
            layout,
            |widget, children, context| {
                if [Spacer::KIND, Expand::KIND].contains(&widget.kind()) {
                    return Ok(Size::ZERO);
                }

                let widget_constraints = {
                    let mut constraints = max_constraints;
                    if self.unconstrained {
                        match self.axis {
                            Axis::Vertical => constraints.unbound_height(),
                            Axis::Horizontal => constraints.unbound_width(),
                        }
                    }
                    constraints
                };

                let mut widget_size = widget.layout(children, widget_constraints, context)?;

                if self.offset.skip(&mut widget_size) {
                    return Ok(Size::ZERO);
                }

                used_size.apply(widget_size);

                if used_size.no_space_left() {
                    return Err(Error::InsufficientSpaceAvailble);
                }

                Ok(widget_size)
            },
        );

        // Apply spacer and expand if the layout is constrained
        if !self.unconstrained {
            {
                let mut exp_ctx = *layout;
                exp_ctx.constraints = used_size.to_constraints();
                let data = Context::new(data.state, data.scope);
                let expanded_size = expand::layout(&mut exp_ctx, children, self.axis, data)?;
                used_size.apply(expanded_size);
            }

            let mut space_ctx = *layout;
            space_ctx.constraints = used_size.to_constraints();
            let data = Context::new(data.state, data.scope);
            let spacer_size = spacers::layout(&mut space_ctx, children, self.axis, data)?;
            used_size.apply(spacer_size);
        }

        match self.axis {
            Axis::Vertical => {
                size.width = size.width.max(used_size.inner.width);
                size.height = size
                    .height
                    .max(used_size.inner.height)
                    .max(max_constraints.min_height);
            }
            Axis::Horizontal => {
                size.height = size.height.max(used_size.inner.height);
                size.width = size
                    .width
                    .max(used_size.inner.width)
                    .max(max_constraints.min_width);
            }
        }

        Ok(size)
    }

    fn finalize(&mut self, nodes: &mut Nodes) -> Size {
        todo!()
    }
}
