use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use prc::hash40::{hash40, Hash40};
use tui_components::tui::style::Modifier;
use tui_components::tui::text::{Span, Spans};
use tui_components::Spannable;
use tui_components::{
    crossterm::event::KeyCode,
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
    sorted_labels: Arc<Mutex<BTreeSet<String>>>,
    matches: Vec<String>,
    match_num: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum Validity {
    LabelExists(Hash40),
    LabelNotExists(Hash40),
    LabelsPoisoned(Hash40),
    Hash(Hash40),
    HashInvalid,
}

impl HashInput {
    pub fn new(hash: Hash40, sorted_labels: Arc<Mutex<BTreeSet<String>>>) -> Self {
        let mut this = Self {
            value: hash.to_string(),
            return_value: hash,
            sorted_labels,
            matches: vec![],
            match_num: None,
        };
        this.update_matches();
        this
    }

    pub fn status(&self) -> Validity {
        if self.value.starts_with("0x") {
            match Hash40::from_hex_str(&self.value) {
                Ok(hash) => Validity::Hash(hash),
                Err(_) => Validity::HashInvalid,
            }
        } else {
            let label_arc = Hash40::label_map();
            let lock = label_arc.lock();
            if let Ok(labels) = lock {
                match labels.hash_of(&self.value) {
                    Some(hash) => Validity::LabelExists(hash),
                    None => Validity::LabelNotExists(hash40(&self.value)),
                }
            } else {
                Validity::LabelsPoisoned(hash40(&self.value))
            }
        }
    }

    pub fn value(&self) -> Hash40 {
        self.return_value
    }

    fn update_matches(&mut self) {
        let status = self.status();
        match status {
            Validity::LabelExists(..) | Validity::LabelNotExists(..) => {
                let sorted_lock = self.sorted_labels.lock();
                if let Ok(sorted_labels) = sorted_lock {
                    let prefix = self.value.to_owned();
                    let prefix_str = self.value.as_str();
                    self.matches = sorted_labels
                        .range(prefix..)
                        .take(1000) // limit to 100 matches for now
                        .take_while(|str| str.starts_with(prefix_str))
                        .map(|str| str.to_owned())
                        .collect();
                    if matches!(status, Validity::LabelNotExists(..)) && !self.matches.is_empty() {
                        self.match_num = Some(0)
                    } else {
                        self.match_num = None;
                    }
                } else {
                    self.matches = vec![];
                    self.match_num = None;
                }
            }
            _ => {
                self.matches = vec![];
                self.match_num = None;
            }
        }
    }

    fn current_match(&self) -> Option<&str> {
        self.match_num
            .and_then(|num| self.matches.get(num).map(|str| str.as_str()))
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
                    self.update_matches();
                    HashInputResponse::Handled
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    self.update_matches();
                    HashInputResponse::Handled
                }
                KeyCode::Down => {
                    if self.matches.is_empty() {
                        self.match_num = None;
                    } else if let Some(current) = self.match_num {
                        if current < self.matches.len() - 1 {
                            self.match_num = Some(current + 1);
                        }
                    } else {
                        self.match_num = Some(0)
                    }
                    HashInputResponse::Handled
                }
                KeyCode::Up => {
                    if self.matches.is_empty() {
                        self.match_num = None;
                    } else if let Some(current) = self.match_num {
                        if current > 0 {
                            self.match_num = Some(current - 1);
                        }
                    }
                    HashInputResponse::Handled
                }
                KeyCode::Tab => {
                    if let Some(current_match) = self.current_match() {
                        self.value = current_match.to_owned();
                        self.update_matches();
                    }
                    HashInputResponse::Handled
                }
                KeyCode::Enter => {
                    let status = self.status();
                    match status {
                        Validity::Hash(hash)
                        | Validity::LabelExists(hash)
                        | Validity::LabelNotExists(hash)
                        | Validity::LabelsPoisoned(hash) => {
                            self.return_value = hash;
                            HashInputResponse::Submit
                        }
                        Validity::HashInvalid => HashInputResponse::None,
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
            Validity::Hash(..) | Validity::LabelExists(..) => Color::Green,
            Validity::HashInvalid => Color::Red,
            Validity::LabelNotExists(..) | Validity::LabelsPoisoned(..) => Color::LightYellow,
        };
        spans
            .0
            .push(Span::styled(self.value.clone(), Style::default().fg(color)));
        if let Some(current_match) = self.current_match() {
            spans.0.push(Span::styled(
                format!(" ({})?", current_match.trim_start_matches(&self.value)),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ))
        }
        spans
    }
}
