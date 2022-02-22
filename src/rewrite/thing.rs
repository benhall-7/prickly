use std::fmt::Display;

use prc::{hash40::Hash40, ParamKind, ParamList, ParamStruct};
use tui_components::{
    crossterm::event::{KeyCode, KeyEvent},
    tui::{
        layout::{Constraint, Rect},
        style::{Color, Style},
        widgets::{Row, StatefulWidget, Table, TableState},
    },
    App, AppResponse, Component, Event,
};

const PARAM_TABLE_WIDTH: u16 = 20;
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
    last_draw_area: Option<Rect>,
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
            last_draw_area: None,
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

    fn draw(
        &mut self,
        rect: tui_components::tui::layout::Rect,
        buffer: &mut tui_components::tui::buffer::Buffer,
    ) {
        self.root.draw(rect, buffer);
    }
}

pub enum ParamResponse {
    None,
}

impl Component for Param {
    type Response = ParamResponse;

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

    fn draw(
        &mut self,
        rect: tui_components::tui::layout::Rect,
        buffer: &mut tui_components::tui::buffer::Buffer,
    ) {
        let right = if let Some(next) = self.next_mut() {
            next.draw(rect, buffer);
            let prev_area = next.last_draw_area.unwrap();
            prev_area.left()
        } else {
            rect.right()
        };
        if right > MIN_PARAM_TABLE_WIDTH {
            let left = right.saturating_sub(PARAM_TABLE_WIDTH);
            let table_area = Rect {
                x: left,
                y: rect.y,
                width: right.saturating_sub(left),
                height: rect.height,
            };
            self.last_draw_area = Some(table_area);

            let children = self.param.children();
            let rows = children
                .iter()
                .map(|(index, _)| format!("{}", index))
                .map(|str| Row::new(vec![str]));

            let table = Table::new(rows)
                .widths(&[Constraint::Percentage(100)])
                .highlight_style(Style::default().bg(Color::Blue));
            StatefulWidget::render(table, table_area, buffer, &mut self.state)
        }
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
