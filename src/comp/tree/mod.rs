pub mod tree_data;

use std::cmp::max;
use tree_data::{TreeData, TreeRow};

use crossterm::event::KeyCode;
use prc::param::*;
use regex::Regex;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget};
use tui_components::components::*;
use tui_components::crossterm::event::KeyModifiers;
use tui_components::{crossterm, tui, Component, Event};

pub struct Tree {
    // when we update our regex filter, we have to manually update this too
    data: TreeData,
    selection: TableState,
    focused: bool,
    editing: Option<Input>,
}

pub struct EditingData<'a> {
    input: &'a mut Input,
    param_index: usize,
}

impl Tree {
    pub fn new(param: &ParamKind, filter: Option<&Regex>) -> Self {
        let mut selection = TableState::default();
        selection.select(Some(0));

        Tree {
            data: TreeData::new(param).apply_filter(filter),
            selection,
            focused: true,
            editing: None,
        }
    }

    pub fn new_with_state(param: &ParamKind, filter: Option<&Regex>, state: TableState) -> Self {
        Tree {
            data: TreeData::new(param).apply_filter(filter),
            selection: state,
            focused: true,
            editing: None,
        }
    }

    pub fn current_row(&self) -> Option<&TreeRow> {
        self.data.rows.get(self.index())
    }

    pub fn current_row_mut(&mut self) -> Option<&mut TreeRow> {
        let index = self.index();
        self.data.rows.get_mut(index)
    }

    pub fn focus(&mut self, focus: bool) {
        self.focused = focus;
    }

    pub fn table_state(&self) -> &TableState {
        &self.selection
    }

    // possible optimization by bsearch
    pub fn select_param_index(&mut self, index: usize) {
        self.selection.select(Some(
            self.data
                .rows
                .iter()
                .position(|r| r.index == index)
                .unwrap_or(0),
        ));
    }

    pub fn start_editing(&mut self) {
        let mut input = Input::default().error_style(Style::default().fg(Color::Yellow));
        input.focused = true;
        self.editing = Some(input)
    }

    pub fn set_editing_error(&mut self, error: Option<String>) {
        if let Some(input) = &mut self.editing {
            input.error = error;
        }
    }

    pub fn finish_editing(&mut self) {
        self.editing = None;
    }

    /// Returns information about the currently active editor, if any.
    /// When returning Some, it contains a mutable reference to the Input,
    /// as well as the index of the param being edited
    fn editing_data(&mut self) -> Option<EditingData> {
        let param_index = self.param_index();
        self.editing
            .as_mut()
            .zip(param_index)
            .map(|(input, param_index)| EditingData { input, param_index })
    }

    fn index(&self) -> usize {
        self.selection.selected().unwrap_or(0)
    }

    fn param_index(&self) -> Option<usize> {
        self.current_row().map(|row| row.index)
    }

    fn set_index(&mut self, new: usize) {
        self.selection.select(Some(new));
    }

    fn inc(&mut self) {
        if self.data.rows.is_empty() || self.index() >= self.data.rows.len() - 1 {
            self.set_index(0);
        } else {
            self.selection.select(Some(self.index() + 1));
        }
    }

    fn dec(&mut self) {
        if self.data.rows.is_empty() {
            self.set_index(0);
        } else if self.index() == 0 {
            self.set_index(self.data.rows.len() - 1);
        } else {
            self.selection.select(Some(self.index() - 1));
        }
    }
}

pub enum TreeResponse {
    None,
    Focus,
    Unfocus,
    Handled,
    IncValue(usize),
    DecValue(usize),
    SetValue(usize, String),
}

impl Component for Tree {
    type Response = TreeResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Some(EditingData { input, param_index }) = self.editing_data() {
            match input.handle_event(event) {
                InputResponse::Submit => {
                    return TreeResponse::SetValue(param_index, input.value.clone())
                }
                InputResponse::Cancel => self.editing = None,
                InputResponse::None | InputResponse::Edited { .. } => {}
            }
            TreeResponse::Handled
        } else if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Up => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        match self.current_row() {
                            Some(row) if row.kind.is_incremental() => {
                                return TreeResponse::IncValue(row.index)
                            }
                            _ => {}
                        }
                    } else {
                        self.dec();
                    }
                    TreeResponse::Handled
                }
                KeyCode::Down => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        match self.current_row() {
                            Some(row) if row.kind.is_incremental() => {
                                return TreeResponse::DecValue(row.index)
                            }
                            _ => {}
                        }
                    } else {
                        self.inc();
                    }
                    TreeResponse::Handled
                }
                // might change these two
                KeyCode::Enter => TreeResponse::Focus,
                KeyCode::Backspace => TreeResponse::Unfocus,
                _ => TreeResponse::None,
            }
        } else {
            TreeResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            })
            .title(Span::styled(
                if self.editing.is_some() {
                    "PARAMS (editing)"
                } else {
                    "PARAMS"
                },
                Style::default().fg(Color::White),
            ));
        let mut table_area = block.inner(rect);
        table_area.height -= 1;

        let name_len = self
            .data
            .rows
            .iter()
            .fold(0, |max_len, row| max(max_len, row.name.len())) as u16;
        let type_len =
            self.data
                .rows
                .iter()
                .fold(0, |max_len, row| max(max_len, row.kind.as_str().len())) as u16;
        let constraints = [
            Constraint::Length(name_len),
            Constraint::Length(type_len),
            Constraint::Percentage(100),
        ];

        let index = self.current_row().map(|r| r.index).unwrap_or(0);
        let editing = self.editing.clone();
        let table = Table::new(self.data.rows.iter().map(|row| {
            let value = if row.index == index && editing.is_some() {
                editing.as_ref().unwrap().get_spans()
            } else {
                Spans::from(row.value.as_str())
            };
            Row::new(vec![
                row.name.as_str().into(),
                row.kind.as_str().into(),
                value,
            ])
        }))
        .widths(&constraints)
        .highlight_style(if self.focused {
            Style::default().bg(Color::Blue)
        } else {
            Style::default().bg(Color::DarkGray)
        });
        Widget::render(block, rect, buf);
        StatefulWidget::render(table, table_area, buf, &mut self.selection);
    }
}
