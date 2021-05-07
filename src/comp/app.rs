use super::{
    Component, Confirm, ConfirmResponse, Event, Explorer, ExplorerMode, ExplorerResponse, Filter,
    FilterResponse, Tree, TreeResponse,
};
use crate::rect_ext::RectExt;
use std::env::current_dir;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use prc::{open, param::*, save};
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, TableState, Widget};

pub struct App {
    /// The owned param struct
    base: ParamKind,
    /// The app mode
    mode: AppMode,
    /// Selection state on the current param
    tail: Tree,
    /// The past levels' selection info and param names
    route: Vec<RouteInfo>,
    /// regex filter for the current level's param names
    filter: Filter,
    /// The last directory a file was opened in
    open_dir: PathBuf,
    /// The last directory a file was saved in
    save_dir: PathBuf,
    /// Whether unsaved changes were made
    unsaved: bool,
}

#[derive(Debug)]
enum AppMode {
    ParamView,
    FileOpen(Box<Explorer>),
    RegexEdit,
    ConfirmOpen(Confirm),
    ConfirmExit(Confirm),
}

struct RouteInfo {
    table: TableState,
    index: usize,
    name: String,
}

impl App {
    pub fn new(param: ParamKind) -> Self {
        let filter = Filter::new();
        let tail = Tree::new(&param, filter.regex());
        App {
            base: param,
            mode: AppMode::ParamView,
            tail,
            route: vec![],
            filter,
            open_dir: current_dir().unwrap(),
            save_dir: current_dir().unwrap(),
            unsaved: false,
        }
    }

    pub fn set_param(&mut self, param: ParamKind) {
        let filter = Filter::new();
        let tail = Tree::new(&param, filter.regex());
        self.base = param;
        self.mode = AppMode::ParamView;
        self.tail = tail;
        self.route.clear();
        self.filter = filter;
        self.unsaved = false;
    }

    pub fn current_param(&self) -> &ParamKind {
        let mut ptr = &self.base;
        for route in self.route.iter() {
            match ptr {
                ParamKind::Struct(s) => ptr = &s.0[route.index()].1,
                ParamKind::List(l) => ptr = &l.0[route.index()],
                _ => panic!("Only struct or list params can be indexed"),
            }
        }
        ptr
    }

    pub fn current_param_mut(&mut self) -> &mut ParamKind {
        let mut ptr = &mut self.base;
        for route in self.route.iter() {
            match ptr {
                ParamKind::Struct(s) => ptr = &mut s.0[route.index()].1,
                ParamKind::List(l) => ptr = &mut l.0[route.index()],
                _ => panic!("Only struct or list params can be indexed"),
            }
        }
        ptr
    }

    pub fn open_file(&mut self) {
        self.mode = AppMode::FileOpen(Box::new(Explorer::new(self.open_dir.clone(), ExplorerMode::Open)));
    }

    pub fn save_file(&mut self) {
        self.mode = AppMode::FileOpen(Box::new(Explorer::new(self.save_dir.clone(), ExplorerMode::Save)));
    }

    pub fn confirm_open(&mut self) {
        self.mode = AppMode::ConfirmOpen(Confirm::new("Unsaved changes. Confirm open?"));
    }

    pub fn confirm_exit(&mut self) {
        self.mode = AppMode::ConfirmExit(Confirm::new("Unsaved changes. Confirm exit?"))
    }
}

impl RouteInfo {
    pub fn new(tree: &Tree) -> Self {
        let table = tree.table_state().clone();
        let row = tree.current_row().unwrap();
        RouteInfo {
            table,
            index: row.index,
            name: row.name.clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.table.selected().unwrap()
    }
}

pub enum AppResponse {
    None,
    Exit,
}

impl Component for App {
    type Response = AppResponse;

    fn handle_event(&mut self, event: Event) -> Self::Response {
        match &mut self.mode {
            AppMode::ParamView => {
                match self.tail.handle_event(event) {
                    TreeResponse::Focus => {
                        if let Some(row) = self.tail.current_row() {
                            if row.is_parent {
                                self.route.push(RouteInfo::new(&self.tail));
                                self.tail = Tree::new(self.current_param(), self.filter.regex());
                            } else {
                                self.tail.start_editing();
                            }
                        }
                    }
                    TreeResponse::Unfocus => {
                        if !self.route.is_empty() {
                            let last_state = self.route.pop().unwrap();
                            let index = last_state.index;
                            self.tail = Tree::new_with_state(
                                self.current_param(),
                                self.filter.regex(),
                                last_state.table,
                            );
                            self.tail.select_param_index(index);
                        }
                    }
                    TreeResponse::SetValue(index, value) => {
                        let parent = self.current_param_mut();
                        let child = match parent {
                            ParamKind::Struct(s) => &mut s.0[index].1,
                            ParamKind::List(l) => &mut l.0[index],
                            _ => panic!("Only struct or list params can be indexed"),
                        };
                        macro_rules! parse {
                            ($($param_kind:ident),*) => { match child {
                                $(ParamKind::$param_kind(v) => match value.parse() {
                                    Ok(value) => {
                                        *v = value;
                                        self.tail.current_row_mut().unwrap().value = format!("{}", v);
                                        self.tail.finish_editing();
                                        self.unsaved = true;
                                    }
                                    Err(_) => self.tail.set_editing_error(Some("[Parse error]".into())),
                                })*
                                _ => panic!("Only value type params can be set"),
                            }}
                        }
                        parse!(Bool, I8, U8, I16, U16, I32, U32, Float, Hash, Str);
                    }
                    TreeResponse::None => {
                        // app events here
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                KeyCode::Char('/') => {
                                    self.mode = AppMode::RegexEdit;
                                    self.filter.focus(true);
                                    self.tail.focus(false);
                                }
                                KeyCode::Char('o')
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    if self.unsaved {
                                        self.confirm_open();
                                    } else {
                                        self.open_file();
                                    }
                                }
                                KeyCode::Char('s')
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    self.save_file();
                                }
                                KeyCode::Esc => {
                                    if self.unsaved {
                                        self.confirm_exit();
                                    } else {
                                        return AppResponse::Exit;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    TreeResponse::Handled => {}
                }
            }
            AppMode::RegexEdit => match self.filter.handle_event(event) {
                FilterResponse::Exit => {
                    self.mode = AppMode::ParamView;
                    self.filter.focus(false);
                    self.tail.focus(true);

                    let param = self.current_param();
                    let regex = self.filter.regex();
                    let state = self.tail.table_state().clone();
                    let param_index = self.tail.current_row().map(|r| r.index);

                    self.tail = Tree::new_with_state(param, regex, state);
                    if let Some(i) = param_index {
                        self.tail.select_param_index(i);
                    }
                }
                FilterResponse::None => {}
            },
            AppMode::FileOpen(exp) => match exp.handle_event(event) {
                ExplorerResponse::Open(path) => {
                    if let Some(parent) = path.parent() {
                        self.open_dir = parent.to_path_buf();
                    }
                    let res = open(path).map(ParamKind::from);
                    match res {
                        Ok(param) => self.set_param(param),
                        Err(_e) => { /* Log error? Display error prompt? */ }
                    }
                }
                ExplorerResponse::Save(path) => {
                    if let Some(parent) = path.parent() {
                        self.save_dir = parent.to_path_buf();
                    }
                    match save(path, self.base.try_into_ref().unwrap()) {
                        Ok(()) => self.unsaved = false,
                        Err(_e) => { /* Log error? Display error prompt? */ }
                    }
                    self.mode = AppMode::ParamView;
                }
                ExplorerResponse::Cancel => self.mode = AppMode::ParamView,
                ExplorerResponse::Handled => {}
                ExplorerResponse::None => {}
            },
            AppMode::ConfirmOpen(confirm) => if let ConfirmResponse::Confirm(yes) = confirm.handle_event(event) {
                if yes {
                    self.open_file()
                } else {
                    self.mode = AppMode::ParamView;
                }
            },
            AppMode::ConfirmExit(confirm) => if let ConfirmResponse::Confirm(yes) = confirm.handle_event(event) {
                if yes {
                    return AppResponse::Exit;
                } else {
                    self.mode = AppMode::ParamView;
                }
            },
        }

        AppResponse::None
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        let route = if self.route.is_empty() {
            None
        } else {
            let mut route = vec![Span::styled(
                &self.route[0].name,
                Style::default().fg(Color::Green),
            )];
            for r in self.route[1..].iter() {
                route.push(Span::raw(" > "));
                route.push(Span::styled(&r.name, Style::default().fg(Color::Green)));
            }
            Some(Paragraph::new(Spans::from(route)))
        };
        let constraints = Layout::default()
            .constraints(vec![
                // route
                Constraint::Length(if route.is_some() { 2 } else { 0 }),
                // filter
                Constraint::Length(2),
                // the rest filled with the param tree
                Constraint::Percentage(100),
            ])
            .split(rect);
        route.map_or((), |p| {
            let block = Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .title("ROUTE");
            let inner = block.inner(constraints[0]);
            block.render(constraints[0], buf);
            p.render(inner, buf);
        });
        self.filter.draw(constraints[1], buf);
        self.tail.draw(constraints[2], buf);

        match &mut self.mode {
            AppMode::FileOpen(exp) => {
                let center_area = rect.centered(rect.scaled(0.75, 0.75));
                let clear = Clear;
                clear.render(center_area, buf);
                exp.draw(center_area, buf);
            }
            AppMode::ConfirmOpen(confirm) => confirm.draw(rect, buf),
            AppMode::ConfirmExit(confirm) => confirm.draw(rect, buf),
            _ => {}
        }
    }
}
