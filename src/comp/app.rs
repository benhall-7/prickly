use super::{Component, Event, Tree, TreeResponse};

use crossterm::event::KeyCode;
use prc::param::*;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, TableState, Widget};

pub struct App {
    /// The owned param struct
    base: ParamKind,
    /// The current level's selection info
    tail: TableState,
    /// The past levels' selection info and param names
    route: Vec<RouteInfo>,
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
            tail: new_table(),
            route: vec![],
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
        Tree::new(ptr, &mut self.tail)
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
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Esc => return AppResponse::Exit,
                _ => {}
            }
        }

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
        AppResponse::None
    }

    fn draw(&mut self, rect: Rect, buf: &mut Buffer) {
        // top bar shows the path so far
        if !self.route.is_empty() {
            let constraints = Layout::default()
                .constraints(vec![Constraint::Length(1), Constraint::Percentage(100)])
                .split(rect);
            let mut route = vec![Span::styled(
                &self.route[0].name,
                Style::default().fg(Color::Green),
            )];
            for r in self.route[1..].iter() {
                route.push(Span::raw(" > "));
                route.push(Span::styled(&r.name, Style::default().fg(Color::Green)));
            }
            let p = Paragraph::new(Spans::from(route));
            p.render(constraints[0], buf);
            self.current_tree().draw(constraints[1], buf);
        } else {
            self.current_tree().draw(rect, buf);
        }
    }
}
