use super::{Component, Event};
use crossterm::event::KeyCode;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};

#[derive(Debug, Default)]
// todo: add cursor
pub struct Input {
    pub value: String,
    pub error: Option<String>,
    pub focused: bool,
    text_style: Style,
    editing_style: Style,
    error_style: Style,
}

impl Input {
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_style = style;
        self
    }

    pub fn editing_style(mut self, style: Style) -> Self {
        self.editing_style = style;
        self
    }

    pub fn error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }
}

#[derive(Debug)]
pub enum InputResponse {
    None,
    Edited,
    Submit,
    Cancel,
}

impl Component for Input {
    type Response = InputResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.value.push(c);
                    InputResponse::Edited
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    InputResponse::Edited
                }
                KeyCode::Enter => InputResponse::Submit,
                KeyCode::Esc => InputResponse::Cancel,
                _ => InputResponse::None,
            }
        } else {
            InputResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let mut spans = vec![];
        if self.focused {
            spans.extend(vec![
                Span::raw("> "),
                Span::styled(&self.value, self.text_style),
            ]);
            if let Some(e) = &self.error {
                spans.extend(vec![
                    Span::raw(" "),
                    Span::styled(e, self.error_style)
                ]);
            }
        } else {
            if self.error.is_some() {
                spans.push(Span::styled(&self.value, self.error_style));
            } else {
                spans.push(Span::styled(&self.value, self.text_style));
            }
        }
        let p = Paragraph::new(Spans::from(spans));
        p.render(rect, buf);
    }
}

