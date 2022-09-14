use std::{
    collections::BTreeSet,
    env::current_dir,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use prc::ParamKind;
use tui_components::{
    components::{Confirm, ConfirmResponse, Explorer, ExplorerMode, ExplorerResponse},
    crossterm::event::{KeyCode, KeyModifiers},
    rect_ext::RectExt,
    tui::{
        buffer::Buffer,
        widgets::{Clear, Widget},
    },
    App, AppResponse, Component, Event,
};

use super::{
    empty::Empty,
    param::{Param, ParamParent, ParamResponse},
};

#[derive(Debug)]
pub struct Root {
    state: State,
    sorted_labels: Arc<Mutex<BTreeSet<String>>>,
    open_dir: PathBuf,
    save_dir: PathBuf,
}

#[derive(Debug)]
enum State {
    Empty(EmptyState),
    Normal {
        param: Param,
        edited: bool,
        state: Box<NormalState>,
    },
}

#[derive(Debug)]
enum EmptyState {
    View,
    Open(Box<Explorer>),
}

#[derive(Debug)]
enum NormalState {
    View,
    Open(Explorer),
    Save(Explorer),
    ConfirmExit(Confirm),
    ConfirmOpen(Confirm),
}

impl Root {
    pub fn new(param: Option<ParamKind>, sorted_labels: Arc<Mutex<BTreeSet<String>>>) -> Self {
        let open_dir = current_dir().unwrap();
        let save_dir = open_dir.clone();
        if let Some(some) = param {
            let param = Param::new(
                ParamParent::Struct(some.try_into_owned().unwrap()),
                sorted_labels.clone(),
            );
            Self {
                state: State::Normal {
                    param,
                    edited: false,
                    state: Box::new(NormalState::View),
                },
                sorted_labels,
                open_dir,
                save_dir,
            }
        } else {
            Self {
                state: State::Empty(EmptyState::View),
                sorted_labels,
                open_dir,
                save_dir,
            }
        }
    }

    fn open(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            self.open_dir = parent.to_path_buf();
        }
        match prc::open(path) {
            Ok(prc) => {
                self.state = State::Normal {
                    param: Param::new(ParamParent::Struct(prc), self.sorted_labels.clone()),
                    edited: false,
                    state: Box::new(NormalState::View),
                };
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn save(&mut self, path: PathBuf) {
        if let State::Normal {
            param,
            edited,
            state,
        } = &mut self.state
        {
            if let Some(parent) = path.parent() {
                self.save_dir = parent.to_path_buf();
            }
            let param = param.recreate_param();
            if prc::save(path, param.try_into_ref().unwrap()).is_ok() {
                *edited = false;
            }
            // TODO: error message in case of failure
            *state = Box::new(NormalState::View);
        }
    }
}

impl App for Root {
    fn handle_event(&mut self, event: Event) -> AppResponse {
        match &mut self.state {
            State::Empty(EmptyState::View) => {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Esc => return AppResponse::Exit,
                        KeyCode::Char('o')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            self.state = State::Empty(EmptyState::Open(Box::new(Explorer::new(
                                self.open_dir.clone(),
                                ExplorerMode::Open,
                            ))))
                        }
                        KeyCode::Char('s')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            self.state = State::Empty(EmptyState::Open(Box::new(Explorer::new(
                                self.open_dir.clone(),
                                ExplorerMode::Open,
                            ))))
                        }
                        _ => {}
                    }
                }
            }
            State::Empty(EmptyState::Open(open)) => match open.handle_event(event) {
                ExplorerResponse::Open(path) => self.open(path).unwrap_or_default(),
                ExplorerResponse::Save(_) => {}
                ExplorerResponse::Cancel => self.state = State::Empty(EmptyState::View),
                ExplorerResponse::Handled => {}
                ExplorerResponse::None => {}
            },
            State::Normal {
                param,
                edited,
                state,
            } => match state.as_mut() {
                NormalState::View => match param.handle_event(event) {
                    ParamResponse::None => {
                        if let Event::Key(key) = event {
                            match key.code {
                                KeyCode::Esc => {
                                    if *edited {
                                        let msg = "You have unsaved changes. Are you sure you want to exit?";
                                        *state =
                                            Box::new(NormalState::ConfirmExit(Confirm::new(msg)));
                                    } else {
                                        return AppResponse::Exit;
                                    }
                                }
                                KeyCode::Char('o')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    if *edited {
                                        let msg = "You have unsaved changes. Are you sure you want to open a new file?";
                                        *state =
                                            Box::new(NormalState::ConfirmOpen(Confirm::new(msg)));
                                    } else {
                                        *state = Box::new(NormalState::Open(Explorer::new(
                                            self.open_dir.clone(),
                                            ExplorerMode::Open,
                                        )));
                                    }
                                }
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    *state = Box::new(NormalState::Save(Explorer::new(
                                        self.save_dir.clone(),
                                        ExplorerMode::Save,
                                    )));
                                }
                                _ => {}
                            }
                        }
                    }
                    ParamResponse::Handled {
                        edited: component_edited,
                    } => {
                        if component_edited {
                            *edited = true;
                        }
                    }
                    ParamResponse::Exit => {}
                },
                NormalState::Open(open) => match open.handle_event(event) {
                    ExplorerResponse::Open(path) => self.open(path).unwrap_or_default(),
                    ExplorerResponse::Cancel => *state = Box::new(NormalState::View),
                    ExplorerResponse::Save(_) => {}
                    ExplorerResponse::Handled => {}
                    ExplorerResponse::None => {}
                },
                NormalState::Save(save) => match save.handle_event(event) {
                    ExplorerResponse::Save(path) => self.save(path),
                    ExplorerResponse::Cancel => *state = Box::new(NormalState::View),
                    ExplorerResponse::Open(_) => {}
                    ExplorerResponse::Handled => {}
                    ExplorerResponse::None => {}
                },
                NormalState::ConfirmExit(confirm) => match confirm.handle_event(event) {
                    ConfirmResponse::Confirm(answer) => {
                        if answer {
                            return AppResponse::Exit;
                        } else {
                            *state = Box::new(NormalState::View);
                        }
                    }
                    ConfirmResponse::Handled => {}
                    ConfirmResponse::None => {}
                },
                NormalState::ConfirmOpen(confirm) => match confirm.handle_event(event) {
                    ConfirmResponse::Confirm(answer) => {
                        if answer {
                            *state = Box::new(NormalState::Open(Explorer::new(
                                self.open_dir.clone(),
                                ExplorerMode::Open,
                            )));
                        } else {
                            *state = Box::new(NormalState::View);
                        }
                    }
                    ConfirmResponse::Handled => {}
                    ConfirmResponse::None => {}
                },
            },
        }
        AppResponse::None
    }

    fn draw(&mut self, rect: tui_components::tui::layout::Rect, buffer: &mut Buffer) {
        let explorer_rect = rect.centered(rect.scaled(0.75, 0.75));

        match &mut self.state {
            State::Empty(EmptyState::View) => {
                Empty.draw(rect, buffer);
            }
            State::Empty(EmptyState::Open(open)) => {
                open.draw(explorer_rect, buffer);
            }
            State::Normal {
                param,
                edited: _,
                state,
            } => {
                let param_buffer = param.draw(rect, buffer);
                buffer.merge(&param_buffer);

                match state.as_mut() {
                    NormalState::View => {}
                    NormalState::Open(open) => {
                        let clear = Clear;
                        clear.render(explorer_rect, buffer);
                        open.draw(explorer_rect, buffer)
                    }
                    NormalState::Save(save) => {
                        let clear = Clear;
                        clear.render(explorer_rect, buffer);
                        save.draw(explorer_rect, buffer)
                    }
                    // TODO: updated boundaries
                    NormalState::ConfirmExit(confirm) => confirm.draw(rect, buffer),
                    NormalState::ConfirmOpen(confirm) => confirm.draw(rect, buffer),
                }
            }
        }
    }
}
