use super::{Component, Event, Filter, FilterResponse, Tree, TreeResponse};

use crossterm::event::KeyCode;
use prc::param::*;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, TableState, Widget};

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
}

#[derive(Debug, PartialEq)]
enum AppMode {
    ParamView,
    RegexEdit,
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
        }
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
        &ptr
    }
}

impl RouteInfo {
    pub fn new(tree: &Tree) -> Self {
        let table = tree.table_state().clone();
        let row = tree.current_row();
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
        match self.mode {
            AppMode::ParamView => {
                match self.tail.handle_event(event) {
                    TreeResponse::Focus => {
                        if !self.tail.is_empty() && self.tail.current_row().is_parent {
                            self.route.push(RouteInfo::new(&self.tail));
                            self.tail = Tree::new(self.current_param(), self.filter.regex());
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
                    TreeResponse::None => {}
                }

                // app events here
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('f') => {
                            self.mode = AppMode::RegexEdit;
                            self.filter.focus(true);
                            self.tail.focus(false);
                        }
                        KeyCode::Esc => return AppResponse::Exit,
                        _ => {}
                    }
                }
            }
            AppMode::RegexEdit => match self.filter.handle_event(event) {
                FilterResponse::Exit => {
                    self.mode = AppMode::ParamView;
                    self.filter.focus(false);
                    self.tail.focus(true);

                    let p = self.current_param();
                    let r = self.filter.regex();
                    let s = self.tail.table_state().clone();
                    let i = self.tail.current_row().index;

                    self.tail = Tree::new_with_state(p, r, s);
                    self.tail.select_param_index(i);
                }
                FilterResponse::None => {}
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
    }
}
