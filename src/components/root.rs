use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use prc::ParamKind;
use tui_components::{
    crossterm::event::KeyCode, tui::buffer::Buffer, App, AppResponse, Component, Event,
};

use super::param::{Param, ParamParent, ParamResponse};

#[derive(Debug)]
pub struct Root {
    root: Param,
}

impl App for Root {
    fn handle_event(&mut self, event: tui_components::Event) -> tui_components::AppResponse {
        if let ParamResponse::None = self.root.handle_event(event) {
            if let Event::Key(key) = event {
                if key.code == KeyCode::Esc {
                    return AppResponse::Exit;
                }
            }
        }
        AppResponse::None
    }

    fn draw(&mut self, rect: tui_components::tui::layout::Rect, buffer: &mut Buffer) {
        let param_buffer = self.root.draw(rect, buffer);
        buffer.merge(&param_buffer);
    }
}

impl Root {
    pub fn new(param: ParamKind, sorted_labels: Arc<Mutex<BTreeSet<String>>>) -> Self {
        let root = Param::new(
            ParamParent::Struct(param.try_into_owned().unwrap()),
            sorted_labels,
        );
        Self { root }
    }
}
