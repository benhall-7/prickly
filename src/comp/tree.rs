use super::{Component, Event, TreeData, TreeRow};
use std::cmp::max;

use crossterm::event::KeyCode;
use prc::param::*;
use regex::Regex;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget};

pub struct Tree {
    // when we update our regex filter, we have to manually update this too
    data: TreeData,
    selection: TableState,
    focused: bool,
}

impl Tree {
    pub fn new(param: &ParamKind, filter: Option<&Regex>) -> Self {
        let mut selection = TableState::default();
        selection.select(Some(0));

        Tree {
            data: TreeData::new(param).apply_filter(filter),
            selection,
            focused: true,
        }
    }

    pub fn new_with_state(param: &ParamKind, filter: Option<&Regex>, state: TableState) -> Self {
        Tree {
            data: TreeData::new(param).apply_filter(filter),
            selection: state,
            focused: true,
        }
    }

    pub fn current_row(&self) -> &TreeRow {
        &self.data.rows[self.index()]
    }

    pub fn is_empty(&self) -> bool {
        self.data.rows.is_empty()
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

    fn index(&self) -> usize {
        self.selection.selected().unwrap()
    }

    fn set_index(&mut self, new: usize) {
        self.selection.select(Some(new));
    }

    fn inc(&mut self) {
        self.selection.select(Some(self.index() + 1));
    }

    fn dec(&mut self) {
        self.selection.select(Some(self.index() - 1));
    }
}

pub enum TreeResponse {
    None,
    Focus,
    Unfocus,
}

impl Component for Tree {
    type Response = TreeResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Up => {
                    if self.data.rows.is_empty() {
                        self.set_index(0);
                    } else if self.index() == 0 {
                        self.set_index(self.data.rows.len() - 1);
                    } else {
                        self.dec();
                    }
                }
                KeyCode::Down => {
                    if self.data.rows.is_empty() {
                        self.set_index(0);
                    } else if self.index() >= self.data.rows.len() - 1 {
                        self.set_index(0);
                    } else {
                        self.inc();
                    }
                }
                // might change these two
                KeyCode::Right => return TreeResponse::Focus,
                KeyCode::Left => return TreeResponse::Unfocus,
                _ => {}
            }
        }
        TreeResponse::None
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Blue)
            } else {
                Default::default()
            })
            .title(Span::styled("PARAMS", Style::default().fg(Color::White)));
        let mut table_area = block.inner(rect);
        table_area.height -= 1;

        let name_len = self
            .data
            .rows
            .iter()
            .fold(0, |max_len, row| max(max_len, row.name.len())) as u16;
        let type_len = self
            .data
            .rows
            .iter()
            .fold(0, |max_len, row| max(max_len, row.kind.len())) as u16;
        let constraints = [
            Constraint::Length(name_len),
            Constraint::Length(type_len),
            Constraint::Percentage(100),
        ];

        let table = Table::new(
            self.data
                .rows
                .iter()
                .map(|row| Row::new(vec![row.name.as_str(), row.kind, row.value.as_str()])),
        )
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
