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
    tail: TableState,
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
    name: String,
}

fn new_table() -> TableState {
    let mut t = TableState::default();
    t.select(Some(0));
    t
}

impl App {
    pub fn new(param: ParamKind) -> Self {
        App {
            base: param,
            mode: AppMode::ParamView,
            tail: new_table(),
            route: vec![],
            filter: Filter::new(),
        }
    }

    pub fn current_tree(&mut self) -> Tree {
        let mut ptr = &mut self.base;
        for route in self.route.iter() {
            match ptr {
                ParamKind::Struct(s) => ptr = &mut s.0[route.index()].1,
                ParamKind::List(l) => ptr = &mut l.0[route.index()],
                _ => panic!("Only struct or list params can be indexed"),
            }
        }
        let focused = self.mode == AppMode::ParamView;
        Tree::new(ptr, &mut self.tail, focused, self.filter.regex())
    }
}

impl RouteInfo {
    pub fn new(tree: &Tree) -> Self {
        let index = tree.selection.selected().unwrap();
        let name = tree.param.name_at(index);
        RouteInfo {
            table: tree.selection.clone(),
            name,
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
                let mut tree = self.current_tree();
                match tree.handle_event(event) {
                    TreeResponse::Focus => match tree.current_param() {
                        ParamKind::Struct(_) | ParamKind::List(_) => {
                            let this_route = RouteInfo::new(&tree);
                            self.route.push(this_route);
                            self.tail = new_table();
                        }
                        _ => {}
                    },
                    TreeResponse::Unfocus => {
                        if !self.route.is_empty() {
                            self.tail = self.route.pop().unwrap().table;
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
                        }
                        KeyCode::Esc => return AppResponse::Exit,
                        _ => {}
                    }
                }
            }
            AppMode::RegexEdit => match self.filter.handle_event(event) {
                FilterResponse::Exit => {
                    self.filter.focus(false);
                    self.mode = AppMode::ParamView;
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
        self.current_tree().draw(constraints[2], buf);
    }
}
