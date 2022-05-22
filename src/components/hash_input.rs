use prc::hash40::{hash40, Hash40};
use tui_components::tui::text::{Span, Spans};
use tui_components::Spannable;
use tui_components::{
    crossterm::event::KeyCode,
    span_builder::SpanBuilder,
    tui::{
        style::{Color, Style},
        widgets::{Paragraph, Widget},
    },
    Component,
};

#[derive(Debug)]
pub struct HashInput {
    value: String,
    return_value: Hash40,
}

#[derive(Debug, Clone, Copy)]
pub enum HashStatus {
    LabelExists(Hash40),
    LabelNotExists(Hash40),
    LabelsPoisoned(Hash40),
    Hash(Hash40),
    HashInvalid,
}

impl HashInput {
    pub fn new(hash: Hash40) -> Self {
        Self {
            value: hash.to_string(),
            return_value: hash,
        }
    }

    pub fn status(&self) -> HashStatus {
        if self.value.starts_with("0x") {
            match Hash40::from_hex_str(&self.value) {
                Ok(hash) => HashStatus::Hash(hash),
                Err(_) => HashStatus::HashInvalid,
            }
        } else {
            let label_arc = Hash40::label_map();
            let lock = label_arc.lock();
            if let Ok(labels) = lock {
                match labels.hash_of(&self.value) {
                    Some(hash) => HashStatus::LabelExists(hash),
                    None => HashStatus::LabelNotExists(hash40(&self.value)),
                }
            } else {
                HashStatus::LabelsPoisoned(hash40(&self.value))
            }
        }
    }

    pub fn value(&self) -> Hash40 {
        self.return_value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HashInputResponse {
    None,
    Handled,
    Submit,
    Cancel,
}

impl Component for HashInput {
    type Response = HashInputResponse;
    type DrawResponse = ();

    fn handle_event(&mut self, event: tui_components::Event) -> Self::Response {
        if let tui_components::Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.value.push(c);
                    HashInputResponse::Handled
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    HashInputResponse::Handled
                }
                KeyCode::Enter => {
                    let status = self.status();
                    match status {
                        HashStatus::Hash(hash)
                        | HashStatus::LabelExists(hash)
                        | HashStatus::LabelNotExists(hash)
                        | HashStatus::LabelsPoisoned(hash) => {
                            self.return_value = hash;
                            HashInputResponse::Submit
                        }
                        HashStatus::HashInvalid => HashInputResponse::None,
                    }
                }
                KeyCode::Esc => HashInputResponse::Cancel,
                _ => HashInputResponse::None,
            }
        } else {
            HashInputResponse::None
        }
    }

    fn draw(
        &mut self,
        rect: tui_components::tui::layout::Rect,
        buffer: &mut tui_components::tui::buffer::Buffer,
    ) -> Self::DrawResponse {
        let text = Paragraph::new(self.get_spans());
        Widget::render(text, rect, buffer);
    }
}

impl Spannable for HashInput {
    fn get_spans<'a, 'b>(&'a self) -> tui_components::tui::text::Spans<'b> {
        let mut spans = Spans::default();
        spans.0.push(Span::styled(
            String::from("> "),
            Style::default().fg(Color::Gray),
        ));
        let status = self.status();
        let color = match status {
            HashStatus::Hash(..) | HashStatus::LabelExists(..) => Color::Green,
            HashStatus::HashInvalid => Color::Red,
            HashStatus::LabelNotExists(..) | HashStatus::LabelsPoisoned(..) => Color::LightYellow,
        };
        spans
            .0
            .push(Span::styled(self.value.clone(), Style::default().fg(color)));
        spans
    }
}
