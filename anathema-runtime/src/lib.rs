use std::io::{stdout, Stdout};
use std::sync::Arc;
use std::time::Instant;

use anathema_render::{size, Screen, Size};
use anathema_widget_core::contexts::{DataCtx, LayoutCtx, PaintCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Constraints, Padding};
use anathema_widget_core::node::{NodeId, Nodes};
use anathema_widget_core::template::Template;
use anathema_widget_core::views::View;
use anathema_widget_core::{Pos, Values};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use events::Event;

use self::meta::Meta;
use crate::events::{EventProvider, Events};

pub mod events;
mod meta;

pub struct Runtime<E, ER> {
    pub enable_meta: bool,
    meta: Meta,
    nodes: Nodes,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    ctx: DataCtx,
    events: E,
    event_receiver: ER,
}

impl<E, ER> Drop for Runtime<E, ER> {
    fn drop(&mut self) {
        let _ = Screen::show_cursor(&mut self.output);
        let _ = disable_raw_mode();
    }
}

impl<E, ER> Runtime<E, ER>
where
    E: Events,
    ER: EventProvider,
{
    pub fn new(
        templates: impl Into<Arc<[Template]>>,
        ctx: DataCtx,
        events: E,
        event_receiver: ER,
    ) -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        Screen::hide_cursor(&mut stdout)?;
        let screen = Screen::new(size);

        let inst = Self {
            output: stdout,
            meta: Meta::new(size),
            screen,
            constraints,
            nodes: Nodes::new(templates),
            ctx,
            events,
            event_receiver,
            enable_meta: false,
        };

        Ok(inst)
    }

    pub fn register_view(&mut self, name: impl Into<String>, view: impl View + 'static) {
        self.ctx.views.register(name.into(), view);
    }

    fn layout(&mut self) -> Result<()> {
        self.nodes.clear();
        let values = Values::new(&self.ctx);
        let root_id = NodeId::root();
        let layout_ctx = LayoutCtx::new(&root_id, &values, self.constraints, Padding::ZERO);
        let mut node_gen = self.nodes.gen(layout_ctx);
        let mut values = values.next();
        while let Some(widget) = node_gen.next(&mut values).transpose()? {
            widget.layout(&root_id, self.constraints, &values)?;
        }
        Ok(())
    }

    fn position(&mut self) {
        for widget in &mut self.nodes.iter_mut() {
            widget.position(Pos::ZERO);
        }
    }

    fn paint(&mut self) {
        let values = Values::new(&self.ctx);
        for widget in &mut self.nodes.iter_mut() {
            widget.paint(PaintCtx::new(&mut self.screen, None));
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.screen.clear_all(&mut self.output)?;

        'run: loop {
            while let Some(event) = self.event_receiver.next() {
                let event = self.events.event(event, &mut self.ctx, &mut self.nodes);
                match event {
                    Event::Resize(width, height) => {
                        let size = Size::from((width, height));
                        self.screen.erase();
                        self.screen.render(&mut self.output)?;
                        self.screen.resize(size);

                        self.constraints.max_width = size.width;
                        self.constraints.max_height = size.height;

                        self.meta.size = size;
                    }
                    Event::Blur => self.meta.focus = false,
                    Event::Focus => self.meta.focus = true,
                    Event::Quit => break 'run Ok(()),
                    _ => {}
                }
            }

            let total = Instant::now();
            self.layout()?;
            self.meta.timings.layout = total.elapsed();

            let now = Instant::now();
            self.position();
            self.meta.timings.position = now.elapsed();

            let now = Instant::now();
            self.paint();
            self.meta.timings.paint = now.elapsed();

            let now = Instant::now();
            self.screen.render(&mut self.output)?;
            self.meta.timings.render = now.elapsed();
            self.meta.timings.total = total.elapsed();
            self.screen.erase();

            if self.enable_meta {
                self.meta.update(&mut self.ctx, self.nodes.count());
            }
        }
    }
}
