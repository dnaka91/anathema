use anathema_values::{Context, NodeId, Scope, State};

use super::{LoopNode, Node, NodeKind};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::Expression;
use crate::{Nodes, WidgetContainer};

pub trait NodeVisitor {
    type Output;

    fn visit(&mut self, node: &mut Node<'_>) -> Self::Output;
    fn visit_single(
        &mut self,
        widget_container: &mut WidgetContainer,
        nodes: &mut Nodes<'_>,
    ) -> Self::Output;
    fn visit_loop(&mut self, loop_node: &mut LoopNode<'_>) -> Self::Output;
    fn visit_control_flow(&mut self) -> Self::Output;
}
