use super::{Component, Input, InputResponse};
use regex::{Regex, Error};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Style, Color};
use tui::widgets::{Block, Borders, Widget};

#[derive(Debug)]
pub struct Filter {
    pub input: Input,
    last_input: String,
    regex: Result<Regex, Error>
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
}

pub enum FilterResponse {
    None,
    Exit,
}

impl Component for Filter {
    type Response = FilterResponse;

    fn handle_event(&mut self, event: super::Event) -> Self::Response {
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
            InputResponse::Edited | InputResponse::None => FilterResponse::None,
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::TOP).title("regex filter");
        let inner = block.inner(rect);
        block.render(rect, buf);
        self.input.draw(inner, buf);
    }
}
