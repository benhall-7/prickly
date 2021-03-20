use super::{Component, Event};
use crossterm::event::KeyCode;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::{Paragraph, Widget};

#[derive(Debug, Default)]
// todo: add cursor
pub struct Input {
    pub value: String,
    text_style: Style,
}

impl Input {
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_style = style;
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

    fn draw(&mut self, rect: Rect, buffer: &mut Buffer) {
        
    }
}