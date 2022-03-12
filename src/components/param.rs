use std::fmt::Display;

use prc::{hash40::Hash40, ParamKind, ParamList, ParamStruct};
use tui_components::components::num_input::{NumInputResponse, SignedIntInput, UnsignedIntInput};
use tui_components::components::{Checkbox, CheckboxResponse, Input, InputResponse};
use tui_components::crossterm::event::KeyCode;
use tui_components::tui::buffer::Buffer;
use tui_components::tui::layout::{Constraint, Rect};
use tui_components::tui::style::{Color, Style};
use tui_components::tui::text::Spans;
use tui_components::tui::widgets::{Block, Borders, Row, StatefulWidget, Table, Widget};
use tui_components::Event;
use tui_components::{tui::widgets::TableState, Component};

use crate::utils::modulo::{add_mod, sub_mod};

const MIN_PARAM_TABLE_WIDTH: u16 = 10;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum SelectedParam {
    Bool(Checkbox),
    I8(SignedIntInput<i8>),
    U8(UnsignedIntInput<u8>),
    I16(SignedIntInput<i16>),
    U16(UnsignedIntInput<u16>),
    I32(SignedIntInput<i32>),
    U32(UnsignedIntInput<u32>),
    Float(f32),
    Hash(Hash40),
    Str(Input),
    NewLevel(Param),
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
                ParamKind::Bool(val) => {
                    self.selected = Some(Box::new(SelectedParam::Bool(Checkbox::new(*val))));
                    true
                }
                ParamKind::I8(int) => {
                    self.selected = Some(Box::new(SelectedParam::I8(SignedIntInput::new(*int))));
                    true
                }
                ParamKind::U8(int) => {
                    self.selected = Some(Box::new(SelectedParam::U8(UnsignedIntInput::new(*int))));
                    true
                }
                ParamKind::I16(int) => {
                    self.selected = Some(Box::new(SelectedParam::I16(SignedIntInput::new(*int))));
                    true
                }
                ParamKind::U16(int) => {
                    self.selected = Some(Box::new(SelectedParam::U16(UnsignedIntInput::new(*int))));
                    true
                }
                ParamKind::I32(int) => {
                    self.selected = Some(Box::new(SelectedParam::I32(SignedIntInput::new(*int))));
                    true
                }
                ParamKind::U32(int) => {
                    self.selected = Some(Box::new(SelectedParam::U32(UnsignedIntInput::new(*int))));
                    true
                }
                ParamKind::Str(str) => {
                    let mut input = Input::default();
                    input.value = str.clone();
                    input.focused = true;
                    self.selected = Some(Box::new(SelectedParam::Str(input)));
                    true
                }
                _ => false, // todo: this should begin an editing state
            }
        } else {
            false
        }
    }

    /// Removes selection from the current param.
    /// If the selected param was a value, update_value determines whether or not we update it
    fn exit(&mut self, update_value: bool) {
        if let Some(index) = self.state.selected() {
            if let Some(selected) = self.selected.take() {
                match *selected {
                    SelectedParam::NewLevel(level) => match level.param {
                        ParamParent::List(list) => *self.param.nth_mut(index) = list.into(),
                        ParamParent::Struct(str) => *self.param.nth_mut(index) = str.into(),
                    },
                    SelectedParam::Bool(val) => {
                        if update_value {
                            *self.param.nth_mut(index) = val.value.into()
                        }
                    }
                    SelectedParam::I8(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::U8(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::I16(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::U16(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::I32(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::U32(int) => {
                        if update_value {
                            *self.param.nth_mut(index) = int.value().into()
                        }
                    }
                    SelectedParam::Float(_) => todo!(),
                    SelectedParam::Hash(_) => todo!(),
                    SelectedParam::Str(str) => {
                        if update_value {
                            *self.param.nth_mut(index) = str.value.into()
                        }
                    }
                }
            }
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

    fn get_selected_span<'a, 'b: 'a>(&'a self) -> Option<(usize, Spans<'b>)> {
        self.state
            .selected()
            .zip(self.selected.as_deref())
            .map(|(index, selected)| {
                let spans = match &selected {
                    // can't select the todo! ones yet, so this is safe
                    SelectedParam::Bool(val) => val.get_span_builder().get_spans(),
                    SelectedParam::I8(int) => int.get_span_builder().get_spans(),
                    SelectedParam::U8(int) => int.get_span_builder().get_spans(),
                    SelectedParam::I16(int) => int.get_span_builder().get_spans(),
                    SelectedParam::U16(int) => int.get_span_builder().get_spans(),
                    SelectedParam::I32(int) => int.get_span_builder().get_spans(),
                    SelectedParam::U32(int) => int.get_span_builder().get_spans(),
                    SelectedParam::Float(_) => todo!(),
                    SelectedParam::Hash(_) => todo!(),
                    SelectedParam::Str(str) => str.get_span_builder().get_spans(),
                    SelectedParam::NewLevel(param) => match &param.param {
                        ParamParent::List(list) => {
                            Spans::from(format!("({} children)", list.0.len()))
                        }
                        ParamParent::Struct(str) => {
                            Spans::from(format!("({} children)", str.0.len()))
                        }
                    },
                };
                (index, spans)
            })
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

#[derive(Debug, Clone)]
pub enum ParamResponse {
    None,
    Exit,
    Handled,
}

impl<'a> Component for Param {
    type Response = ParamResponse;
    type DrawResponse = Buffer;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        // if the param has a child, see what it returns
        //    if it returns an "Exit" event, unselect and call the exit function
        //    if it returns a "None" event, do nothing
        // else, we're at the base and will handle the event here
        if let Some(next) = self.next_mut() {
            match next.handle_event(event) {
                ParamResponse::Exit => self.exit(false),
                ParamResponse::Handled => {}
                ParamResponse::None => return ParamResponse::None,
            }
        } else if let Some(selected) = self.selected.as_deref_mut() {
            let response = match selected {
                SelectedParam::I8(int) => int.handle_event(event),
                SelectedParam::U8(int) => int.handle_event(event),
                SelectedParam::I16(int) => int.handle_event(event),
                SelectedParam::U16(int) => int.handle_event(event),
                SelectedParam::I32(int) => int.handle_event(event),
                SelectedParam::U32(int) => int.handle_event(event),
                SelectedParam::Bool(val) => {
                    match val.handle_event(event) {
                        CheckboxResponse::Submit => self.exit(true),
                        CheckboxResponse::Exit => self.exit(false),
                        _ => {}
                    }
                    return ParamResponse::Handled;
                }
                SelectedParam::Str(str) => {
                    match str.handle_event(event) {
                        InputResponse::Submit => self.exit(true),
                        InputResponse::Cancel => self.exit(false),
                        _ => {}
                    }
                    return ParamResponse::Handled;
                }
                _ => unreachable!(),
            };
            match response {
                NumInputResponse::Submit => self.exit(true),
                NumInputResponse::Cancel => self.exit(false),
                _ => {}
            }
        } else if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => self.up(),
                KeyCode::Down => self.down(),
                KeyCode::Enter => {
                    self.enter();
                }
                KeyCode::Backspace => return ParamResponse::Exit,
                _ => return ParamResponse::None,
            }
        }
        ParamResponse::Handled
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

        let selected_info = self.get_selected_span();

        let children = self.param.children();
        let columns = children
            .iter()
            .enumerate()
            .map(|(list_index, (index, param))| {
                let name = Spans::from(format!("{}", index));
                let ty = Spans::from(param_type(param));

                let value = match &selected_info {
                    Some((selected_index, spans)) if list_index == *selected_index => {
                        spans.to_owned()
                    }
                    _ => Spans::from(param_value(param)),
                };
                [name, ty, value]
            })
            .collect::<Vec<_>>();

        let widths = columns.iter().fold([0, 0, 0], |current, col| {
            [
                current[0].max(col[0].width() as u16),
                current[1].max(col[1].width() as u16),
                current[2].max(col[2].width() as u16),
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
        ParamKind::Bool(v) => if *v { '✓' } else { '✗' }.into(),
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
