use std::collections::HashMap;
use std::time::Duration;

use anathema_render::Size;
use anathema_widget_core::contexts::DataCtx;
use anathema_widget_core::{Number, Value};

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

    fn size_map(&self, hm: &mut HashMap<String, Value>) {
        hm.insert(
            "width".to_string(),
            Value::Number(Number::Unsigned(self.size.width as u64)),
        );
        hm.insert(
            "height".to_string(),
            Value::Number(Number::Unsigned(self.size.height as u64)),
        );
    }

    fn timings_map(&self, hm: &mut HashMap<String, Value>) {
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
        match ctx.get_mut::<HashMap<String, Value>>(META) {
            None => {
                let mut metamap = HashMap::new();

                let mut size = HashMap::new();
                self.size_map(&mut size);

                let mut timings = HashMap::new();
                self.timings_map(&mut timings);

                metamap.insert(SIZE.into(), size.into());
                metamap.insert(TIMINGS.to_string(), timings.into());
                metamap.insert(FOCUS.to_string(), self.focus.into());
                metamap.insert(COUNT.to_string(), node_len.into());
                ctx.insert(META, metamap);
            }
            Some(meta) => {
                match meta.get_mut(FOCUS) {
                    Some(focus) => *focus = self.focus.into(),
                    None => {
                        meta.insert(FOCUS.to_string(), self.focus.into());
                    }
                };

                match meta.get_mut(COUNT) {
                    Some(count) => *count = node_len.into(),
                    None => {
                        meta.insert(COUNT.to_string(), node_len.into());
                    }
                };

                match meta.get_mut(SIZE) {
                    Some(Value::Map(size)) => self.size_map(size),
                    _ => {
                        let mut size = HashMap::new();
                        self.size_map(&mut size);
                        meta.insert(SIZE.into(), size.into());
                    }
                }

                match meta.get_mut(TIMINGS) {
                    Some(Value::Map(timings)) => self.timings_map(timings),
                    _ => {
                        let mut timings = HashMap::new();
                        self.timings_map(&mut timings);
                        meta.insert(TIMINGS.into(), timings.into());
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
