use regex::{Error, Regex};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Widget};
use tui_components::components::*;
use tui_components::{tui, Component, Event};

#[derive(Debug)]
pub struct Filter {
    pub input: Input,
    last_input: String,
    regex: Result<Regex, Error>,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
            input: Input::default()
                .error_style(Style::default().bg(Color::Red))
                .editing_style(Style::default().bg(Color::Blue)),
            last_input: String::new(),
            regex: Regex::new(""),
        }
    }

    pub fn focus(&mut self, focus: bool) {
        self.input.focused = focus;
    }

    pub fn regex(&self) -> Option<&Regex> {
        self.regex.as_ref().ok()
    }
}

pub enum FilterResponse {
    None,
    Exit,
}

impl Component for Filter {
    type Response = FilterResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        match self.input.handle_event(event) {
            InputResponse::Submit => {
                self.regex = Regex::new(&self.input.value);
                self.last_input = self.input.value.clone();
                match &self.regex {
                    Ok(_) => {
                        self.input.error = None;
                        FilterResponse::Exit
                    }
                    Err(e) => {
                        self.input.error = Some(format!("{}", e));
                        FilterResponse::None
                    }
                }
            }
            InputResponse::Cancel => {
                self.input.value = self.last_input.clone();
                FilterResponse::Exit
            }
            InputResponse::Edited { .. } | InputResponse::None => FilterResponse::None,
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(if self.input.focused {
                Style::default().fg(Color::Blue)
            } else {
                Default::default()
            })
            .title(Span::styled(
                "REGEX FILTER",
                Style::default().fg(Color::White),
            ));
        let inner = block.inner(rect);
        block.render(rect, buf);
        self.input.draw(inner, buf);
    }
}
