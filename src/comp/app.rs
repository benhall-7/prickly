use super::{Component, Event, Explorer, ExplorerMode, ExplorerResponse, Filter, FilterResponse, Tree, TreeResponse};

use crossterm::event::{KeyCode, KeyModifiers};
use prc::param::*;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Clear, Borders, Paragraph, TableState, Widget};

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

#[derive(Debug)]
enum AppMode {
    ParamView,
    FileOpen(Explorer),
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
                                KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                    self.mode = AppMode::FileOpen(Explorer::new("./", ExplorerMode::Open));
                                }
                                KeyCode::Esc => return AppResponse::Exit,
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
                ExplorerResponse::Submit(_) => todo!(),
                ExplorerResponse::Cancel => self.mode = AppMode::ParamView,
                ExplorerResponse::Handled => {}
                ExplorerResponse::None => {}
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
                let mul = 0.75;
                let center_area = Rect {
                    width: (mul * rect.width as f32) as u16,
                    height: (mul * rect.height as f32) as u16,
                    x: rect.x + ((1.0 - mul) / 2.0 * rect.width as f32) as u16,
                    y: rect.y + ((1.0 - mul) / 2.0 * rect.height as f32) as u16,
                };
                let clear = Clear;
                clear.render(center_area, buf);
                exp.draw(center_area, buf);
            }
            _ => {}
        }
    }
}
