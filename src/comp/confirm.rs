use super::{Component, Event};
use crate::rect_ext::RectExt;
use crossterm::event::KeyCode;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Widget};

#[derive(Debug)]
pub struct Confirm {
    choice: bool,
    title: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmResponse {
    Confirm(bool),
    Handled,
    None,
}

impl Confirm {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            choice: false,
            title: title.into(),
        }
    }
}

impl Component for Confirm {
    type Response = ConfirmResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Right => {
                    self.choice = false;
                    ConfirmResponse::Handled
                }
                KeyCode::Left => {
                    self.choice = true;
                    ConfirmResponse::Handled
                }
                KeyCode::Enter => ConfirmResponse::Confirm(self.choice),
                KeyCode::Esc => ConfirmResponse::Confirm(false),
                _ => ConfirmResponse::None,
            }
        } else {
            ConfirmResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(&self.title, Style::default().fg(Color::White)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let text_styles = if self.choice {
            [Style::default().fg(Color::Green), Style::default()]
        } else {
            [Style::default(), Style::default().fg(Color::Green)]
        };
        let inside_text = Spans::from(vec![
            Span::styled("Yes", text_styles[0]),
            Span::raw(" / "),
            Span::styled("No", text_styles[1]),
        ]);
        let max_width = (inside_text.width() + 2).max(self.title.len() + 2);
        let p = Paragraph::new(inside_text).alignment(Alignment::Center);

        let block_area = rect.centered(Rect {
            x: 0,
            y: 0,
            width: max_width as u16,
            height: 3,
        });
        let block_inner = block.inner(block_area);

        Widget::render(Clear, block_area, buf);
        Widget::render(block, block_area, buf);
        Widget::render(p, block_inner, buf);
    }
}
