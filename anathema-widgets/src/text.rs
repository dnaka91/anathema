use anathema_render::{Size, Style};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::node::{NodeEval, Nodes};
use anathema_widget_core::{
    AnyWidget, LocalPos, TextPath, Value, ValuesAttributes, Widget, WidgetContainer,
    WidgetFactory,
};
use unicode_width::UnicodeWidthStr;

use crate::layout::text::{Entry, Range, TextAlignment, TextLayout, Wrap};

// -----------------------------------------------------------------------------
//     - Text -
// -----------------------------------------------------------------------------
/// Text widget
/// ```ignore
/// Attributes:
/// * background
/// * foreground
/// * text-align
/// * trimstart
/// * trimend
/// * collapse_spaces <- rename this to something less stupid
/// * wordwrap
/// ```
///
/// Note: Spans, unlike other widgets, does not require a widget id
///
/// A `Text` widget will be as wide as its text.
#[derive(Debug, PartialEq, Eq)]
pub struct Text {
    /// Word wrapping
    pub word_wrap: Wrap,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: TextAlignment,
    /// Text
    pub text: String,
    /// Text style
    pub style: Style,

    layout: TextLayout,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            style: Style::new(),
            word_wrap: Wrap::Normal,
            text_alignment: TextAlignment::Left,
            layout: TextLayout::ZERO,
        }
    }
}

impl Text {
    pub const KIND: &'static str = "Text";

    /// Create a new instance of a `Text` widget
    pub fn new(text: impl Into<String>) -> Self {
        let mut inst = Self::default();
        inst.set_text(text);
        inst
    }

    /// Update the text of the first `TextSpan` with new text, without changing attributes or `NodeId`.
    /// If there are no `TextSpan` one will be created and inserted.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    fn text_and_style<'a>(
        &'a self,
        entry: &Entry,
        children: &'a [&mut WidgetContainer],
    ) -> (&'a str, Style) {
        let widget_index = match entry {
            Entry::All { index, .. } => index,
            Entry::Range(Range { slice, .. }) => slice,
            Entry::Newline => {
                unreachable!("when painting the lines, the NewLine is always skipped")
            }
        };

        let (text, style) = if *widget_index == 0 {
            (self.text.as_str(), self.style)
        } else {
            let span = children[widget_index - 1].to_ref::<TextSpan>();
            (span.text.as_str(), span.style)
        };

        if let Entry::Range(Range { start, end, .. }) = entry {
            (&text[*start..*end], style)
        } else {
            (text, style)
        }
    }

    fn paint_line(
        &self,
        range: &mut std::ops::Range<usize>,
        children: &[&mut WidgetContainer],
        y: usize,
        ctx: &mut PaintCtx<'_, WithSize>,
    ) {
        let mut pos = LocalPos::new(0, y);

        // Calculate the line width.
        // This is only relevant if `Align` is not `Left`.
        let mut line_width = 0;
        for entry in &self.layout.lines.inner[range.clone()] {
            let (text, _) = self.text_and_style(entry, children);
            line_width += text.width();
        }

        let max_width = self.layout.size().width;
        match self.text_alignment {
            TextAlignment::Left => {}
            TextAlignment::Centre => pos.x = max_width / 2 - line_width / 2,
            TextAlignment::Right => pos.x = max_width - line_width,
        }

        // ... then print the chars
        for entry_index in range.clone() {
            let entry = &self.layout.lines.inner[entry_index];
            let (text, style) = self.text_and_style(entry, children);
            let Some(new_pos) = ctx.print(text, style, pos) else {
                continue;
            };
            pos = new_pos;
        }

        range.end += 1;
        range.start = range.end;
    }
}

impl Widget for Text {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'widget, 'parent>(
        &mut self,
        ctx: LayoutCtx<'widget, 'parent>,
        mut nodes: NodeEval<'widget>,
    ) -> Result<Size> {
        let max_size = Size::new(ctx.constraints.max_width, ctx.constraints.max_height);
        self.layout.set_max_size(max_size);
        self.layout.set_wrap(self.word_wrap);

        self.layout.process(self.text.as_str());

        let mut values = ctx.values.next();
        while let Some(mut span) = nodes.next(&mut values).transpose()? {
            // Ignore any widget that isn't a span
            if span.kind() != TextSpan::KIND {
                continue;
            }

            let Some(inner_span) = span.try_to_mut::<TextSpan>() else {
                nodes.pop();
                continue;
            };
            self.layout.process(inner_span.text.as_str());
        }

        Ok(self.layout.size())
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>, nodes: &mut Nodes) {
        let mut y = 0;
        let mut range = 0..0;
        let children = nodes.iter_mut().collect::<Vec<_>>();
        for entry in &self.layout.lines.inner {
            match entry {
                Entry::All { .. } | Entry::Range(_) => range.end += 1,
                Entry::Newline => {
                    self.paint_line(&mut range, children.as_slice(), y, &mut ctx);
                    y += 1;
                    continue;
                }
            };
        }

        self.paint_line(&mut range, children.as_slice(), y, &mut ctx);
    }

    fn position(&mut self, _: PositionCtx, _: &mut Nodes) {
        // NOTE: there is no need to position text as the text
        // is printed from the context position
    }
}

/// Represents a chunk of text with its own style
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    /// The text
    pub text: String,
    /// Style for the text
    pub style: Style,
}

impl TextSpan {
    const KIND: &'static str = "TextSpan";

    pub fn new() -> Self {
        Self {
            text: String::new(),
            style: Style::new(),
        }
    }
}

impl Widget for TextSpan {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, _ctx: LayoutCtx<'_, '_>, _: NodeEval<'_>) -> Result<Size> {
        panic!("layout should never be called directly on a span");
    }

    fn position(&mut self, _: PositionCtx, _: &mut Nodes) {
        // NOTE: there is no need to position text as the text is printed from the context position
        panic!("don't invoke position on the span directly.");
    }

    fn paint(&mut self, _: PaintCtx<'_, WithSize>, _: &mut Nodes) {
        panic!("don't invoke paint on the span directly.");
    }
}

pub(crate) struct TextFactory;

impl WidgetFactory for TextFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let mut widget = Text::default();

        widget.word_wrap = values
            .get_attrib("wrap")
            .and_then(Value::to_str)
            .map(From::from)
            .unwrap_or(Wrap::Normal);

        widget.text_alignment = values
            .get_attrib("text-align")
            .and_then(Value::to_str)
            .map(From::from)
            .unwrap_or(TextAlignment::Left);

        widget.style = values.style();
        if let Some(text) = text {
            widget.text = values.text_to_string(text).to_string();
        }
        Ok(Box::new(widget))
    }
}

pub(crate) struct SpanFactory;

impl WidgetFactory for SpanFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let mut widget = TextSpan::new();
        if let Some(text) = text {
            widget.text = values.text_to_string(text).to_string();
        }
        widget.style = values.style();

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::template_span;
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn word_wrap_excessive_space() {
        test_widget(
            Text::new("hello      how are     you"),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello      how  ║
            ║are     you     ║
            ║                ║
            ║                ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn word_wrap() {
        test_widget(
            Text::new("hello how are you"),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are   ║
            ║you             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn no_word_wrap() {
        let mut text = Text::new("hello how are you");
        text.word_wrap = Wrap::Overflow;
        test_widget(
            text,
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are yo║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn break_word_wrap() {
        let mut text = Text::new("hellohowareyoudoing");
        text.word_wrap = Wrap::WordBreak;
        test_widget(
            text,
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hellohowareyoudo║
            ║ing             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let body = [
            template_span("two"),
            template_span(" averylongword"),
            template_span(" bunny "),
        ];
        test_widget(
            Text::new("one"),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═════╗
            ║onetwo             ║
            ║averylongword bunny║
            ║                   ║
            ╚═══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn right_alignment() {
        let mut text = Text::new("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Right;
        test_widget(
            text,
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║             a one║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn centre_alignment() {
        let mut text = Text::new("a one xxxxxxxxxxxxxxxxxx");
        text.text_alignment = TextAlignment::Centre;
        test_widget(
            text,
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║       a one      ║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }
}
