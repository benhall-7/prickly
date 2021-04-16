use super::{Component, Event, Input, InputResponse};
use crossterm::event::KeyCode;
use std::fs::{read_dir, Metadata};
use std::path::{Path, PathBuf};
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Paragraph, Row, StatefulWidget, Table, TableState, Widget};
use tui::{
    buffer::Buffer,
    layout::{Direction, Layout},
};

#[derive(Debug, Clone)]
pub struct Explorer {
    path: PathBuf,
    input: Input,
    input_active: bool,
    files: Result<Vec<EntryInfo>, String>,
    mode: ExplorerMode,
    /// used to confirm if the user wants to overwrite an existing file
    confirm_overwrite: Option<bool>,
    table_state: TableState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExplorerMode {
    Open,
    Save,
}

#[derive(Debug, Clone)]
struct EntryInfo {
    path: PathBuf,
    meta: Metadata,
}

impl Explorer {
    pub fn new<P: AsRef<Path>>(path: P, mode: ExplorerMode) -> Self {
        // get read_dir iterator, then collect file paths into a list
        let files = Self::get_files(&path);
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Explorer {
            path: path.as_ref().to_path_buf(),
            input: Input::default(),
            input_active: false,
            files,
            mode,
            confirm_overwrite: None,
            table_state,
        }
    }

    fn get_files<P: AsRef<Path>>(path: P) -> Result<Vec<EntryInfo>, String> {
        read_dir(path).map_err(|e| format!("{}", e)).map(|dir| {
            dir.into_iter()
                .filter_map(|sub| {
                    sub.ok().map(|s| EntryInfo {
                        path: s.path(),
                        meta: s.metadata().unwrap(),
                    })
                })
                .collect()
        })
    }

    fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.files = Self::get_files(&path);
        self.path = path.as_ref().to_path_buf();
        self.table_state.select(Some(0));
    }

    fn index(&self) -> usize {
        self.table_state.selected().unwrap()
    }

    fn increment(&mut self) {
        if let Ok(paths) = &self.files {
            let new = if self.index() >= paths.len().saturating_sub(1) {
                0
            } else {
                self.index() + 1
            };
            self.table_state.select(Some(new));
        }
    }

    fn decrement(&mut self) {
        if let Ok(paths) = &self.files {
            let new = if self.index() == 0 {
                paths.len().saturating_sub(1)
            } else {
                self.index() - 1
            };
            self.table_state.select(Some(new));
        }
    }

    fn selected_path(&self) -> Option<&EntryInfo> {
        let index = self.index();
        self.files
            .as_ref()
            .ok()
            .map(|files| files.get(index))
            .flatten()
    }
}

#[derive(Debug, Clone)]
pub enum ExplorerResponse {
    Open(PathBuf),
    Save(PathBuf),
    Cancel,
    Handled,
    None,
}

impl Component for Explorer {
    type Response = ExplorerResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        if let Some(overwrite) = &mut self.confirm_overwrite {
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Right if *overwrite => {
                        *overwrite = true;
                        ExplorerResponse::Handled
                    }
                    KeyCode::Left if !*overwrite => {
                        *overwrite = false;
                        ExplorerResponse::Handled
                    }
                    _ => ExplorerResponse::None,
                }
            } else {
                ExplorerResponse::None
            }
        } else if self.input_active {
            match self.input.handle_event(event) {
                InputResponse::Submit => {
                    if let Ok(files) = &self.files {
                        match self.mode {
                            ExplorerMode::Open => {
                                self.input_active = false;
                                ExplorerResponse::Handled
                            }
                            ExplorerMode::Save => {
                                // behavior:
                                // if input matches folder name exactly, traverse it
                                // else save file
                                let possible_folder = files
                                    .iter()
                                    .find(|f| {
                                        f.path.file_name().unwrap().to_string_lossy().as_ref()
                                            == &self.input.value
                                    })
                                    .and_then(|f| if f.meta.is_dir() { Some(f) } else { None });
                                match possible_folder {
                                    Some(folder) => {
                                        let p = folder.path.clone();
                                        self.set_path(p);
                                        ExplorerResponse::Handled
                                    }
                                    None => {
                                        ExplorerResponse::Save(self.path.join(&self.input.value))
                                    }
                                }
                            }
                        }
                    } else {
                        self.input_active = false;
                        ExplorerResponse::Handled
                    }
                }
                InputResponse::Cancel => {
                    self.input_active = false;
                    ExplorerResponse::Handled
                }
                InputResponse::Edited { deletion } => {
                    // change index to first match
                    if let Ok(files) = &self.files {
                        if deletion {
                            return ExplorerResponse::Handled;
                        }
                        if let Some(index) = files.iter().position(|file| {
                            file.path
                                .file_name()
                                // only fails if path ends in .., so this is fine
                                .unwrap()
                                .to_string_lossy()
                                .starts_with(&self.input.value)
                        }) {
                            self.table_state.select(Some(index));
                        }
                        ExplorerResponse::Handled
                    } else {
                        ExplorerResponse::Handled
                    }
                }
                InputResponse::None => ExplorerResponse::None,
            }
        } else {
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Esc => ExplorerResponse::Cancel,
                    KeyCode::Up => {
                        self.decrement();
                        ExplorerResponse::Handled
                    }
                    KeyCode::Down => {
                        self.increment();
                        ExplorerResponse::Handled
                    }
                    KeyCode::Enter => {
                        let info = self
                            .selected_path()
                            .map(|entry| (entry.path.clone(), entry.meta.is_dir()));
                        if let Some((path, is_dir)) = info {
                            if is_dir {
                                self.set_path(path);
                            } else {
                                match self.mode {
                                    ExplorerMode::Open => return ExplorerResponse::Open(path),
                                    ExplorerMode::Save => return ExplorerResponse::Save(path),
                                }
                            }
                        }
                        ExplorerResponse::Handled
                    }
                    KeyCode::Backspace => {
                        let parent = self.path.parent().map(|p| p.to_path_buf());
                        if let Some(par) = parent {
                            self.set_path(par.to_path_buf());
                        }
                        ExplorerResponse::Handled
                    }
                    KeyCode::Char('/') => {
                        self.input_active = true;
                        ExplorerResponse::Handled
                    }
                    _ => ExplorerResponse::None,
                }
            } else {
                ExplorerResponse::None
            }
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        self.input.focused = self.input_active;
        let title = match self.mode {
            ExplorerMode::Open => "Open File",
            ExplorerMode::Save => "Save File",
        };
        let outer = Block::default()
            .title(Span::styled(title, Style::default()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let inner = outer.inner(rect);
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(100),
            ])
            .split(inner);

        let p = Paragraph::new(self.path.to_string_lossy().to_string());

        Widget::render(outer, rect, buf);
        Widget::render(p, areas[0], buf);
        self.input.draw(areas[1], buf);
        match &self.files {
            Ok(files) => {
                let names = files
                    .iter()
                    .map(|p| {
                        let string = p
                            .path
                            .as_path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        Row::new(vec![string])
                    })
                    .collect::<Vec<_>>();
                let table = Table::new(names)
                    .widths(&[Constraint::Percentage(100)])
                    .highlight_style(Style::default().bg(Color::Green));
                StatefulWidget::render(table, areas[2], buf, &mut self.table_state);
            }
            Err(e) => {
                let p = Paragraph::new(Span::styled(e, Style::default().fg(Color::Red)));
                Widget::render(p, areas[2], buf);
            }
        }
    }
}
