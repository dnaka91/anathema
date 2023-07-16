use std::collections::HashMap;
use std::time::Duration;

use anathema_render::Size;
use anathema_widget_core::contexts::DataCtx;
use anathema_widget_core::{Number, Value, Map};

const META: &'static str = "_meta";
const TIMINGS: &'static str = "timings";
const SIZE: &'static str = "size";
const FOCUS: &'static str = "focus";
const COUNT: &'static str = "count";

#[derive(Debug)]
pub(super) struct Meta {
    pub(super) size: Size,
    pub(super) timings: Timings,
    pub(super) focus: bool,
}

impl Meta {
    pub(super) fn new(size: Size) -> Self {
        Self {
            size,
            timings: Timings::default(),
            focus: true,
        }
    }

    fn size_map(&self, hm: &mut Map) {
        hm.insert("width", self.size.width as u64);
        hm.insert("height", self.size.height as u64);
    }

    fn timings_map(&self, hm: &mut Map) {
        hm.insert(
            "layout".to_string(),
            Value::String(format!("{:?}", self.timings.layout)),
        );
        hm.insert(
            "position".to_string(),
            Value::String(format!("{:?}", self.timings.position)),
        );
        hm.insert(
            "paint".to_string(),
            Value::String(format!("{:?}", self.timings.paint)),
        );
        hm.insert(
            "render".to_string(),
            Value::String(format!("{:?}", self.timings.render)),
        );
        hm.insert(
            "total".to_string(),
            Value::String(format!("{:?}", self.timings.total)),
        );
    }

    pub(super) fn update(&mut self, ctx: &mut DataCtx, node_len: usize) {
        match ctx.get_mut::<Map>(META) {
            None => {
                let mut metamap = Map::empty();

                let mut size = Map::empty();
                self.size_map(&mut size);

                let mut timings = Map::empty();
                self.timings_map(&mut timings);

                metamap.insert(SIZE, size);
                metamap.insert(TIMINGS, timings);
                metamap.insert(FOCUS, self.focus);
                metamap.insert(COUNT, node_len);
                ctx.insert(META, metamap);
            }
            Some(meta) => {
                match meta.get_mut::<bool, _>(FOCUS) {
                    Some(focus) => *focus = self.focus.into(),
                    None => {
                        meta.insert(FOCUS.to_string(), self.focus);
                    }
                };

                match meta.get_mut::<u64, _>(COUNT) {
                    Some(count) => *count = node_len as u64,
                    None => {
                        meta.insert(COUNT, node_len);
                    }
                };

                match meta.get_mut(SIZE) {
                    Some(Value::Map(size)) => self.size_map(size),
                    _ => {
                        let mut size = Map::empty();
                        self.size_map(&mut size);
                        meta.insert(SIZE, size);
                    }
                }

                match meta.get_mut(TIMINGS) {
                    Some(Value::Map(timings)) => self.timings_map(timings),
                    _ => {
                        let mut timings = Map::empty();
                        self.timings_map(&mut timings);
                        meta.insert(TIMINGS, timings);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Timings {
    pub(super) layout: Duration,
    pub(super) position: Duration,
    pub(super) paint: Duration,
    pub(super) render: Duration,
    pub(super) total: Duration,
}
