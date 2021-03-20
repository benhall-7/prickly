use super::{Component, Input, InputResponse};
use regex::Regex;

#[derive(Debug)]
pub struct Filter {
    pub input: Input,
    regex: Regex,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
            input: Input::default(),
            regex: Regex::new("").unwrap(),
        }
    }
}

pub enum FilterResponse {
    None,
    Exit,
}

impl Component for Filter {
    type Response = FilterResponse;

    fn handle_event(&mut self, event: super::Event) -> Self::Response {
        todo!()
    }

    fn draw(&mut self, rect: tui::layout::Rect, buffer: &mut tui::buffer::Buffer) {
        todo!()
    }
}
