use anathema_values::{Context, DynValue, Num, Owned, Resolver, Value, ValueRef};

/// Represents the padding of a widget.
/// Padding is not applicable to `text:` widgets.
/// ```ignore
/// # use anathema_widgets::{Text, Border, BorderStyle, Sides, NodeId, Widget, Padding};
/// let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, 8, 5)
///     .into_container(NodeId::anon());
///
/// // Set the padding to 2 on all sides
/// border.padding = Padding::new(2);
///
/// let text = Text::with_text("hi")
///     .into_container(NodeId::anon());
/// border.add_child(text);
/// ```
/// would output
/// ```text
/// ┌──────┐
/// │      │
/// │  hi  │
/// │      │
/// └──────┘
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Padding {
    /// Top padding
    pub top: usize,
    /// Right padding
    pub right: usize,
    /// Bottom padding
    pub bottom: usize,
    /// Left padding
    pub left: usize,
}

impl Padding {
    /// Zero padding
    pub const ZERO: Padding = Self::new(0);

    /// Create a new instance padding
    pub const fn new(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    pub fn from_iter(mut iter: impl Iterator<Item = usize>) -> Self {
        let Some(n) = iter.next() else {
            return Self::ZERO;
        };
        let mut padding = Self::new(n);

        let Some(right) = iter.next() else {
            return padding;
        };
        padding.right = right;

        let Some(bottom) = iter.next() else {
            padding.bottom = padding.top;
            padding.left = padding.right;
            return padding;
        };

        padding.bottom = bottom;

        let Some(left) = iter.next() else {
            padding.left = padding.right;
            return padding;
        };

        padding.left = left;

        padding
    }

    /// Return the current padding and set the padding to zero
    pub fn take(&mut self) -> Self {
        let mut padding = Padding::ZERO;
        std::mem::swap(&mut padding, self);
        padding
    }
}

impl DynValue for Padding {
    fn init_value(
        context: &Context<'_, '_>,
        node_id: Option<&anathema_values::NodeId>,
        expr: &anathema_values::ValueExpr,
    ) -> Value<Self>
    where
        Self: Sized,
    {
        let mut resolver = Resolver::new(context, node_id);
        let Some(value) = expr.eval(&mut resolver) else {
            return Value::Empty;
        };

        let inner = match value {
            ValueRef::Owned(Owned::Num(n)) => Some(Self::new(n.to_usize())),
            ValueRef::Expressions(values) => {
                values
                    .iter()
                    .filter_map(|expr| expr.eval(&mut resolver))
                    .map(|val| match val {
                        ValueRef::Owned(Owned::Num(n)) => n.to_usize(),
                        _ => 0,
                    })
                    .collect::<Vec<_>>();
                panic!()
            }
            _ => None,
        };

        match resolver.is_deferred() {
            true => Value::Dyn {
                inner,
                expr: expr.clone(),
            },
            false => match inner {
                Some(val) => Value::Static(val),
                None => Value::Empty,
            },
        }
    }

    fn resolve(
        value: &mut Value<Self>,
        context: &Context<'_, '_>,
        node_id: Option<&anathema_values::NodeId>,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolve_padding() {
    }
}