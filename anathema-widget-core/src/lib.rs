use anathema_render::{ScreenPos, Style};
use anathema_values::{Context, NodeId};
use generator::Attributes;

pub mod contexts;
pub mod error;
mod factory;
pub mod generator;
pub mod layout;
mod values;
mod widget;

// #[cfg(feature = "testing")]
// pub mod testing;

pub use generator::Nodes;

pub use crate::factory::{Factory, WidgetFactory};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region};
pub use crate::values::{Color, Display};
pub use crate::widget::{AnyWidget, Widget, WidgetContainer};

pub fn style<T>(context: &Context<'_, '_>, attributes: &Attributes, node_id: &NodeId) -> Style {
    panic!()
    // let mut style = Style::new();

    // style.fg = context.attribute("foreground", node_id.into(), attributes);
    // style.set_bold(context.primitive("bold", node_id.into(), attributes).unwrap_or(false));
    // style.set_italic(context.primitive("italic", node_id.into(), attributes).unwrap_or(false));
    // style.set_dim(context.primitive("dim", node_id.into(), attributes).unwrap_or(false));
    // style.set_underlined(context.primitive("underline", node_id.into(), attributes).unwrap_or(false));
    // style.set_overlined(context.primitive("overline", node_id.into(), attributes).unwrap_or(false));
    // style.set_crossed_out(context.primitive("crossed-out", node_id.into(), attributes).unwrap_or(false));
    // style.set_inverse(context.primitive("inverse", node_id.into(), attributes).unwrap_or(false));

    // style
}
