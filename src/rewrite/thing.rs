use std::fmt::Display;

use prc::{hash40::Hash40, ParamKind, ParamList, ParamStruct};
use tui_components::{
    crossterm::event::{KeyCode, KeyEvent},
    tui::{
        buffer::Buffer,
        layout::{Constraint, Rect},
        style::{Color, Style},
        text::Span,
        widgets::{Block, Borders, Row, StatefulWidget, Table, TableState, Widget},
    },
    App, AppResponse, Component, Event,
};

const MIN_PARAM_TABLE_WIDTH: u16 = 10;

use super::modulo::{add_mod, sub_mod};

#[derive(Debug, Clone)]
pub struct Container {
    root: Param,
}

#[derive(Debug, Clone)]
pub struct Param {
    param: ParamParent,
    state: TableState,
    selected: Option<Box<SelectedParam>>,
}

#[derive(Debug, Clone)]
pub enum ParamParent {
    List(ParamList),
    Struct(ParamStruct),
}

#[derive(Debug, Clone)]
pub enum SelectedParam {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    Float(f32),
    Hash(Hash40),
    Str(String),
    NewLevel(Param),
}

impl Container {
    pub fn new(param: ParamKind) -> Self {
        let root = Param::new(ParamParent::Struct(param.try_into_owned().unwrap()));
        Self { root }
    }
}

impl Param {
    pub fn new(param: ParamParent) -> Self {
        Self {
            param,
            state: TableState::default(),
            selected: None,
        }
    }

    fn down(&mut self) {
        let len = self.param.len();
        if len > 0 {
            match self.state.selected() {
                Some(selected) => self.state.select(Some(add_mod(selected, 1, len))),
                None => self.state.select(Some(0)),
            }
        } else {
            self.state.select(None);
        }
    }

    fn up(&mut self) {
        let len = self.param.len();
        if len > 0 {
            match self.state.selected() {
                Some(selected) => self.state.select(Some(sub_mod(selected, 1, len))),
                None => self.state.select(Some(len - 1)),
            }
        } else {
            self.state.select(None);
        }
    }

    fn enter(&mut self) -> bool {
        if let Some(selected) = self.state.selected() {
            match self.param.nth_mut(selected) {
                ParamKind::List(list) => {
                    let taken = std::mem::take(list);
                    let new_param = Param::new(ParamParent::List(taken));
                    self.selected = Some(Box::new(SelectedParam::NewLevel(new_param)));
                    true
                }
                ParamKind::Struct(str) => {
                    let taken = std::mem::take(str);
                    let new_param = Param::new(ParamParent::Struct(taken));
                    self.selected = Some(Box::new(SelectedParam::NewLevel(new_param)));
                    true
                }
                _ => false, // todo: this should begin an editing state
            }
        } else {
            false
        }
    }

    fn next(&self) -> Option<&Param> {
        match self.selected.as_deref() {
            Some(SelectedParam::NewLevel(level)) => Some(level),
            _ => None,
        }
    }

    fn next_mut(&mut self) -> Option<&mut Param> {
        match self.selected.as_deref_mut() {
            Some(SelectedParam::NewLevel(level)) => Some(level),
            _ => None,
        }
    }
}

impl App for Container {
    fn handle_event(&mut self, event: tui_components::Event) -> tui_components::AppResponse {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return AppResponse::Exit,
                _ => {}
            }
        }
        match self.root.handle_event(event) {
            ParamResponse::None => {}
        }
        AppResponse::None
    }

    fn draw(&mut self, rect: tui_components::tui::layout::Rect, buffer: &mut Buffer) {
        let param_buffer = self.root.draw(rect, buffer);
        buffer.merge(&param_buffer);
    }
}

pub enum ParamResponse {
    None,
}

impl<'a> Component for Param {
    type Response = ParamResponse;
    type DrawResponse = Buffer;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        self.next_mut()
            .map(|next| next.handle_event(event))
            .unwrap_or_else(|| {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Up => self.up(),
                        KeyCode::Down => self.down(),
                        KeyCode::Enter => {
                            self.enter();
                        }
                        _ => {}
                    }
                }
                ParamResponse::None
            })
    }

    fn draw(&mut self, rect: tui_components::tui::layout::Rect, _buffer: &mut Buffer) -> Buffer {
        let child_buffer = self.next_mut().map(|child| child.draw(rect, _buffer));
        let is_last_column = child_buffer.is_none();
        let remaining_space = child_buffer
            .as_ref()
            .map(|buf| rect.width - buf.area.width.min(rect.width))
            .unwrap_or(rect.width);

        // the 2nd condition makes it so we always draw the deepest param
        if remaining_space < MIN_PARAM_TABLE_WIDTH && child_buffer.is_some() {
            return child_buffer.unwrap();
        }

        let children = self.param.children();
        let columns = children
            .iter()
            .map(|(index, param)| {
                let name = format!("{}", index);
                let ty = String::from(param_type(param));
                let value = param_value(param);
                [name, ty, value]
            })
            .collect::<Vec<_>>();

        let widths = columns.iter().fold([0, 0, 0], |current, col| {
            [
                current[0].max(col[0].len() as u16),
                current[1].max(col[1].len() as u16),
                current[2].max(col[2].len() as u16),
            ]
        });
        // each column has 1 left border, and the last one has an extra right border
        let desired_width = widths.iter().sum::<u16>() + if child_buffer.is_some() { 3 } else { 4 };
        let true_width = desired_width.min(remaining_space);
        let draw_area = Rect {
            x: 0,
            y: rect.y,
            width: true_width,
            height: rect.height,
        };

        let block = if is_last_column {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        } else {
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray))
        };
        let table_area = block.inner(draw_area);

        let rows = columns.into_iter().map(Row::new);

        let constraints = widths.map(Constraint::Length);
        let table = if is_last_column {
            Table::new(rows)
                .widths(&constraints)
                .column_spacing(1)
                .highlight_style(Style::default().bg(Color::Blue))
        } else {
            Table::new(rows)
                .widths(&constraints)
                .column_spacing(1)
                .style(Style::default().fg(Color::DarkGray))
                .highlight_style(Style::default().fg(Color::Gray).bg(Color::Blue))
        };

        let mut draw_buffer = child_buffer
            .map(|mut buf| {
                // make space within the buf for the component to render
                let new_buf = Buffer::empty(draw_area);
                buf.area.x = true_width;
                buf.merge(&new_buf);
                buf
            })
            .unwrap_or_else(|| Buffer::empty(draw_area));

        Widget::render(block, draw_area, &mut draw_buffer);
        StatefulWidget::render(table, table_area, &mut draw_buffer, &mut self.state);

        draw_buffer
    }
}

pub enum ParentIndex {
    List(usize),
    Struct(Hash40),
}

impl ParamParent {
    pub fn children_mut(&mut self) -> Vec<(ParentIndex, &mut ParamKind)> {
        match self {
            ParamParent::List(list) => list
                .0
                .iter_mut()
                .enumerate()
                .map(|(index, param)| (ParentIndex::List(index), param))
                .collect(),
            ParamParent::Struct(str) => str
                .0
                .iter_mut()
                .map(|(hash, param)| (ParentIndex::Struct(*hash), param))
                .collect(),
        }
    }

    pub fn children(&self) -> Vec<(ParentIndex, &ParamKind)> {
        match self {
            ParamParent::List(list) => list
                .0
                .iter()
                .enumerate()
                .map(|(index, param)| (ParentIndex::List(index), param))
                .collect(),
            ParamParent::Struct(str) => str
                .0
                .iter()
                .map(|(hash, param)| (ParentIndex::Struct(*hash), param))
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ParamParent::List(list) => list.0.len(),
            ParamParent::Struct(str) => str.0.len(),
        }
    }

    pub fn nth(&self, n: usize) -> &ParamKind {
        match self {
            ParamParent::List(list) => &list.0[n],
            ParamParent::Struct(str) => &str.0[n].1,
        }
    }

    pub fn nth_mut(&mut self, n: usize) -> &mut ParamKind {
        match self {
            ParamParent::List(list) => &mut list.0[n],
            ParamParent::Struct(str) => &mut str.0[n].1,
        }
    }
}

impl Display for ParentIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParentIndex::List(index) => write!(f, "{}", *index),
            ParentIndex::Struct(hash) => write!(f, "{}", *hash),
        }
    }
}

fn param_type(param: &ParamKind) -> &'static str {
    match param {
        ParamKind::Bool(_) => "bool",
        ParamKind::I8(_) => "i8",
        ParamKind::U8(_) => "u8",
        ParamKind::I16(_) => "i16",
        ParamKind::U16(_) => "u16",
        ParamKind::I32(_) => "i32",
        ParamKind::U32(_) => "u32",
        ParamKind::Float(_) => "f32",
        ParamKind::Hash(_) => "hash",
        ParamKind::Str(_) => "string",
        ParamKind::List(_) => "list",
        ParamKind::Struct(_) => "struct",
    }
}

fn param_value(param: &ParamKind) -> String {
    match param {
        ParamKind::Bool(v) => format!("{}", v),
        ParamKind::I8(v) => format!("{}", v),
        ParamKind::U8(v) => format!("{}", v),
        ParamKind::I16(v) => format!("{}", v),
        ParamKind::U16(v) => format!("{}", v),
        ParamKind::I32(v) => format!("{}", v),
        ParamKind::U32(v) => format!("{}", v),
        ParamKind::Float(v) => format!("{}", v),
        ParamKind::Hash(v) => format!("{}", v),
        ParamKind::Str(v) => v.to_string(),
        ParamKind::List(v) => format!("({} children)", v.0.len()),
        ParamKind::Struct(v) => format!("({} children)", v.0.len()),
    }
}
