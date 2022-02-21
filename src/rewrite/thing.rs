use prc::{hash40::Hash40, ParamKind, ParamList, ParamStruct};
use tui_components::{
    crossterm::event::{KeyCode, KeyEvent},
    tui::{
        style::Style,
        widgets::{Row, StatefulWidget, Table, TableState},
    },
    App, AppResponse, Component, Event,
};

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

    fn len(&self) -> usize {
        let mut len = 1;
        let mut node = &self.root;
        loop {
            node = match node.next() {
                Some(next) => {
                    len += 1;
                    next
                }
                None => return len,
            }
        }
    }

    fn nth(&self, n: usize) -> Option<&Param> {
        let mut nth_node = Some(&self.root);
        for _ in 0..n {
            nth_node = match nth_node {
                None => return None,
                Some(current) => current.next(),
            }
        }
        nth_node
    }

    fn nth_mut(&mut self, n: usize) -> Option<&mut Param> {
        let mut nth_node = Some(&mut self.root);
        for _ in 0..n {
            nth_node = match nth_node {
                None => return None,
                Some(current) => current.next_mut(),
            }
        }
        nth_node
    }

    fn tail(&self) -> &Param {
        self.nth(self.len() - 1).unwrap()
    }

    fn tail_mut(&mut self) -> &mut Param {
        self.nth_mut(self.len() - 1).unwrap()
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
        let len = match &self.param {
            ParamParent::List(list) => list.0.len(),
            ParamParent::Struct(str) => str.0.len(),
        };
        if len > 0 {
            match self.state.selected() {
                Some(selected) => self.state.select(Some((selected + 1) % len)),
                None => self.state.select(Some(0)),
            }
        } else {
            self.state.select(None);
        }
    }

    fn up(&mut self) {
        let len = match &self.param {
            ParamParent::List(list) => list.0.len(),
            ParamParent::Struct(str) => str.0.len(),
        };
        if len > 0 {
            match self.state.selected() {
                Some(selected) => self.state.select(Some((selected + 1) % len)),
                None => self.state.select(Some(0)),
            }
        } else {
            self.state.select(None);
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
                KeyCode::Esc => AppResponse::Exit,
                _ => AppResponse::None,
            }
        } else {
            AppResponse::None
        }
    }

    fn draw(
        &mut self,
        rect: tui_components::tui::layout::Rect,
        buffer: &mut tui_components::tui::buffer::Buffer,
    ) {
        let tail = self.tail_mut();
        if let ParamParent::Struct(str) = &tail.param {}
    }
}

// pub enum ParamResponse {
//     None,
// }

// impl Component for Param {
//     type Response = ParamResponse;

//     fn handle_event(&mut self, event: Event) -> Self::Response {
//         if let Event::Key(key) = event {

//         }
//         None
//     }

//     fn draw(
//         &mut self,
//         rect: tui_components::tui::layout::Rect,
//         buffer: &mut tui_components::tui::buffer::Buffer,
//     ) {
//         todo!()
//     }
// }
