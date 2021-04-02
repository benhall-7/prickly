use super::{Component, Event};
use crossterm::event::KeyCode;
use std::fs::{read_dir, Metadata};
use std::path::{Path, PathBuf};
use tui::buffer::Buffer;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Paragraph, Row, StatefulWidget, Table, TableState, Widget};

#[derive(Debug, Clone)]
pub struct Explorer {
    path: PathBuf,
    files: Result<Vec<EntryInfo>, String>,
    mode: ExplorerMode,
    state: TableState,
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
        let mut state = TableState::default();
        state.select(Some(0));
        Explorer {
            path: path.as_ref().to_path_buf(),
            files,
            mode,
            state,
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
        self.state.select(Some(0));
    }

    fn index(&self) -> usize {
        self.state.selected().unwrap()
    }

    fn increment(&mut self) {
        if let Ok(paths) = &self.files {
            let new = if self.index() >= paths.len().saturating_sub(1) {
                0
            } else {
                self.index() + 1
            };
            self.state.select(Some(new));
        }
    }

    fn decrement(&mut self) {
        if let Ok(paths) = &self.files {
            let new = if self.index() == 0 {
                paths.len().saturating_sub(1)
            } else {
                self.index() - 1
            };
            self.state.select(Some(new));
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
    Submit(PathBuf),
    Cancel,
    Handled,
    None,
}

impl Component for Explorer {
    type Response = ExplorerResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
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
                            return ExplorerResponse::Submit(path);
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
                _ => ExplorerResponse::None,
            }
        } else {
            ExplorerResponse::None
        }
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let outer = Block::default()
            .title(Span::styled("Explorer", Style::default()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let inner = outer.inner(rect);

        Widget::render(outer, rect, buf);
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
                StatefulWidget::render(table, inner, buf, &mut self.state);
            }
            Err(e) => {
                let p = Paragraph::new(Span::styled(e, Style::default().fg(Color::Red)));
                Widget::render(p, inner, buf);
            }
        }
    }
}
