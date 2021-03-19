use super::{Component, Event};
use std::cmp::max;

use crossterm::event::KeyCode;
use prc::param::*;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Row, StatefulWidget, Table, TableState};

pub struct Tree<'a, 't> {
    pub param: Backing<'a>,
    pub selection: &'t mut TableState,
}

pub enum Backing<'a> {
    List(&'a mut ParamList),
    Struct(&'a mut ParamStruct),
}

impl<'a, 't> Tree<'a, 't> {
    pub fn new(param: &'a mut ParamKind, selection: &'t mut TableState) -> Self {
        let backing = match param {
            ParamKind::Struct(s) => Backing::Struct(s),
            ParamKind::List(l) => Backing::List(l),
            _ => panic!("Only struct or list params can be used for trees"),
        };
        Tree {
            param: backing,
            selection,
        }
    }

    pub fn current_param(&mut self) -> &mut ParamKind {
        self.param.param_at_mut(self.index() as usize)
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

impl<'a> Backing<'a> {
    pub fn len(&self) -> usize {
        match self {
            Backing::List(l) => l.0.len(),
            Backing::Struct(s) => s.0.len(),
        }
    }

    pub fn name_at(&self, index: usize) -> String {
        match self {
            Backing::List(_) => format!("{}", index),
            Backing::Struct(s) => format!("{}", s.0[index].0),
        }
    }

    pub fn param_at(&self, index: usize) -> &ParamKind {
        match self {
            Backing::List(l) => &l.0[index],
            Backing::Struct(s) => &s.0[index].1,
        }
    }

    pub fn param_at_mut(&mut self, index: usize) -> &mut ParamKind {
        match self {
            Backing::List(l) => &mut l.0[index],
            Backing::Struct(s) => &mut s.0[index].1,
        }
    }
}

pub enum TreeResponse {
    None,
    Focus,
    Unfocus,
}

impl<'a, 't> Component for Tree<'a, 't> {
    type Response = TreeResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Up => {
                    if self.index() == 0 {
                        self.set_index(self.param.len() - 1);
                    } else {
                        self.dec();
                    }
                }
                KeyCode::Down => {
                    if self.index() >= self.param.len() - 1 {
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
        let mut table_area = rect;
        table_area.height -= 1;
        let data: Vec<_> = (0..self.param.len() as u16)
            .into_iter()
            .map(|i| {
                let [ty, val] = param_info(self.param.param_at(i as usize));
                [self.param.name_at(i as usize), ty, val]
            })
            .collect();
        let name_len = data
            .iter()
            .fold(0, |max_len, data| max(max_len, data[0].len())) as u16;
        let type_len = data
            .iter()
            .fold(0, |max_len, data| max(max_len, data[1].len())) as u16;
        let constraints = [
            Constraint::Min(name_len),
            Constraint::Min(type_len),
            Constraint::Percentage(100),
        ];

        let table = Table::new(
            data.iter()
                .map(|info| Row::new(info.iter().map(|s| &s[..]))),
        )
        .widths(&constraints)
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
        table.render(table_area, buf, self.selection);
    }
}

fn param_info(param: &ParamKind) -> [String; 2] {
    match param {
        ParamKind::Bool(v) => ["bool".into(), format!("{}", v)],
        ParamKind::I8(v) => ["i8".into(), format!("{}", v)],
        ParamKind::U8(v) => ["u8".into(), format!("{}", v)],
        ParamKind::I16(v) => ["i16".into(), format!("{}", v)],
        ParamKind::U16(v) => ["u16".into(), format!("{}", v)],
        ParamKind::I32(v) => ["i32".into(), format!("{}", v)],
        ParamKind::U32(v) => ["u32".into(), format!("{}", v)],
        ParamKind::Float(v) => ["f32".into(), format!("{}", v)],
        ParamKind::Hash(v) => ["hash".into(), format!("{}", v)],
        ParamKind::Str(v) => ["string".into(), format!("{}", v)],
        ParamKind::List(v) => ["list".into(), format!("({} children)", v.0.len())],
        ParamKind::Struct(v) => ["struct".into(), format!("({} children)", v.0.len())],
    }
}
